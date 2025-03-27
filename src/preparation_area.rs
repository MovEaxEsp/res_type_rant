
use crate::ingredients::{IngredientStack, MovableIngredient};
use crate::interpolable::{Interpolable, Pos2d};
use crate::traits::{BaseGame, Image, PreparationAreaConfig, PreparationAreaStackConfig, TextConfig};
use crate::utils::WordBank;

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
    fn new(pos: Interpolable<Pos2d>, base_image: &Image, keyword: Rc<String>, cfg: &PreparationAreaStackConfig) -> Self {
        PreparationAreaStack {
            stack: IngredientStack::new(pos),
            cooked_stack: None,
            base_image: *base_image,
            is_selected: false,
            is_cooked: false,
            keyword: keyword,
            end_progress: Interpolable::new(1.0, 1.0/cfg.cook_time),
        }
    }

    fn think(&mut self, game: &dyn BaseGame) {
        self.stack.think(game);

        if self.end_progress.advance(game.elapsed_time()) {
            let cooked_stack = self.cooked_stack.take().unwrap();

            // Replace each ingredient with its cooked version
            for i in 0..self.stack.ingredients.len() {
                self.stack.ingredients[i].image = cooked_stack[i];
            }

            self.is_cooked = true;
        }
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
    fn check_keyword(&mut self, keyword: &str, word_bank:&WordBank) -> bool {
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
    pan: PreparationAreaStack,
    cfg: PreparationAreaConfig,
}

impl PreparationArea {
    pub fn new(word_bank: &WordBank, cfg: &PreparationAreaConfig) -> Self {

        let pos = Interpolable::new(Pos2d::new(cfg.xpos, cfg.ypos), 1000.0);

        let ret = PreparationArea {
            pos: pos.clone(),
            pan: PreparationAreaStack::new(
                    Interpolable::new_b(Pos2d::new(cfg.pan.xpos, cfg.pan.ypos), 1000.0, &pos.clone()),
                    &Image::Pan,
                    word_bank.get_new_word(),
                    &cfg.pan),
            cfg: cfg.clone()
        };

        ret
    }

    pub fn think(&mut self, game: &dyn BaseGame) {
        self.pan.think(game);

        self.pos.advance(game.elapsed_time());
    }

    pub fn handle_command(&mut self, keywords: &Vec<&str>, selected_ings: &mut Vec<MovableIngredient>, game:&dyn BaseGame) -> bool {
        let pan = &mut self.pan;

        for keyword in keywords.iter() {
            if pan.check_keyword(*keyword, game.word_bank()) {
                if pan.is_cooked {
                    let cooked_ing = &pan.stack.ingredients[0];
                    let new_ing = MovableIngredient::new(cooked_ing.image, Interpolable::new(cooked_ing.pos.cur(), 1000.0));
                    selected_ings.push(new_ing);
                    pan.reset();

                    return false;
                }
                else if pan.stack.ingredients.is_empty() {
                    for i in 0..selected_ings.len() {
                        if selected_ings[i].image == Image::RawPatty {
                            pan.stack.add_ingredient(selected_ings.remove(i), false);
                            pan.start_cooking(game);
                        }
                    }
                }

                return true;
            }
        }

        return false;
    }

    pub fn draw(&self, game: &dyn BaseGame) {
        game.draw_area_background(&self.pos.cur(), &self.cfg.bg);

        self.pan.draw(game, &self.cfg.pan, &self.cfg.text);

        if game.config().draw_borders {
            game.draw_border(self.pos.cur().xpos, self.pos.cur().ypos, 300.0, IngredientStack::height());
        }
    }

    pub fn update_config(&mut self, cfg: &PreparationAreaConfig) {
        self.cfg = cfg.clone();

        self.pos.set_end(Pos2d::new(cfg.xpos, cfg.ypos));
        self.pan.stack.pos.set_end(Pos2d::new(cfg.pan.xpos, cfg.pan.ypos));
    }
}
