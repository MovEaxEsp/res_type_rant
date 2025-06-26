
use crate::ingredients::{MovableIngredient, IngredientStack};
use crate::painter::{BackgroundConfig, TextConfig};
use crate::traits::{BaseGame, Image};

use engine_p::interpolable::{Interpolable, Pos2d};
use serde::{Serialize,Deserialize};
use wasm_bindgen::prelude::*;

use std::collections::HashSet;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IngredientAreaUiConfig {
    pub pos: Pos2d,
    pub grid_width: usize,
    pub grid_item_width: f64,
    pub grid_item_height: f64,
    pub bg: BackgroundConfig,
    pub text: TextConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IngredientAreaGameConfig {
    pub ingredients: Vec<Image>,
}

pub struct IngredientArea {
    ingredients: Vec<IngredientStack>,
    pos: Interpolable<Pos2d>,
}

impl IngredientArea {
    pub fn new(game: &dyn BaseGame, cfg_ui: &IngredientAreaUiConfig, cfg_game: &IngredientAreaGameConfig) -> Self {
        
        let mut ret = IngredientArea {
            ingredients: Vec::new(),
            pos: Interpolable::new(cfg_ui.pos, 1000.0),
        };

        ret.update_config(game, cfg_ui, cfg_game);

        ret
    }

    pub fn think(&mut self, game: &dyn BaseGame) {
        self.pos.advance(game.elapsed_time());

        for stack in self.ingredients.iter_mut() {
            stack.think(game);
        }
    }

    pub fn draw(&self, game: &dyn BaseGame, cfg_ui: &IngredientAreaUiConfig) {
        game.painter().draw_area_background(&self.pos.cur(), &cfg_ui.bg);

        for ing in self.ingredients.iter() {
            ing.draw(game, None, Some(&cfg_ui.text), None);
        }
    }

    pub fn handle_command(&mut self, keywords: &Vec<String>, selected_ings: &mut Vec<MovableIngredient>, game:&dyn BaseGame) {
        for keyword in keywords.iter() {
            self.ingredients.iter_mut()
                .filter(|ing| match &ing.text { Some(text) => **text == *keyword, None => false})
                .for_each(|ing| {
                    ing.text = Some(game.word_bank().get_new_word());
                    selected_ings.push(ing.ingredients[0].deep_clone());
                });
        }
    }

    pub fn update_config(&mut self, game: &dyn BaseGame, cfg_ui: &IngredientAreaUiConfig, cfg_game: &IngredientAreaGameConfig) {
        self.pos.set_end(cfg_ui.pos);

        self.ingredients.clear();

        for cfg_ing in cfg_game.ingredients.iter() {
            let num_ings = self.ingredients.len();
            let stack_pos = Pos2d::new(
                cfg_ui.grid_item_width * ((num_ings % cfg_ui.grid_width) as f64),
                cfg_ui.grid_item_height * ((num_ings / cfg_ui.grid_width) as f64)
            );
            
            let mut new_stack = IngredientStack::new(Interpolable::new_b(stack_pos, 1000.0, &self.pos));
            new_stack.add_ingredient(MovableIngredient::new(*cfg_ing, Interpolable::new((0,0).into(), 1000.0)), true, game);
            new_stack.text = Some(game.word_bank().get_new_word());
            self.ingredients.push(new_stack);
        }
    }

    pub fn load_ingredients(&self, ings: &mut HashSet<Image>) {
        for stack in self.ingredients.iter() {
            ings.insert(stack.ingredients[0].image);
        }
    }
}
