
use crate::ingredients::MovableIngredient;
use crate::interpolable::Pos2d;
use crate::preparation_area::PreparationArea;
use crate::traits::{BaseGame, Image};
use crate::utils::WordBank;

use std::rc::Rc;

pub struct IngredientArea {
    ingredients: Vec<Image>,
    ingredient_words: Vec<Rc<String>>,
    pos: Pos2d,
}

impl IngredientArea {
    pub fn new(ingredients: Vec<Image>, xpos: f64, ypos: f64, word_bank: &WordBank) -> Self {
        let ingredient_words:Vec<Rc<String>> = (0..ingredients.len()).into_iter().map(|_idx| word_bank.get_new_word()).collect();

        IngredientArea {
            ingredients: ingredients,
            ingredient_words: ingredient_words,
            pos: Pos2d { xpos: xpos, ypos: ypos },
        }
    }

    pub fn draw(&self, game: &dyn BaseGame) {
        // Draw the ingredients in a 6x3 grid

        for i in 0..self.ingredients.len() {
            let xpos = self.pos.xpos + (120.0 * ((i % 6) as f64));
            let ypos = self.pos.ypos + (80.0 * ((i/6) as f64));

            game.draw_image(&self.ingredients[i], xpos, ypos);

            game.draw_command_text(xpos, ypos + 80.0, &self.ingredient_words[i]);
        }

        if game.config().draw_borders {
            game.draw_border(self.pos.xpos, self.pos.ypos, (120*6) as f64, (80*3) as f64);
        }
    }

    pub fn handle_command(&mut self, keyword: &String, prep: &mut PreparationArea, word_bank: &WordBank) -> bool{
        for i in 0..self.ingredients.len() {
            let ing_word: &String = &self.ingredient_words[i];
            if ing_word == keyword {
                self.ingredient_words[i] = word_bank.get_new_word();
                prep.send_ingredient(
                    MovableIngredient::new(
                        self.ingredients[i],
                        120.0 * ((i%6) as f64),
                        80.0 * ((i/6) as f64),
                        500.0,
                    ),
                    &self.pos,
                    word_bank);
                return true;
            }
        }

        return false;
    }
}
