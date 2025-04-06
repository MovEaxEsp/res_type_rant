
use crate::images::Image;
use crate::ingredients::{IngredientStack, MovableIngredient};
use crate::interpolable::{Interpolable, Pos2d};
use crate::traits::{BaseGame, OrderBarConfig, OrderConfig, OrderIngredientConfig, BackgroundConfig, TextConfig, ProgressBarConfig};

use std::rc::Rc;
use std::usize::MAX;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(PartialEq)]
enum OrderBarStackState {
    Normal,
    Serving,
}
struct OrderBarStackThinkRet {
    start_serving: bool,
    pos_done: bool,
}

struct OrderBarStack {
    stack: IngredientStack,
    price: i32,
    state: OrderBarStackState,
}

impl OrderBarStack {
    fn new(pos: Interpolable<Pos2d>, keyword: &Rc<String>, price: i32, depreciation_seconds: f64) -> Self {
        let mut stack = IngredientStack::new(pos);

        stack.progress = Some(Interpolable::new(0.0, 1.0/depreciation_seconds));
        stack.text = Some(Rc::new(format!("$ {}", price)));
        stack.sub_text = Some(keyword.clone());

        OrderBarStack {
            stack: stack,
            price: price,
            state: OrderBarStackState::Normal,
        }
    }

    fn think(&mut self, game: &dyn BaseGame) -> OrderBarStackThinkRet {
        let mut ret = OrderBarStackThinkRet {
            start_serving: false,
            pos_done: false,
        };

        let stack_ret = self.stack.think(game);

        if stack_ret.pos_done {
            if let Some(inner_progress) = &self.stack.progress {
                if self.price > 1 && self.state == OrderBarStackState::Normal {
                    inner_progress.set_cur(1.0);
                }
            }
        }

        if stack_ret.progress_done {
            self.set_price(self.price-1);
            if self.price > 1 {
                if let Some(inner_progress) = &self.stack.progress {
                    inner_progress.set_cur(1.0);
                }
            }
        }

        if stack_ret.all_ungrayed {
            // Serve order
            self.state = OrderBarStackState::Serving;
            self.stack.pos.set_end(self.stack.pos.own_cur() + Pos2d::new(0.0, -300.0));
            ret.start_serving = true;
        }

        ret.pos_done = stack_ret.pos_done;

        ret
    }

    fn set_price(&mut self, price: i32) {
        self.price = price;
        self.stack.text = Some(Rc::new(format!("$ {}", self.price)));
    } 
}

// Manager for the list of orders at the top of the screen
pub struct OrderBar {
    orders: Vec<OrderBarStack>,
    orders_remaining: i32,
    pos: Interpolable<Pos2d>,
    new_item_timer: Interpolable<f64>,
}

impl OrderBar {
    pub fn new(cfg: &OrderBarConfig) -> Self {
        let new_item_timer = Interpolable::new(0.0, 1.0);
        new_item_timer.set_end(4.0);

        OrderBar {
            orders: Vec::new(),
            orders_remaining: 10,
            pos: Interpolable::new(cfg.pos, 1000.0),
            new_item_timer: new_item_timer,
        }
    }

    pub fn think(&mut self, game: &dyn BaseGame) {
        if self.new_item_timer.advance(game.elapsed_time()) {
            self.create_order(game);
        }

        self.pos.advance(game.elapsed_time());

        let mut served_idx = MAX;
        for order_idx in 0..self.orders.len() {
            let think_ret: OrderBarStackThinkRet;

            {
                let order = &mut self.orders[order_idx];
                think_ret = order.think(game);

                if think_ret.pos_done {
                    if order.state == OrderBarStackState::Serving {
                        game.add_money(self.orders[order_idx].price);
                        served_idx = order_idx;
                    }
                }
            }

            if think_ret.start_serving {
                for i in (order_idx+1)..self.orders.len() {
                    self.orders[i].stack.pos.set_end(self.orders[i-1].stack.pos.own_cur());
                }    
            }
        }

        if served_idx != MAX {
            self.orders.remove(served_idx);

            
            if self.orders.len() < 3 {
                self.new_item_timer.set_cur(0.0);
            }
        }
    }

    pub fn handle_command(&mut self, keywords: &Vec<&str>, selected_ings: &mut Vec<MovableIngredient>, _game:&dyn BaseGame) -> bool {
        for keyword in keywords.iter() {
            for my_order in self.orders.iter_mut() {
                if let Some(stack_word) = &my_order.stack.sub_text {
                    if **stack_word == *keyword {
                        // Found a matching order.  Send all matching ingredients to it
                        my_order.stack.try_ungray_ingredients(selected_ings);
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn draw(&self, game: &dyn BaseGame) {
        let cfg = &game.config().order_bar;

        game.draw_area_background(&self.pos.cur(), &cfg.bg);

        for i in 0..self.orders.len() {
            let order = &self.orders[i];
            order.stack.draw(game, Some(&cfg.progress_bar), Some(&cfg.text_price), Some(&cfg.text_keyword));
        }

        game.draw_text(&format!("Remaining: {}", self.orders_remaining), &self.pos.cur(), 1.0, &cfg.text_remaining);
    }

    /// Create a new order in the OrderBar
    pub fn create_order(&mut self, game: &dyn BaseGame) {
        if self.orders_remaining == 0 {
            return;
        }
        self.orders_remaining -= 1;
        
        let cfg = &game.config().order_bar;

        // Figure out which order to make from the config
        let orders = &game.config().order_bar.orders;
        let total_weight: f64 = orders.iter().map(|e| e.weight).sum();
        let mut order_selector = js_sys::Math::random() * (total_weight as f64);
        let mut order_to_make = &orders[0];

        for order in orders.iter() {
            order_selector -= order.weight;
            if order_selector <= 0.0 {
                order_to_make = order;
                break;
            }
        }

        let mut new_order = OrderBarStack::new(Interpolable::new_b(
            Pos2d::new(1000.0, 0.0),
            1000.0,
            &self.pos),
            &game.word_bank().get_new_word(),
        0,
        order_to_make.depreciation_seconds);

        let mut xpos = cfg.order_margin;
        for i in 0..self.orders.len() {
            let order = &self.orders[i];
            xpos += order.stack.width(game) + cfg.order_margin;
        }

        new_order.stack.pos.set_end((xpos, 0).into());
        
        let mut price: i32 = 0;
        for ing in order_to_make.ings.iter() {
            let ing_chance = js_sys::Math::random();
            if ing_chance > ing.chance {
                continue;
            }

            let mut new_ing = MovableIngredient::new(ing.ing, Interpolable::new(Pos2d::new(0.0, 0.0), 1000.0));
            new_ing.grayed_out = true;
            price += ing.price;

            new_order.stack.add_ingredient(new_ing, true, game);
        }

        new_order.set_price(price);

        self.orders.push(new_order);

        if self.orders.len() < 5 {
            self.new_item_timer.set_cur(0.0);
        }
    }

    /// Update our configuration
    pub fn update_config(&mut self, cfg: &OrderBarConfig, game: &dyn BaseGame) {
        self.pos.set_end(cfg.pos);

        let mut xpos = cfg.order_margin;
        for i in 0..self.orders.len() {
            let order = &self.orders[i];
            order.stack.pos.set_end((xpos, 0).into());
            xpos += order.stack.width(game) + cfg.order_margin;
        }
    }

    /// Return the default configuration for the OrderBar 
    pub fn default_config() -> OrderBarConfig {
        OrderBarConfig {
            pos: (1200, 400).into(),
            order_margin: 20.0,
            bg: BackgroundConfig {
                offset: (-50, -300).into(),
                width: 1340.0,
                height: 500.0,
                corner_radius: 30.0,
                border_style: "black".to_string(),
                border_alpha: 1.0,
                border_width: 5.0,
                bg_style: "pink".to_string(),
                bg_alpha: 0.2
            },
            text_price: TextConfig {
                offset: (0, 40).into(),
                stroke: false,
                style: "yellow".to_string(),
                font: "comic sans".to_string(),
                size: 48,
                center_and_fit: true,
                alpha: 0.4,
                is_command: false,
            },
            text_keyword: TextConfig {
                offset: (0, 100).into(),
                stroke: false,
                style: "yellow".to_string(),
                font: "comic sans".to_string(),
                size: 48,
                center_and_fit: true,
                alpha: 0.4,
                is_command: true,
            },
            text_remaining: TextConfig {
                offset: (10, -270).into(),
                stroke: false,
                style: "white".to_string(),
                font: "comic sans".to_string(),
                size: 48,
                center_and_fit: false,
                alpha: 0.4,
                is_command: false,
            },
            progress_bar: ProgressBarConfig {
                bg: BackgroundConfig {
                    offset: (0, 30).into(),
                    width: 100.0,
                    height: 5.0,
                    corner_radius: 0.0,
                    border_style: "black".to_string(),
                    border_alpha: 0.0,
                    border_width: 0.0,
                    bg_style: "black".to_string(),
                    bg_alpha: 0.4
                },
                done_alpha: 1.0,
                done_style: "yellow".to_string(),
            },
            orders: vec![
                OrderConfig { // Burger
                    ings: vec![
                        OrderIngredientConfig {
                            ing: Image::BurgerBottom,
                            chance: 1.0,
                            price: 3,
                        },
                        OrderIngredientConfig {
                            ing: Image::CookedPatty,
                            chance: 1.0,
                            price: 10,
                        },
                        OrderIngredientConfig {
                            ing: Image::LettuceLeaf,
                            chance: 0.7,
                            price: 4,
                        },
                        OrderIngredientConfig {
                            ing: Image::TomatoSlice,
                            chance: 0.6,
                            price: 5,
                        },
                        OrderIngredientConfig {
                            ing: Image::BurgerTop,
                            chance: 1.0,
                            price: 3,
                        }
                    ],
                    weight: 1.0,
                    depreciation_seconds:5.0
                },
                OrderConfig { // Salad
                    ings: vec![
                        OrderIngredientConfig {
                            ing: Image::LettuceLeaf,
                            chance: 1.0,
                            price: 8,
                        },
                        OrderIngredientConfig {
                            ing: Image::TomatoSlice,
                            chance: 1.0,
                            price: 10,
                        },
                    ],
                    weight: 0.5,
                    depreciation_seconds: 5.0
                },
                OrderConfig { // Curry Crab
                    ings: vec![
                        OrderIngredientConfig {
                            ing: Image::CurryCrab,
                            chance: 1.0,
                            price: 30,
                        },
                        OrderIngredientConfig {
                            ing: Image::Dumplings,
                            chance: 1.0,
                            price: 10,
                        },
                    ],
                    weight: 0.5,
                    depreciation_seconds: 8.0
                }
            ]
        }
    }
}
