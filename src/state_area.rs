
use crate::images::Image;
use crate::ingredients::{IngredientStack, MovableIngredient};
use crate::interpolable::{Interpolable, Pos2d};
use crate::traits::{BackgroundConfig, BaseGame, RingConfig, TextConfig};

use serde::{Serialize,Deserialize};

use std::f64::consts::PI;

#[derive(Serialize, Deserialize, Clone)]
pub struct StateUiConfig {
    pub pos: Pos2d,
    pub bg: BackgroundConfig,

    // clock config
    pub clock_r1: f64,
    pub clock_r2: f64,

    // Open Store config
    pub open_text: TextConfig
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StateGameConfig {
    pub day_length: f64,
}

pub struct StateAreaThinkResult {
    pub enter_store: bool,
    pub enter_game: bool,
}

pub struct StateArea {
    clock_progress: Interpolable<f64>,
    open_store_stack: IngredientStack,
}

impl StateArea {
    pub fn new(cfg_ui: &StateUiConfig, cfg_game: &StateGameConfig, game: &dyn BaseGame) -> Self {
        let clock_progress = Interpolable::new(0.0, 1.0);
        clock_progress.set_end(cfg_game.day_length);

        let mut open_store_stack = IngredientStack::new(Interpolable::new(cfg_ui.pos, 1000.0));
        open_store_stack.text = Some(game.word_bank().get_new_word());
        open_store_stack.add_ingredient(MovableIngredient::new(Image::OpenSign, Interpolable::new((0,0).into(), 1000.0)), true, game);

        StateArea {
            clock_progress: clock_progress,
            open_store_stack: open_store_stack,
        }
    }

    pub fn handle_command(&mut self, keywords: &Vec<String>, game: &dyn BaseGame) {
        if self.in_store() {
            if let Some(kw) = &self.open_store_stack.text {
                if keywords.iter().any(|k| *k == **kw) {
                    // Restart the 'day' timer
                    self.clock_progress.set_cur(0.0);
                    self.open_store_stack.text = Some(game.word_bank().get_new_word());
                }
            }
        }
    }

    pub fn think(&mut self, game: &dyn BaseGame) -> StateAreaThinkResult {
        let ret = StateAreaThinkResult {
            enter_store: self.clock_progress.advance(game.elapsed_time()),
            enter_game: false,
        };

        // TODO remove when these are used elsewhere
        ret.enter_store;
        ret.enter_game;

        ret
    }

    pub fn draw(&self, cfg_ui: &StateUiConfig, cfg_game: &StateGameConfig, game: &dyn BaseGame) {
        if self.in_store() {
            // Draw stack to go back to the game
            self.open_store_stack.draw(game, None, Some(&cfg_ui.open_text), None);
        }
        else {
            // Show the 'clock'
            let mut ring = RingConfig {
                stroke: false,
                style: "yellow".to_string(),
            };
            
            let progress_rad = PI * self.clock_progress.cur()/cfg_game.day_length;
            game.draw_ring(&cfg_ui.pos, cfg_ui.clock_r1, cfg_ui.clock_r2, PI, PI + progress_rad, &ring); 

            ring.stroke = true;
            ring.style = "purple".to_string();
            game.draw_ring(&cfg_ui.pos, cfg_ui.clock_r1, cfg_ui.clock_r2, PI, 0.0, &ring); 
        }

    }

    pub fn in_store(&self) -> bool {
        !self.clock_progress.is_moving()
    }

    pub fn update_config(&mut self, cfg_ui: &StateUiConfig, cfg_game: &StateGameConfig) {
        self.open_store_stack.pos.set_end(cfg_ui.pos);
        self.clock_progress.set_end(cfg_game.day_length);
    }

    pub fn default_ui_config() -> StateUiConfig {
        StateUiConfig {
            pos: (650, 250).into(),
            bg: BackgroundConfig {
                offset: (-50, -70).into(),
                width: 500.0,
                height: 500.0,
                corner_radius: 30.0,
                border_style: "black".to_string(),
                border_alpha: 0.3,
                border_width: 5.0,
                bg_style: "orange".to_string(),
                bg_alpha: 0.2
            },
            clock_r1: 150.0,
            clock_r2: 50.0,
            open_text: TextConfig {
                offset: (0, 0).into(),
                stroke: false,
                style: "yellow".to_string(),
                font: "comic sans".to_string(),
                size: 48,
                center_and_fit: true,
                alpha: 0.4,
                is_command: true,
            },
        }
    }

    pub fn default_game_config() -> StateGameConfig {
        StateGameConfig {
            day_length: 75.0
        }
    }
}