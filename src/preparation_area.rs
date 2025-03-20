
use crate::ingredients::{IngredientStack, MovableIngredient};
use crate::interpolable::{Interpolable, InterpolableStore, Pos2d};
use crate::order_bar::OrderBar;
use crate::traits::{BaseGame, Image, PreparationAreaConfig, PreparationAreaStackConfig, TextConfig};
use crate::utils::WordBank;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

struct PreparationAreaStack {
    stack: IngredientStack,
    cooked_stack: Option<Vec<Image>>,
    base_image: Image,
    is_selected: bool,
    is_cooked: bool,
    keyword: Rc<String>,
    end_progress: Interpolable<f64>,
}

impl PreparationAreaStack {
    fn new(pos: Interpolable<Pos2d>, base_image: &Image, keyword: Rc<String>, cfg: &PreparationAreaStackConfig) -> Rc<RefCell<Self>> {
        let ret = Rc::new(RefCell::new(PreparationAreaStack {
            stack: IngredientStack::new(pos),
            cooked_stack: None,
            base_image: *base_image,
            is_selected: false,
            is_cooked: false,
            keyword: keyword,
            end_progress: Interpolable::new(1.0, 1.0/cfg.cook_time),
        }));

        let cb_ref = ret.clone();
        ret.borrow().end_progress.set_moved_handler(Box::new(move || {
            // Handle cooking being done
            let mut cb_self = cb_ref.borrow_mut();

            let cooked_stack = cb_self.cooked_stack.take().unwrap();

            // Replace each ingredient with its cooked version
            for i in 0..cb_self.stack.ingredients.len() {
                cb_self.stack.ingredients[i].image = cooked_stack[i];
            }

            cb_self.is_cooked = true;
        }));

        ret
    }

    fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        self.stack.collect_interpolables(dest);

        dest.interpolables_1d.push(self.end_progress.clone());
    }

    fn draw(&self, game: &dyn BaseGame, cfg: &PreparationAreaStackConfig, text_cfg: &TextConfig) {
        let props = game.image_props(&self.base_image);
        if self.is_selected{
            game.draw_halo(
                self.stack.pos.cur().xpos,
                self.stack.pos.cur().ypos + IngredientStack::height(),
                props.width*1.2,
                props.height*1.2);  
        }
        else if self.end_progress.is_moving() {
            // TODO Draw progress bar
        }
        else {
            game.draw_text(&self.keyword, &self.stack.pos.cur(), &text_cfg);
        }
        game.draw_image(
            &self.base_image,
            &(self.stack.pos.cur() + Pos2d::new(cfg.base_x_off, cfg.base_y_off)));

        if let Some(cooked_ings) = &self.cooked_stack {
            // Draw the ingredients transitioning to their cooked versions, if they have one

            // Make a temporary stack of the cooked versions of our ingredients
            let mut cooked_stack = IngredientStack::new(self.stack.pos.clone());
            for i in 0..self.stack.ingredients.len() {
                let cooked_img = &cooked_ings[i];
                let cooked_ing = MovableIngredient::new(*cooked_img, Interpolable::new(Pos2d::new(0.0,0.0), 1000.0));
                cooked_stack.add_ingredient(cooked_ing, true);
            }

            // TODO rewrite PreparationAreaStack using the text/progress of IngStack
            // Draw each stack, so the total alpha = 1
            game.set_global_alpha(1.0 - self.end_progress.cur());
            self.stack.draw_stack(game);

            game.set_global_alpha(self.end_progress.cur());
            cooked_stack.draw_stack(game);

            game.set_global_alpha(1.0);
        }
        else {
            self.stack.draw_stack(game);
        }
    }

    fn start_cooking(&mut self, game: &dyn BaseGame) {
        // Figure out the cooked versions of our ingredients
        let mut cooked_stack: Vec<Image> = Vec::new();
        for ing in self.stack.ingredients.iter() {
            cooked_stack.push(game.image_props(&ing.image).cooked_image.or(Some(ing.image)).unwrap());
        }
        self.cooked_stack = Some(cooked_stack);

        self.end_progress.set_cur(0.0);
    }

    // Return 'true' if the specified 'keyword' matches our keyword, and replace it with a new
    // one from the specified 'word_bank'.
    fn check_keyword(&mut self, keyword: &String, word_bank:&WordBank) -> bool {
        if self.end_progress.is_moving() || self.is_selected{
            // Can't match while we're cooking or already selected
            return false;
        }

        if *keyword == *self.keyword {
            self.keyword = word_bank.get_new_word();
            return true;
        }

        return false;
    }

    fn reset(&mut self) {
        self.is_cooked = false;
        self.stack.ingredients.clear();
        self.cooked_stack = None;
    }
}

pub struct PreparationArea {
    pos: Interpolable<Pos2d>,
    plate: Rc<RefCell<PreparationAreaStack>>,
    pan: Rc<RefCell<PreparationAreaStack>>,
    cfg: PreparationAreaConfig,
}

impl PreparationArea {
    pub fn new(word_bank: &WordBank, cfg: &PreparationAreaConfig) -> Self {

        let pos = Interpolable::new(Pos2d::new(cfg.xpos, cfg.ypos), 1000.0);

        let ret = PreparationArea {
            pos: pos.clone(),
            plate: PreparationAreaStack::new(
                    Interpolable::new_b(Pos2d::new(cfg.plate.xpos, cfg.plate.ypos), 1000.0, &pos.clone()),
                    &Image::Plate,
                    word_bank.get_new_word(),
                    &cfg.plate),
            pan: PreparationAreaStack::new(
                    Interpolable::new_b(Pos2d::new(cfg.pan.xpos, cfg.pan.ypos), 1000.0, &pos.clone()),
                    &Image::Pan,
                    word_bank.get_new_word(),
                    &cfg.pan),
            cfg: cfg.clone()
        };

        ret.plate.borrow_mut().is_selected = true;

        ret
    }

    pub fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        self.plate.borrow().collect_interpolables(dest);
        self.pan.borrow().collect_interpolables(dest);
        dest.interpolables_2d.push(self.pos.clone());
    }

    fn send_ingredient_imp(ingredient: MovableIngredient, game:&dyn BaseGame, plate: &mut PreparationAreaStack, pan: &mut PreparationAreaStack) {
        if plate.is_selected {
            if ingredient.image != Image::RawPatty {
                plate.stack.add_ingredient(ingredient, false);
            }
        }
        else if pan.is_selected {
            // Always go back to plate after sending something to pan
            pan.is_selected = false;
            plate.is_selected = true;

            if ingredient.image == Image::RawPatty {
                pan.stack.add_ingredient(ingredient, false);
                pan.start_cooking(game);
            }
        }
    }

    pub fn send_ingredient(&mut self, ingredient: MovableIngredient, _word_bank: &WordBank, game: &dyn BaseGame) {
        PreparationArea::send_ingredient_imp(ingredient, game,&mut self.plate.borrow_mut(), &mut self.pan.borrow_mut());
    }

    pub fn handle_command(&mut self, keyword: &String, order_bar: &Rc<RefCell<OrderBar>>, word_bank: &WordBank, game:&dyn BaseGame) -> bool {
        let mut plate = self.plate.borrow_mut();
        let mut pan = self.pan.borrow_mut();

        if  keyword == "trash" {
            plate.stack.ingredients.clear();
            return true;
        }
        
        if keyword == "send" {
            let order_bar_rc= order_bar.clone();

            let done_order = &mut plate.stack;
            let mut new_order = IngredientStack::new(Interpolable::new(done_order.pos.cur(), 1000.0)); 
            for ing in done_order.ingredients.iter() {
                new_order.add_ingredient(
                    MovableIngredient::new(ing.image, Interpolable::new(Pos2d::new(0.0, 0.0), 1000.0)),
                    true);
            }
            done_order.ingredients.clear();

            order_bar.borrow_mut().try_submit_order(new_order,
                                                    order_bar_rc);
            return true;
        }
        
        if plate.check_keyword(keyword, word_bank) {
            plate.is_selected = true;
            pan.is_selected = false;
            return true;
        }

        if pan.check_keyword(keyword, word_bank) {
            if pan.is_cooked {
                let cooked_ing = &pan.stack.ingredients[0];
                let new_ing = MovableIngredient::new(cooked_ing.image, Interpolable::new(cooked_ing.pos.cur(), 1000.0));
                PreparationArea::send_ingredient_imp(new_ing, game, &mut plate, &mut pan);
                pan.reset();
            }
            else {
                pan.is_selected = true;
                plate.is_selected = false;
            }
            return true;
        }

        return false;
    }

    pub fn draw(&self, game: &dyn BaseGame) {
        game.draw_area_background(&self.pos.cur(), &self.cfg.bg);

        self.plate.borrow().draw(game, &self.cfg.plate, &self.cfg.text);
        self.pan.borrow().draw(game, &self.cfg.pan, &self.cfg.text);

        if game.config().draw_borders {
            game.draw_border(self.pos.cur().xpos, self.pos.cur().ypos, 300.0, IngredientStack::height());
        }
    }

    pub fn update_config(&mut self, cfg: &PreparationAreaConfig) {
        self.cfg = cfg.clone();

        self.pos.set_end(Pos2d::new(cfg.xpos, cfg.ypos));
        self.plate.borrow_mut().stack.pos.set_end(Pos2d::new(cfg.plate.xpos, cfg.plate.ypos));
        self.pan.borrow_mut().stack.pos.set_end(Pos2d::new(cfg.pan.xpos, cfg.pan.ypos));
    }
}
