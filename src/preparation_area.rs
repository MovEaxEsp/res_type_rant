
use crate::ingredients::{IngredientStack, MovableIngredient};
use crate::interpolable::{Interpolable, InterpolableStore, Pos2d};
use crate::order_bar::OrderBar;
use crate::traits::{BaseGame, Image};
use crate::utils::WordBank;

use std::cell::RefCell;
use std::rc::Rc;

///////// PreparationAreaStack
struct PreparationAreaStack {
    stack: IngredientStack,
    base_image: Image,
    is_selected: bool,
    keyword: Rc<String>,
    end_progress: Interpolable<f64>,
}

impl PreparationAreaStack {
    fn new(xpos: f64, ypos: f64, base_image: &Image, keyword: Rc<String>) -> Self {
        PreparationAreaStack {
            stack: IngredientStack::new(xpos, ypos),
            base_image: *base_image,
            is_selected: false,
            keyword: keyword,
            end_progress: Interpolable::new(0.0, 0.2),
        }
    }

    fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        self.stack.collect_interpolables(dest);

        dest.interpolables_1d.push(self.end_progress.clone());
    }

    fn draw(&self, state: &dyn BaseGame) {

        let props = state.image_props(&self.base_image);
        if self.is_selected{
            state.draw_halo(
                self.stack.pos.cur().xpos,
                self.stack.pos.cur().ypos + IngredientStack::height(),
                props.width*1.2,
                props.height*1.2);  
        }
        else {
            state.draw_command_text(
                self.stack.pos.cur().xpos,
                self.stack.pos.cur().ypos + IngredientStack::height() + props.height + 50.0,
                &self.keyword);
        }
        state.draw_image(
            &self.base_image,
            self.stack.pos.cur().xpos + 10.0,
            self.stack.pos.cur().ypos + IngredientStack::height());

        if self.end_progress.is_moving() {
            // Draw the ingredients transitioning to their cooked versions, if they have one

            // Make a temporary stack of the cooked versions of our ingredients
            let cooked_stack = IngredientStack::new(self.stack.pos.cur().xpos, self.stack.pos.cur().ypos);
            for ing in self.stack.ingredients.iter() {
                let mut cooked_ing = ing.image;
                if let Some(cooked_img) = state.image_props(&ing.image).cooked_image {
                    cooked_ing = cooked_img;
                }

                cooked_stack.add_ingredient(cooked_ing, &Pos2d{xpos: 0.0, ypos: 0.0}, false);
                todo finish here
            }
        }
    }
}

///////// PreparationArea
pub struct PreparationArea {
    xpos: f64,
    ypos: f64,
    plate: PreparationAreaStack,
    pan: PreparationAreaStack,
}

impl PreparationArea {
    pub fn new(xpos: f64, ypos: f64, word_bank: &WordBank) -> Rc<RefCell<Self>> {

        let ret = Rc::new(RefCell::new(PreparationArea {
            xpos: xpos,
            ypos: ypos,
            plate: PreparationAreaStack::new(xpos+10.0, ypos+10.0, &Image::Plate, word_bank.get_new_word()),
            pan: PreparationAreaStack::new(xpos + 180.0, ypos+10.0,&Image::Pan, word_bank.get_new_word()),
        }));

        ret.borrow_mut().plate.is_selected = true;

        let cb_ref = ret.clone();
        ret.borrow().pan.end_progress.set_moved_handler(Box::new(move || {
            let cb_self = cb_ref.borrow_mut();
            cb_self.handle_pan_done();
        }));

        ret
    }

    pub fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        self.plate.collect_interpolables(dest);
        self.pan.collect_interpolables(dest);
    }

    pub fn handle_pan_done(&self) {
        // Nothing for now?
    }

    pub fn send_ingredient(&mut self, ingredient: MovableIngredient, cur_base_pos: &Pos2d, word_bank: &WordBank) {
        if self.plate.is_selected {

            if ingredient.image != Image::RawPatty {
                self.plate.stack.add_ingredient(ingredient, cur_base_pos, false);
            }
        }
        else if self.pan.is_selected {
            // Always go back to plate after sending something to pan
            self.pan.keyword = word_bank.get_new_word();
            self.pan.is_selected = false;
            self.plate.is_selected = true;

            if ingredient.image == Image::RawPatty {
                self.pan.stack.add_ingredient(ingredient, cur_base_pos, false);
                self.pan.end_progress.set_end(1.0);
            }
        }
    }

    pub fn handle_command(&mut self, command: &String, order_bar: &Rc<RefCell<OrderBar>>, word_bank: &WordBank) -> bool {
        if  command == "trash" {
            self.plate.stack.ingredients.clear();
            return true;
        }
        
        if command == "send" {
            let order_bar_rc= order_bar.clone();

            let done_order = &mut self.plate.stack;
            let mut new_order = IngredientStack::new(done_order.pos.cur().xpos,
                                                                      done_order.pos.cur().ypos); 
            std::mem::swap(&mut new_order.ingredients, &mut done_order.ingredients);

            order_bar.borrow_mut().try_submit_order(new_order,
                                                    order_bar_rc);
            return true;
        }
        
        if !self.plate.is_selected {
            if *self.plate.keyword == *command {
                self.plate.is_selected = true;
                self.plate.keyword = word_bank.get_new_word();
                self.pan.is_selected = false;
                return true;
            }
        }

        if !self.pan.is_selected {
            if *self.pan.keyword == *command {
                self.pan.is_selected = true;
                self.pan.keyword = word_bank.get_new_word();
                self.plate.is_selected = false;
                return true;
            }
        }

        return false;
    }

    pub fn draw(&self, game: &dyn BaseGame) {

        let plate_props = game.image_props(&Image::Plate);
        if let Some(word) = &self.plate_word {
            game.draw_command_text(self.xpos, self.ypos + IngredientStack::height() + plate_props.height + 50.0,&word);
        }
        else {
            game.draw_halo(self.xpos+10.0, self.ypos + IngredientStack::height(), plate_props.width*1.2, plate_props.height*1.2);
        }
        game.draw_image(&Image::Plate, self.xpos + 10.0, self.ypos + IngredientStack::height());

        let pan_props = game.image_props(&Image::Pan);
        if let Some(word) = &self.pan_word {
            game.draw_command_text(self.xpos + 190.0, self.ypos + IngredientStack::height() + pan_props.height + 50.0,&word);
        }
        else {
            game.draw_halo(self.xpos+180.0, self.ypos + IngredientStack::height(), pan_props.width*1.2, pan_props.height*1.2);
        }
        game.draw_image(&Image::Pan, self.xpos + 180.0, self.ypos + IngredientStack::height());

        self.plate.draw(game);
        self.pan.draw(game);

        if game.config().draw_borders {
            game.draw_border(self.xpos, self.ypos, 300.0, IngredientStack::height());
        }
    }
}
