use crate::ingredients::{IngredientStack, MovableIngredient};
use crate::interpolable::{Interpolable, Pos2d, InterpolableStore};
use crate::traits::{BaseGame, Image, OrderBarConfig};

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Manager for the list of orders at the top of the screen

pub struct OrderBar {
    orders: Vec<IngredientStack>,
    pos: Interpolable<Pos2d>,
    new_item_timer: Interpolable<f64>,
    submitted_order: Option<IngredientStack>,
    cfg: OrderBarConfig,
}

impl OrderBar {
    pub fn new(cfg: &OrderBarConfig) -> Rc<RefCell<Self>> {
        let new_item_timer = Interpolable::new(0.0, 1.0);
        new_item_timer.set_end(3.0);

        let ret= Rc::new(RefCell::new(OrderBar {
            orders: Vec::new(),
            pos: Interpolable::new(Pos2d::new(cfg.xpos, cfg.ypos), 1000.0),
            new_item_timer: Interpolable::new(0.0, 1.0),
            submitted_order: None,
            cfg: cfg.clone(),
        }));

        let cb_ret: Rc<RefCell<OrderBar>> = ret.clone();

        ret.borrow().new_item_timer.set_moved_handler(Box::new(move || {
            let inner_ret = cb_ret.clone();
            inner_ret.borrow_mut().create_order();
        }));
        ret.borrow_mut().new_item_timer.set_end(2.0);

        ret
    }

    pub fn draw(&self, game: &dyn BaseGame) {
        game.draw_area_background(&self.pos.cur(), &self.cfg.bg);

        for i in 0..self.orders.len() {
            self.orders[i].draw(game);
        }

        for submitted in self.submitted_order.iter() {
            submitted.draw(game);
        }

        if game.config().draw_borders {
            game.draw_border(self.pos.cur().xpos, self.pos.cur().ypos, 600.0, IngredientStack::height());
        }
    }

    pub fn try_submit_order(&mut self, mut order: IngredientStack, self_rc: Rc<RefCell<Self>>) -> bool{
        for i in 0..self.orders.len() {
            let bar_order = &self.orders[i];
            if order.ingredients.len() != bar_order.ingredients.len() {
                continue;
            }

            let mut is_match = true;
            for ing_idx in 0..order.ingredients.len() {
                if order.ingredients[ing_idx].image != bar_order.ingredients[ing_idx].image {
                    is_match = false;
                    break; 
                }
            }

            if !is_match {
                continue;
            }

            // found a matching order
            order.pos.rebase(&self.pos, bar_order.pos.own_cur(), false);
            order.pos.set_speed(1000.0);

            let local_self_rc = self_rc.clone();

            order.pos.set_moved_handler(Box::new(move || {
                let next_local_self_rc = local_self_rc.clone();
                let mut order_bar = local_self_rc.borrow_mut();
                order_bar.serve_order(i, next_local_self_rc);
                order_bar.submitted_order = None;
            }));

            self.submitted_order = Some(order);
            return true;
        }

        return false;
    }

    pub fn serve_order(&mut self, order_idx: usize, self_rc: Rc<RefCell<Self>>) {
        let order = &self.orders[order_idx];
        let cur_pos = order.pos.own_cur();
        order.pos.set_end(Pos2d::new(cur_pos.xpos,-100.0));

        for i in (order_idx+1)..self.orders.len() {
            self.orders[i].pos.set_end(self.orders[i-1].pos.own_cur());
        }

        let local_self_rc = self_rc.clone();

        order.pos.set_moved_handler(Box::new(move || {
            let mut order_bar = local_self_rc.borrow_mut();

            order_bar.orders.remove(order_idx);
        }));

        if self.orders.len() < 2 {
            self.new_item_timer.set_cur(0.0);
        }
    }

    pub fn create_order(&mut self) {
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

        let mut new_order = IngredientStack::new(Interpolable::new_b(
            Pos2d::new(1000.0, 0.0),
            1000.0,
            &self.pos));

        new_order.pos.set_end(Pos2d::new(
            20.0 + 120.0*self.orders.len() as f64,
            0.0));
        
        for ing in orders[ord_idx].iter() {
            let ing_chance = js_sys::Math::random();
            if ing_chance > ing.chance {
                continue;
            }

            new_order.add_ingredient(
                MovableIngredient::new(ing.image, Interpolable::new(Pos2d::new(0.0, 0.0), 1000.0)),
                true);
        }

        self.orders.push(new_order);

        if self.orders.len() < 5 {
            self.new_item_timer.set_cur(0.0);
        }
    }

    pub fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        dest.interpolables_1d.push(self.new_item_timer.clone());
        dest.interpolables_2d.push(self.pos.clone());

        for order in self.orders.iter() {
            order.collect_interpolables(dest);
        }

        for submitted in self.submitted_order.iter() {
            submitted.collect_interpolables(dest);
        }
    }

    pub fn update_config(&mut self, cfg: &OrderBarConfig) {
        self.pos.set_end(Pos2d::new(cfg.xpos, cfg.ypos));
        self.cfg = cfg.clone();
    }
}
