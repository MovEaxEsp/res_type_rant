
use crate::ingredients::MovableIngredient;
use crate::interpolable::{Interpolable, Pos2d};
use crate::traits::{BaseGame, Image, IngredientAreaConfig};
use crate::utils::WordBank;

use std::rc::Rc;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub struct IngredientArea {
    ingredients: Vec<Image>,
    ingredient_words: Vec<Rc<String>>,
    pos: Interpolable<Pos2d>,
    cfg: IngredientAreaConfig,
}

impl IngredientArea {
    pub fn new(ingredients: Vec<Image>, word_bank: &WordBank, cfg: &IngredientAreaConfig) -> Self {
        let ingredient_words:Vec<Rc<String>> = (0..ingredients.len()).into_iter().map(|_idx| word_bank.get_new_word()).collect();

        IngredientArea {
            ingredients: ingredients,
            ingredient_words: ingredient_words,
            pos: Interpolable::new(Pos2d::new(cfg.xpos, cfg.ypos), 1000.0),
            cfg: cfg.clone(),
        }
    }

    pub fn think(&mut self, game: &dyn BaseGame) {
        self.pos.advance(game.elapsed_time());
    }

    fn grid_item_pos(&self, i: usize) -> Pos2d {
        Pos2d::new(
            self.pos.cur().xpos + self.cfg.grid_item_width * ((i % self.cfg.grid_width) as f64),
            self.pos.cur().ypos + self.cfg.grid_item_height * ((i/self.cfg.grid_width) as f64)
        )
    }

    pub fn draw(&self, game: &dyn BaseGame) {
        game.draw_area_background(&self.pos.cur(), &self.cfg.bg);

        for i in 0..self.ingredients.len() {
            let img_pos = self.grid_item_pos(i);

            game.draw_image(&self.ingredients[i], &img_pos);

            game.draw_text(&self.ingredient_words[i], &img_pos, &self.cfg.text);
        }

        if game.config().draw_borders {
            game.draw_border(self.pos.cur().xpos, self.pos.cur().ypos, (120*6) as f64, (80*3) as f64);
        }
    }

    pub fn handle_command(&mut self, keywords: &Vec<&str>, selected_ings: &mut Vec<MovableIngredient>, game:&dyn BaseGame) {
        for keyword in keywords.iter() {
            for i in 0..self.ingredients.len() {
                let ing_word: &String = &self.ingredient_words[i];
                if ing_word == keyword {
                    self.ingredient_words[i] = game.word_bank().get_new_word();

                    let ing_pos = Interpolable::new_b(
                        Pos2d::new(120.0 * ((i%6) as f64), 80.0 * ((i/6) as f64)),
                        1000.0,
                        &self.pos);

                    selected_ings.push(MovableIngredient::new(
                            self.ingredients[i],
                            ing_pos));
                }
            }
        }
    }

    pub fn update_config(&mut self, cfg: &IngredientAreaConfig) {
        self.cfg = cfg.clone();

        self.pos.set_end(Pos2d::new(cfg.xpos, cfg.ypos));
    }
}
