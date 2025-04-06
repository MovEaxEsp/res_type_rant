
use crate::images::Image;
use crate::ingredients::{IngredientStack, MovableIngredient};
use crate::interpolable::{Interpolable, Pos2d};
use crate::traits::{BackgroundConfig, BaseGame, PreparationAreaConfig, ProgressBarConfig, TextConfig, CookerConfig, CookingRecipe};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

struct PreparationAreaStack {
    stack: IngredientStack,
    cooked_stack: Option<IngredientStack>,
    is_cooked: bool,
}

impl PreparationAreaStack {
    fn new(pos: Interpolable<Pos2d>,  cfg: &CookerConfig, game: &dyn BaseGame) -> Self {
        let mut stack = IngredientStack::new(pos);
        stack.add_ingredient(
            MovableIngredient::new(cfg.base_image, Interpolable::new((0,0).into(), 1000.0)),
            true, 
            game);
        stack.text = Some(game.word_bank().get_new_word());
        stack.progress = Some(Interpolable::new(0.0, 1.0));
        
        PreparationAreaStack {
            stack: stack,
            cooked_stack: None,
            is_cooked: false,
        }
    }

    fn think(&mut self, game: &dyn BaseGame) {
        let ret = self.stack.think(game);

        if ret.progress_done {
            let mut cooked_stack = self.cooked_stack.take().unwrap();

            // Replace out stack with the stack of outputs
            std::mem::swap(&mut self.stack.ingredients, &mut cooked_stack.ingredients);

            self.is_cooked = true;
            for progress in self.stack.progress.iter() {
                progress.set_cur(0.0);
                progress.set_end(0.0);
            }
        }

        if ret.ingredient_arrived {
            // Start cooking
            for progress in self.stack.progress.iter() {
                progress.set_end(1.0);
            }
        }
    }

    fn update_config(&mut self, cfg: &CookerConfig, inst_idx: usize) {
        self.stack.pos.set_end(cfg.instances[inst_idx]);
        self.stack.ingredients[0].image = cfg.base_image;
    }

    fn draw(&self, game: &dyn BaseGame, text_cfg: &TextConfig, progress_cfg: &ProgressBarConfig) {
        if let Some(cooked_stack) = &self.cooked_stack {
            // Draw the ingredients transitioning to their cooked versions, if they have one

            let mut cur_progress = 1.0;
            for progress in self.stack.progress.iter() {
                cur_progress = progress.cur();
            }

            // Don't show the text while we're cooking
            let mut draw_text_cfg: Option<&TextConfig> = Some(text_cfg);
            if self.cooked_stack.is_some() {
                draw_text_cfg = None;
            }

            game.set_global_alpha(1.0 - cur_progress);
            self.stack.draw(game, Some(progress_cfg), draw_text_cfg, None);

            game.set_global_alpha(cur_progress);
            cooked_stack.draw(game, None, None, None);

            game.set_global_alpha(1.0);
        }
        else {
            self.stack.draw(game, Some(progress_cfg), Some(text_cfg), None);
        }
    }

    // Return 'true' if the specified 'keyword' matches our keyword, and replace it with a new
    // one from the specified 'word_bank'.
    fn check_keyword(&mut self, keyword: &str, selected_ings: &mut Vec<MovableIngredient>, cfg: &CookerConfig, game: &dyn BaseGame) -> bool {
        for progress in self.stack.progress.iter() {
            if progress.is_moving() {
                // Can't match while we're cooking or already selected
                return false;
            }
        }

        for my_keyword in self.stack.text.iter() {
            if *keyword != **my_keyword {
                return false;
            }
        }

        self.stack.text = Some(game.word_bank().get_new_word());

        if self.is_cooked {
            // Add our cooked ingredients to the selected_ings
            for cooked_ing in self.stack.ingredients.iter() {
                if cooked_ing.image == cfg.base_image {
                    continue;
                }
                let new_ing = MovableIngredient::new(cooked_ing.image, Interpolable::new(cooked_ing.pos.cur(), 1000.0));
                selected_ings.push(new_ing);
            }

            self.is_cooked = false;
            self.stack.ingredients.truncate(1);
            self.cooked_stack = None;

            return false;
        }

        // If our stack has anything besides our base image
        if self.stack.ingredients.len() != 1 {
            return false;
        }

        // Figure out if the selected_ings match any of our recipes
        let mut selected_ing_positions = Vec::new();
        for recipe in cfg.recipes.iter() {
            selected_ing_positions.clear();
            if recipe.inputs.iter()
                .all(|img| selected_ings.iter()
                    .position(|ing| ing.image == *img)
                    .and_then(|pos| {selected_ing_positions.push(pos); Some(pos)})
                    .is_some())
             {
                // All the ingredients of this recipe are in selected_ings.

                // Pull out our raw ings from selected_ings
                // Sort so we can remove by index safely
                selected_ing_positions.sort();
                let mut our_ings: Vec<MovableIngredient> = Vec::new();
                for pos in selected_ing_positions.iter().rev() {
                    our_ings.push(selected_ings.remove(*pos));
                }

                // Sort our_ings so they're in the order defined in the recipe
                our_ings.sort_by_key(|ing| recipe.inputs.iter().position(|input| ing.image == *input));
                for ing in our_ings.into_iter() {
                    self.stack.add_ingredient(ing, false, game);
                }

                // Set up cooked stack and timer.  The timer will start once of all of our ingredients arrive to us
                let mut cooked_stack = IngredientStack::new(self.stack.pos.clone());

                // Add our base to the cooked stack
                cooked_stack.add_ingredient(
                    MovableIngredient::new(self.stack.ingredients[0].image, Interpolable::new((0,0).into(), 1000.0)),
                    true,
                    game);

                // And the rest of the ingredients
                for output in recipe.outputs.iter() {
                    let cooked_ing = MovableIngredient::new(*output, Interpolable::new(Pos2d::new(0.0,0.0), 1000.0));
                    cooked_stack.add_ingredient(cooked_ing, true, game);
                }
                self.cooked_stack = Some(cooked_stack);
                for progress in self.stack.progress.iter() {
                    progress.set_speed(1.0/recipe.cook_time);
                    log(&format!("Cooking progress {:?}", progress));
                }

                // The command should be considered 'handled' and not checked further
                return true;
            }
        }

        return false;
    }
}

pub struct PreparationArea {
    pos: Interpolable<Pos2d>,
    cookers: Vec<Vec<PreparationAreaStack>>,
}

impl PreparationArea {
    pub fn new(game: &dyn BaseGame) -> Self {
        let cfg = &game.config().preparation_area;
        let pos = Interpolable::new(cfg.pos, 1000.0);

        let mut cookers = Vec::new();
        for cooker_cfg in cfg.cookers.iter() {
            let mut instances = Vec::new();
            for inst_pos in cooker_cfg.instances.iter() {
                instances.push(PreparationAreaStack::new(Interpolable::new_b(*inst_pos, 1000.0, &pos), cooker_cfg, game));
            }
            cookers.push(instances);
        }

        PreparationArea {
            pos: pos.clone(),
            cookers: cookers,
        }
    }

    pub fn think(&mut self, game: &dyn BaseGame) {
        for cooker_type in self.cookers.iter_mut() {
            for inst in cooker_type.iter_mut() {
                inst.think(game);
            }
        }

        self.pos.advance(game.elapsed_time());
    }

    pub fn handle_command(&mut self, keywords: &Vec<&str>, selected_ings: &mut Vec<MovableIngredient>, game:&dyn BaseGame) -> bool {
        for (cooker_type, cfg) in self.cookers.iter_mut().zip(game.config().preparation_area.cookers.iter()) {
            for cooker in cooker_type.iter_mut() {
                for keyword in keywords.iter() {
                    if cooker.check_keyword(*keyword, selected_ings, cfg, game) {        
                        return true;
                    }
                }        
            }
        }
       
        return false;
    }

    pub fn draw(&self, game: &dyn BaseGame) {
        game.draw_area_background(&self.pos.cur(), &game.config().preparation_area.bg);

        for (cooker_type, _cfg) in self.cookers.iter().zip(game.config().preparation_area.cookers.iter()) {
            for cooker in cooker_type.iter() {
                cooker.draw(game, &game.config().preparation_area.text, &game.config().preparation_area.progress);
            }
        }
    }

    pub fn update_config(&mut self, game: &dyn BaseGame) {
        let cfg = &game.config().preparation_area;

        self.pos.set_end(cfg.pos);

        // Remove excess cookers
        if cfg.cookers.len() > self.cookers.len() {
            self.cookers.drain(cfg.cookers.len()..self.cookers.len());
        }

        for cooker_idx in 0..cfg.cookers.len() {
            let cooker_cfg = &cfg.cookers[cooker_idx];
            if cooker_idx >= self.cookers.len() {
                // Need new cooker
                self.cookers.push(Vec::new());
            }

            let cooker_vec = &mut self.cookers[cooker_idx];
            if cooker_vec.len() > cooker_cfg.instances.len() {
                cooker_vec.drain(cooker_cfg.instances.len()..cooker_vec.len());
            }

            for inst_idx in 0..cooker_cfg.instances.len() {
                if inst_idx >= cooker_vec.len() {
                    // Need new instance
                    cooker_vec.push(PreparationAreaStack::new(
                        Interpolable::new_b(cooker_cfg.instances[inst_idx], 1000.0, &self.pos),
                        cooker_cfg,
                        game));
                }
                else {
                    // Update the stack
                    cooker_vec[inst_idx].update_config(cooker_cfg, inst_idx);
                }
            }
        }
    }

    pub fn default_config() -> PreparationAreaConfig {
        PreparationAreaConfig {
            pos: (1200, 800).into(),
            cookers: vec![
                CookerConfig {
                    base_image: Image::Pan,
                    base_offset: (-10, 10).into(),
                    recipes: vec![
                        CookingRecipe {
                            inputs: vec![Image::RawPatty],
                            outputs: vec![Image::CookedPatty],
                            cook_time: 10.0,
                        },
                        CookingRecipe {
                            inputs: vec![Image::EggsRaw],
                            outputs: vec![Image::EggsFried],
                            cook_time: 6.0,
                        },
                        CookingRecipe {
                            inputs: vec![Image::BaconRaw],
                            outputs: vec![Image::BaconCooked],
                            cook_time: 8.0,
                        },
                    ],
                    instances: vec![ (0, 300).into(), (300, 300).into()]
                },
                CookerConfig {
                    base_image: Image::TriniPot,
                    base_offset: (0, 10).into(),
                    recipes: vec![
                        CookingRecipe {
                            inputs: vec![Image::RawCrab, Image::Curry],
                            outputs: vec![Image::CurryCrab],
                            cook_time: 15.0,
                        },
                        CookingRecipe {
                            inputs: vec![Image::Flour],
                            outputs: vec![Image::Dumplings],
                            cook_time: 5.0,
                        }
                    ],
                    instances: vec![ (600, 300).into(), (900, 300).into()]
                },
            ],
            bg: BackgroundConfig {
                offset: (-50, -70).into(),
                width: 1300.0,
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
            },
            progress: ProgressBarConfig {
                bg: BackgroundConfig {
                    offset: (0, 30).into(),
                    width: 100.0,
                    height: 5.0,
                    corner_radius: 0.0,
                    border_style: "black".to_string(),
                    border_alpha: 0.0,
                    border_width: 0.0,
                    bg_style: "black".to_string(),
                    bg_alpha: 0.4
                },
                done_alpha: 1.0,
                done_style: "yellow".to_string(),
            },
        }
    }
}