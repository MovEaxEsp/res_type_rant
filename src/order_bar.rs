
use crate::ingredients::{IngredientStack, MovableIngredient};
use crate::painter::{BackgroundConfig, ProgressBarConfig, TextConfig};
use crate::traits::{BaseGame, Image, Sound};

use engine_p::interpolable::{Interpolable, Pos2d};
use engine_p::sounds::PlaybackConfig;
use serde::{Serialize,Deserialize};
use wasm_bindgen::prelude::*;

use std::collections::HashSet;
use std::rc::Rc;
use std::usize::MAX;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderIngredientConfig {
    pub ing: Image,
    pub chance: f64,
    pub price: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderConfig {
    pub ings: Vec<OrderIngredientConfig>,
    pub weight: f64, // how likely this order is to be chosen
    pub depreciation_seconds: f64, // seconds until order price is reduced
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderBarUiConfig {
    pub pos: Pos2d,
    pub order_margin: f64,
    pub bg: BackgroundConfig,
    pub text_price: TextConfig,
    pub text_keyword: TextConfig,
    pub text_remaining: TextConfig,
    pub progress_bar: ProgressBarConfig,
    pub money_sound: PlaybackConfig<Sound>,
    pub orders: Vec<OrderConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderBarGameConfig {
    pub order_period: f64,
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

// ==========================
// OrderBar
// ==========================

// Manager for the list of orders at the top of the screen
pub struct OrderBar {
    orders: Vec<OrderBarStack>,
    //orders_remaining: i32,
    pos: Interpolable<Pos2d>,
    new_item_timer: Interpolable<f64>,
    available_ings: HashSet<Image>,
}

impl OrderBar {
    /// Create a new OrderBar
    pub fn new(cfg_ui: &OrderBarUiConfig, cfg_game: &OrderBarGameConfig) -> Self {
        let new_item_timer = Interpolable::new(0.0, 1.0);
        new_item_timer.set_end(cfg_game.order_period);

        OrderBar {
            orders: Vec::new(),
            //orders_remaining: 10,
            pos: Interpolable::new(cfg_ui.pos, 1000.0),
            new_item_timer: new_item_timer,
            available_ings: HashSet::new(),
        }
    }

    /// Reset the state of the OrderBar, to start a new day
    pub fn reset_state(&mut self) {
        self.orders.clear();
        self.new_item_timer.set_cur(0.0);
    }

    /// Update the state of the OrderBar for the frame
    pub fn think(&mut self, game: &dyn BaseGame, cfg_ui: &OrderBarUiConfig, cfg_game: &OrderBarGameConfig) {
        if self.new_item_timer.advance(game.elapsed_time()) {
            self.create_order(game, cfg_ui, cfg_game);
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
                        game.sounds().play_sound(&cfg_ui.money_sound);
                        served_idx = order_idx;
                    }
                }
            }
        }

        if served_idx != MAX {
            self.orders.remove(served_idx);

            // Update positions of all our orders
            let mut xpos = cfg_ui.order_margin;
            for ord in self.orders.iter_mut() {
                ord.stack.pos.set_end((xpos, 0).into());
                xpos += ord.stack.width(game) + cfg_ui.order_margin;
            }
            
            if self.orders.len() < 5 {
                self.new_item_timer.set_cur(0.0);
            }
        }
    }

    /// Handle the user typing the specified 'keywords' on the command line
    pub fn handle_command(&mut self, keywords: &Vec<String>, selected_ings: &mut Vec<MovableIngredient>, _game:&dyn BaseGame) -> bool {
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

    /// Draw the OrderBar
    pub fn draw(&self, game: &dyn BaseGame, cfg_ui: &OrderBarUiConfig) {
        game.painter().draw_area_background(&self.pos.cur(), &cfg_ui.bg);

        for i in 0..self.orders.len() {
            let order = &self.orders[i];
            order.stack.draw(game, Some(&cfg_ui.progress_bar), Some(&cfg_ui.text_price), Some(&cfg_ui.text_keyword));
        }

        //game.draw_text(&format!("Remaining: {}", self.orders_remaining), &self.pos.cur(), 1.0, &cfg.text_remaining);
    }

    /// Create a new order in the OrderBar
    pub fn create_order(&mut self, game: &dyn BaseGame, cfg_ui: &OrderBarUiConfig, _cfg_game: &OrderBarGameConfig) {
        //if self.orders_remaining == 0 {
        //    return;
        //}
        //self.orders_remaining -= 1;
        
        // Figure out which order to make from the config

        // .. figure out which orders we can make with the available ingredients
        let mut orders: Vec<&OrderConfig> = Vec::new();
        for order in  cfg_ui.orders.iter() {
            // Can only use an order if all its ingredients are either optional, or present in available_ings
            if order.ings.iter().all(|ing| ing.chance < 1.0 || self.available_ings.contains(&ing.ing)) {
                orders.push(order);
            }
        }

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

        // Figure out location for our new order
        let mut new_order = OrderBarStack::new(Interpolable::new_b(
            Pos2d::new(1000.0, 0.0),
            1000.0,
            &self.pos),
            &game.word_bank().get_new_word(),
        0,
        order_to_make.depreciation_seconds);

        let mut xpos = cfg_ui.order_margin;
        for i in 0..self.orders.len() {
            let order = &self.orders[i];
            xpos += order.stack.width(game) + cfg_ui.order_margin;
        }

        new_order.stack.pos.set_end((xpos, 0).into());
        
        // Figure out the ingredients for 'order_to_make'
        let mut price: i32 = 0;
        for ing in order_to_make.ings.iter() {
            let ing_chance = js_sys::Math::random();
            if ing_chance > ing.chance || !self.available_ings.contains(&ing.ing) {
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

    pub fn set_available_ingredients(&mut self, ings: HashSet<Image>) {
        self.available_ings = ings;
    }

    /// Update our configuration
    pub fn update_config(&mut self, cfg_ui: &OrderBarUiConfig, cfg_game: &OrderBarGameConfig, game: &dyn BaseGame) {
        self.pos.set_end(cfg_ui.pos);

        let mut xpos = cfg_ui.order_margin;
        for i in 0..self.orders.len() {
            let order = &self.orders[i];
            order.stack.pos.set_end((xpos, 0).into());
            xpos += order.stack.width(game) + cfg_ui.order_margin;
        }

        self.new_item_timer.set_end(cfg_game.order_period);
    }
}
