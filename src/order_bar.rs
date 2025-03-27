use crate::ingredients::{IngredientStack, MovableIngredient};
use crate::interpolable::{Interpolable, Pos2d};
use crate::traits::{BaseGame, Image, OrderBarConfig};
use crate::utils::WordBank;

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
    keyword: Rc<String>,
    state: OrderBarStackState,
}

impl OrderBarStack {
    fn new(pos: Interpolable<Pos2d>, keyword: &Rc<String>, price: i32, depreciation_seconds: f64) -> Self {
        let mut stack = IngredientStack::new(pos);

        stack.progress = Some(Interpolable::new(0.0, 1.0/depreciation_seconds));
        stack.text = Some(format!("$ {}", price));
        stack.sub_text = Some(keyword.clone().to_string());

        OrderBarStack {
            stack: stack,
            price: price,
            keyword: keyword.clone(),
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
        self.stack.text = Some(format!("$ {}", self.price));
    } 
}

// Manager for the list of orders at the top of the screen
pub struct OrderBar {
    orders: Vec<OrderBarStack>,
    orders_remaining: i32,
    pos: Interpolable<Pos2d>,
    new_item_timer: Interpolable<f64>,
    cfg: OrderBarConfig,
}

impl OrderBar {
    pub fn new(cfg: &OrderBarConfig) -> Self {
        let new_item_timer = Interpolable::new(0.0, 1.0);
        new_item_timer.set_end(4.0);

        OrderBar {
            orders: Vec::new(),
            orders_remaining: 10,
            pos: Interpolable::new(Pos2d::new(cfg.xpos, cfg.ypos), 1000.0),
            new_item_timer: new_item_timer,
            cfg: cfg.clone(),
        }
    }

    pub fn think(&mut self, game: &dyn BaseGame) {
        if self.new_item_timer.advance(game.elapsed_time()) {
            self.create_order(game.word_bank());
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
                if *my_order.keyword != *keyword {
                    continue;
                }

                // Found a matching order.  Send all matching ingredients to it
                my_order.stack.try_ungray_ingredients(selected_ings);
                return true;
            }
        }

        false
    }

    pub fn draw(&self, game: &dyn BaseGame) {
        game.draw_area_background(&self.pos.cur(), &self.cfg.bg);

        for i in 0..self.orders.len() {
            let order = &self.orders[i];
            order.stack.draw(game, &self.cfg.text_price, &self.cfg.progress_bar);
            order.stack.draw_sub_text(game, &self.cfg.text_keyword);
        }

        game.draw_text(&format!("Remaining: {}", self.orders_remaining), &self.pos.cur(), &self.cfg.text_remaining);

        if game.config().draw_borders {
            game.draw_border(self.pos.cur().xpos, self.pos.cur().ypos, 600.0, IngredientStack::height());
        }
    }

    pub fn create_order(&mut self, word_bank: &WordBank) {
        if self.orders_remaining == 0 {
            return;
        }
        self.orders_remaining -= 1;
        
        struct Ing {
            image: Image,
            chance: f64,
        }

        fn ing(image: Image, chance: f64) -> Ing {
            Ing {
                image: image,
                chance: chance,
            }
        }

        type Order = Vec<Ing>;
        let orders:Vec<Order> = vec![
            vec![ing(Image::BurgerBottom, 1.0), ing(Image::CookedPatty, 1.0), ing(Image::LettuceLeaf, 0.5), ing(Image::TomatoSlice, 0.5), ing(Image::BurgerTop, 1.0)],
        ];

        let ord_idx = (js_sys::Math::random() * (orders.len() as f64)) as usize;

        let mut new_order = OrderBarStack::new(Interpolable::new_b(
            Pos2d::new(1000.0, 0.0),
            1000.0,
            &self.pos),
            &word_bank.get_new_word(),
        0,
        self.cfg.depreciation_seconds);

        new_order.stack.pos.set_end(Pos2d::new(
            20.0 + 120.0*self.orders.len() as f64,
            0.0));
        
        for ing in orders[ord_idx].iter() {
            let ing_chance = js_sys::Math::random();
            if ing_chance > ing.chance {
                continue;
            }

            let mut ing = MovableIngredient::new(ing.image, Interpolable::new(Pos2d::new(0.0, 0.0), 1000.0));
            ing.grayed_out = true;

            new_order.stack.add_ingredient(ing, true);
        }

        let price = (new_order.stack.ingredients.len()  * 6) as i32;
        new_order.set_price(price);


        self.orders.push(new_order);

        if self.orders.len() < 5 {
            self.new_item_timer.set_cur(0.0);
        }
    }

    pub fn update_config(&mut self, cfg: &OrderBarConfig) {
        self.pos.set_end(Pos2d::new(cfg.xpos, cfg.ypos));
        self.cfg = cfg.clone();
    }
}
