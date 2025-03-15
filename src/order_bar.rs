use crate::ingredients::{IngredientStack, MovableIngredient};
use crate::interpolable::{Interpolable, Pos2d, InterpolableStore};
use crate::traits::{BaseGame, Image};

use std::cell::RefCell;
use std::rc::Rc;

// Manager for the list of orders at the top of the screen

pub struct OrderBar {
    orders: Vec<IngredientStack>,
    pos: Pos2d,
    new_item_timer: Interpolable<f64>,
    submitted_order: Option<IngredientStack>,
}

impl OrderBar {
    pub fn new(xpos: f64, ypos: f64) -> Rc<RefCell<Self>> {
        let new_item_timer = Interpolable::new(0.0, 1.0);
        new_item_timer.set_end(3.0);

        let ret= Rc::new(RefCell::new(OrderBar {
            orders: Vec::new(),
            pos: Pos2d{xpos:xpos, ypos:ypos},
            new_item_timer: Interpolable::new(0.0, 1.0),
            submitted_order: None,
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
        for i in 0..self.orders.len() {
            self.orders[i].draw(game);
        }

        for submitted in self.submitted_order.iter() {
            submitted.draw(game);
        }

        if game.config().draw_borders {
            game.draw_border(self.pos.xpos, self.pos.ypos, 600.0, IngredientStack::height());
        }
    }

    pub fn try_submit_order(&mut self, order: IngredientStack, self_rc: Rc<RefCell<Self>>) -> bool{
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
            order.pos.set_end(bar_order.pos.cur());

            let local_self_rc = self_rc.clone();

            order.pos.set_moved_handler(Box::new(move || {
                let next_local_self_rc =local_self_rc.clone();
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
        let cur_pos = order.pos.cur();
        order.pos.set_end(Pos2d{xpos:cur_pos.xpos, ypos: -100.0});

        for i in (order_idx+1)..self.orders.len() {
            self.orders[i].pos.set_end(self.orders[i-1].pos.cur());
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

        let mut new_order = IngredientStack::new(self.pos.xpos + 1000.0, self.pos.ypos);
        for ing in orders[ord_idx].iter() {
            let ing_chance = js_sys::Math::random();
            if ing_chance > ing.chance {
                continue;
            }

            new_order.add_ingredient(
                MovableIngredient::new(ing.image, 0.0, 0.0, 800.0),
                &self.pos,
                true);
        }

        let end_xpos = 20.0 + 120.0*self.orders.len() as f64;
        new_order.pos.set_end(Pos2d{xpos: end_xpos, ypos: self.pos.ypos});

        self.orders.push(new_order);

        if self.orders.len() < 5 {
            self.new_item_timer.set_cur(0.0);
        }
    }

    pub fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        dest.interpolables_1d.push(self.new_item_timer.clone());

        for order in self.orders.iter() {
            order.collect_interpolables(dest);
        }

        for submitted in self.submitted_order.iter() {
            submitted.collect_interpolables(dest);
        }
    }
}
