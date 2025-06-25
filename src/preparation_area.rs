
use crate::images::Image;
use crate::ingredients::{IngredientStack, MovableIngredient};
use crate::painter::{BackgroundConfig, ProgressBarConfig, TextConfig};
use crate::sounds::{PlaybackConfig, Sound};
use crate::traits::BaseGame;

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
pub struct CookingRecipe {
    pub inputs: Vec<Image>,
    pub outputs: Vec<Image>,
    pub cook_time: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CookerConfig {
    pub recipes: Vec<CookingRecipe>,
    pub base_image: Image,
    pub base_offset: Pos2d,
    pub instances: Vec<Pos2d>,
    pub num_unlocked: i32,
    pub cooking_sound: PlaybackConfig,
    pub done_cooking_sound: PlaybackConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PreparationAreaConfig {
    pub pos: Pos2d,
    pub cookers: Vec<CookerConfig>,
    pub bg: BackgroundConfig,
    pub text: TextConfig,
    pub progress: ProgressBarConfig,
}

struct PreparationAreaStack {
    stack: IngredientStack,
    cooked_stack: Option<IngredientStack>,
    is_cooked: bool,
    is_unlocked: bool,
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
            is_unlocked: false,
        }
    }

    fn reset_state(&mut self) {
        self.stack.ingredients.truncate(1);
        self.cooked_stack = None;
        self.is_cooked = false;
        for progress in self.stack.progress.iter_mut() {
            progress.set_cur(0.0);
            progress.set_end(0.0);
        }
    }

    fn think(&mut self, cfg: &CookerConfig, game: &dyn BaseGame) {
        if !self.is_unlocked {
            return;
        }

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

            game.sounds().play_sound(&cfg.done_cooking_sound);
        }

        if ret.ingredient_arrived {
            // Start cooking
            for progress in self.stack.progress.iter() {
                progress.set_end(1.0);

                // Figure out how long to play the sound for
                let mut snd_cfg = cfg.cooking_sound.clone();
                snd_cfg.play_length = Some(1.0/progress.speed());

                game.sounds().play_sound(&snd_cfg);
            }
        }
    }

    fn update_config(&mut self, cfg: &CookerConfig, inst_idx: usize) {
        self.stack.pos.set_end(cfg.instances[inst_idx]);
        self.stack.ingredients[0].image = cfg.base_image;
        self.is_unlocked = cfg.num_unlocked > inst_idx as i32;
    }

    fn draw(&self, game: &dyn BaseGame, text_cfg: &TextConfig, progress_cfg: &ProgressBarConfig) {
        if !self.is_unlocked {
            return;
        }

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

            game.painter().set_global_alpha(1.0 - cur_progress);
            self.stack.draw(game, Some(progress_cfg), draw_text_cfg, None);

            game.painter().set_global_alpha(cur_progress);
            cooked_stack.draw(game, None, None, None);

            game.painter().set_global_alpha(1.0);
        }
        else {
            self.stack.draw(game, Some(progress_cfg), Some(text_cfg), None);
        }
    }

    // Return 'true' if the specified 'keyword' matches our keyword, and replace it with a new
    // one from the specified 'word_bank'.
    fn check_keyword(&mut self, keyword: &String, selected_ings: &mut Vec<MovableIngredient>, cfg: &CookerConfig, game: &dyn BaseGame) -> bool {
        if !self.is_unlocked {
            return false;
        }

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
    pub fn new(game: &dyn BaseGame, cfg: &PreparationAreaConfig) -> Self {
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

    /// Reset our state for the start of a new day
    pub fn reset_state(&mut self) {
        for cook_type in self.cookers.iter_mut() {
            for cook in cook_type.iter_mut() {
                cook.reset_state();
            }
        }
    }

    /// Update our state for the current frame
    pub fn think(&mut self, cfg: &PreparationAreaConfig, game: &dyn BaseGame) {
        for (cooker_type, cfg) in self.cookers.iter_mut().zip(cfg.cookers.iter()) {
            for inst in cooker_type.iter_mut() {
                inst.think(cfg, game);
            }
        }

        self.pos.advance(game.elapsed_time());
    }

    /// Handle the specified 'keywords' being typed by the user.
    pub fn handle_command(&mut self, keywords: &Vec<String>, selected_ings: &mut Vec<MovableIngredient>, game:&dyn BaseGame, cfg: &PreparationAreaConfig) -> bool {
        for (cooker_type, cfg) in self.cookers.iter_mut().zip(cfg.cookers.iter()) {
            for cooker in cooker_type.iter_mut() {
                for keyword in keywords.iter() {
                    if cooker.check_keyword(keyword, selected_ings, cfg, game) {        
                        return true;
                    }
                }        
            }
        }
       
        return false;
    }

    /// Draw ourselves
    pub fn draw(&self, game: &dyn BaseGame, cfg: &PreparationAreaConfig) {
        game.painter().draw_area_background(&self.pos.cur(), &cfg.bg);

        for (cooker_type, _cfg) in self.cookers.iter().zip(cfg.cookers.iter()) {
            for cooker in cooker_type.iter() {
                cooker.draw(game, &cfg.text, &cfg.progress);
            }
        }
    }

    /// Update our config
    pub fn update_config(&mut self, game: &dyn BaseGame, cfg: &PreparationAreaConfig) {
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

    // Figure out which ingredients are possible to produce using our cooking recipes, and append 
    // the corresponding outputs to the specified 'ings'.
    pub fn append_possible_ingredients(&self, ings: &mut HashSet<Image>, cfg: &PreparationAreaConfig) {
        for cooker in cfg.cookers.iter() {
            if cooker.num_unlocked == 0 {
                // Can't cook anything with this cooker
                continue;
            }
            
            for recipe in cooker.recipes.iter() {
                if recipe.inputs.iter().all(|ing| ings.contains(ing)) {
                    recipe.outputs.iter().for_each(|ing| { ings.insert(*ing); } );
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
                    cooking_sound: PlaybackConfig {
                        sound: Sound::Frying,
                        play_length: None, // will be overwritten with the actual length
                        random_start: true,
                    },
                    done_cooking_sound: PlaybackConfig {
                        sound: Sound::Done,
                        play_length: None,
                        random_start: false,
                    },
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
                    instances: vec![ (0, 100).into(), (300, 100).into(), (600, 100).into()],
                    num_unlocked: 0,
                },
                CookerConfig {
                    base_image: Image::TriniPot,
                    base_offset: (0, 10).into(),
                    cooking_sound: PlaybackConfig {
                        sound: Sound::Frying,
                        play_length: None, // will be overwritten with the actual length
                        random_start: true,
                    },
                    done_cooking_sound: PlaybackConfig {
                        sound: Sound::Done,
                        play_length: None,
                        random_start: false,
                    },
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
                    instances: vec![ (0, 550).into(), (300, 550).into(), (600, 550).into()],
                    num_unlocked: 0,
                },
            ],
            bg: BackgroundConfig {
                offset: (-50, -70).into(),
                width: 1300.0,
                height: 700.0,
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