
use crate::images::Image;
use crate::ingredients::{MovableIngredient, IngredientStack};
use crate::interpolable::{Interpolable, Pos2d};
use crate::traits::{BackgroundConfig, BaseGame, TextConfig};

use serde::{Serialize,Deserialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IngredientAreaConfig {
    pub pos: Pos2d,
    pub grid_width: usize,
    pub grid_item_width: f64,
    pub grid_item_height: f64,
    pub ingredients: Vec<Image>,
    pub bg: BackgroundConfig,
    pub text: TextConfig,
}

pub struct IngredientArea {
    ingredients: Vec<IngredientStack>,
    pos: Interpolable<Pos2d>,
}

impl IngredientArea {
    pub fn new(game: &dyn BaseGame, cfg: &IngredientAreaConfig) -> Self {
        
        let mut ret = IngredientArea {
            ingredients: Vec::new(),
            pos: Interpolable::new(cfg.pos, 1000.0),
        };

        ret.update_config(game, cfg);

        ret
    }

    pub fn think(&mut self, game: &dyn BaseGame) {
        self.pos.advance(game.elapsed_time());

        for stack in self.ingredients.iter_mut() {
            stack.think(game);
        }
    }

    pub fn draw(&self, game: &dyn BaseGame, cfg: &IngredientAreaConfig) {
        game.draw_area_background(&self.pos.cur(), &cfg.bg);

        for ing in self.ingredients.iter() {
            ing.draw(game, None, Some(&cfg.text), None);
        }
    }

    pub fn handle_command(&mut self, keywords: &Vec<&str>, selected_ings: &mut Vec<MovableIngredient>, game:&dyn BaseGame) {
        for keyword in keywords.iter() {
            self.ingredients.iter_mut()
                .filter(|ing| match &ing.text { Some(text) => **text == *keyword, None => false})
                .for_each(|ing| {
                    ing.text = Some(game.word_bank().get_new_word());
                    log(&format!("Selected ing: {:?}", ing.ingredients[0].image));
                    selected_ings.push(ing.ingredients[0].deep_clone());
                });
        }
    }

    pub fn update_config(&mut self, game: &dyn BaseGame, cfg: &IngredientAreaConfig) {
        self.pos.set_end(cfg.pos);

        self.ingredients.clear();

        for cfg_ing in cfg.ingredients.iter() {
            let num_ings = self.ingredients.len();
            let stack_pos = Pos2d::new(
                cfg.grid_item_width * ((num_ings % cfg.grid_width) as f64),
                cfg.grid_item_height * ((num_ings / cfg.grid_width) as f64)
            );
            
            let mut new_stack = IngredientStack::new(Interpolable::new_b(stack_pos, 1000.0, &self.pos));
            new_stack.add_ingredient(MovableIngredient::new(*cfg_ing, Interpolable::new((0,0).into(), 1000.0)), true, game);
            new_stack.text = Some(game.word_bank().get_new_word());
            self.ingredients.push(new_stack);
        }
    }

    pub fn default_config() -> IngredientAreaConfig {
        IngredientAreaConfig {
            pos: (80, 800).into(),
            grid_width: 5,
            grid_item_width: 170.0,
            grid_item_height: 200.0,
            ingredients: vec![
                Image::BurgerBottom, Image::BurgerTop, Image::LettuceLeaf,
                Image::TomatoSlice, Image::Flour, Image::Curry,
                Image::RawPatty, Image::RawCrab, Image::BaconRaw, Image::EggsRaw,
            ],
            bg: BackgroundConfig {
                offset: (-50, -150).into(),
                width: 900.0,
                height: 500.0,
                corner_radius: 30.0,
                border_style: "black".to_string(),
                border_alpha: 0.3,
                border_width: 5.0,
                bg_style: "orange".to_string(),
                bg_alpha: 0.2
            },
            text: TextConfig {
                offset: (0, 0).into(),
                stroke: false,
                style: "yellow".to_string(),
                font: "comic sans".to_string(),
                size: 48,
                center_and_fit: true,
                alpha: 0.4,
                is_command: true,
            }
        }
    }
}
