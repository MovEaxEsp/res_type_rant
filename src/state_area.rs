
use crate::ingredients::{IngredientStack, MovableIngredient};
use crate::painter::{BackgroundConfig, ProgressBarConfig, RingConfig, TextConfig};
use crate::traits::{BaseGame, Image};

use engine_p::interpolable::{Interpolable, Pos2d};
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
    pub text: TextConfig,
    pub progress: ProgressBarConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StateGameConfig {
    pub day_length: f64,
    pub money_down_sec: f64,
    pub money_down_amt: i32,
}

#[derive(PartialEq)]
enum StoreState {
    Open,
    Closing,
    Closed,
}

pub struct StateArea {
    clock_progress: Interpolable<f64>,
    open_close_store_stack: IngredientStack,
    state: StoreState,
}

impl StateArea {
    pub fn new(cfg_ui: &StateUiConfig, cfg_game: &StateGameConfig, game: &dyn BaseGame) -> Self {
        let clock_progress = Interpolable::new(0.0, 1.0);
        clock_progress.set_end(cfg_game.day_length);

        let mut store_stack = IngredientStack::new(Interpolable::new(cfg_ui.pos, 1000.0));
        store_stack.text = Some(game.word_bank().get_new_word());
        store_stack.add_ingredient(MovableIngredient::new(Image::OpenSign, Interpolable::new((0,0).into(), 1000.0)), true, game);

        StateArea {
            clock_progress: clock_progress,
            open_close_store_stack: store_stack,
            state: StoreState::Open,
        }
    }

    pub fn handle_command(&mut self, keywords: &Vec<String>, game: &dyn BaseGame) {
        if self.state == StoreState::Closing {
            if let Some(kw) = &self.open_close_store_stack.text {
                if keywords.iter().any(|k| *k == **kw) {
                    // Become 'closed' and show the store
                    self.open_close_store_stack.text = Some(game.word_bank().get_new_word());
                    self.open_close_store_stack.progress = None;
                    self.state = StoreState::Closed;
                }
            }
        }
        else if self.state == StoreState::Closed {
            if let Some(kw) = &self.open_close_store_stack.text {
                if keywords.iter().any(|k| *k == **kw) {
                    // Restart the 'day' timer
                    self.clock_progress.set_cur(0.0);
                    self.open_close_store_stack.text = Some(game.word_bank().get_new_word());
                    self.state = StoreState::Open;
                }
            }
        }
    }

    pub fn think(&mut self, cfg_game: &StateGameConfig, game: &dyn BaseGame) {
        if self.state == StoreState::Open {
            // Advance the day time
            if self.clock_progress.advance(game.elapsed_time()) {
                // Move to 'closing'
                self.state = StoreState::Closing;
                
                let money_state = Interpolable::new(0.0, 1.0/cfg_game.money_down_sec);
                money_state.set_end(1.0);

                self.open_close_store_stack.progress = Some(money_state);
            }
        }
        else if self.state == StoreState::Closing {
            // Advance the 'money down' timer
            if  let Some(money_progress) = &self.open_close_store_stack.progress {
                if money_progress.advance(game.elapsed_time()) {
                    game.add_money(cfg_game.money_down_amt);
                    money_progress.set_cur(0.0);
                }
            }
        }
    }

    pub fn draw(&self, cfg_ui: &StateUiConfig, cfg_game: &StateGameConfig, game: &dyn BaseGame) {
        if self.state == StoreState::Open {
            // Show the 'clock'
            let mut ring = RingConfig {
                stroke: false,
                style: "yellow".to_string(),
            };
            
            let progress_rad = PI * self.clock_progress.cur()/cfg_game.day_length;
            game.painter().draw_ring(&cfg_ui.pos, cfg_ui.clock_r1, cfg_ui.clock_r2, PI, PI + progress_rad, &ring); 

            ring.stroke = true;
            ring.style = "purple".to_string();
            game.painter().draw_ring(&cfg_ui.pos, cfg_ui.clock_r1, cfg_ui.clock_r2, PI, 0.0, &ring); 
        }
        else {
            // Draw stack to show open/closed sign
            self.open_close_store_stack.draw(game, Some(&cfg_ui.progress), Some(&cfg_ui.text), None);
        }
    }

    pub fn in_store(&self) -> bool {
        self.state == StoreState::Closed
    }

    pub fn update_config(&mut self, cfg_ui: &StateUiConfig, cfg_game: &StateGameConfig) {
        self.open_close_store_stack.pos.set_end(cfg_ui.pos);
        self.clock_progress.set_end(cfg_game.day_length);
    }
}