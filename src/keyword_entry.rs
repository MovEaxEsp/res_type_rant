use crate::interpolable::{Interpolable, Pos2d};
use crate::painter::{BackgroundConfig, TextConfig};
use crate::traits::BaseGame;

use serde::{Serialize,Deserialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KeywordEntryUiConfig {
    pub pos: Pos2d,
    pub caret_speed: f64,
    pub bg: BackgroundConfig,
    pub text: TextConfig,
}

pub struct KeywordEntry {
    entered_text: String,
    caret_timer: Interpolable<f64>,
}

impl KeywordEntry {
    pub fn new(cfg_ui: &KeywordEntryUiConfig) -> Self {
        let timer = Interpolable::new(0.0, cfg_ui.caret_speed);
        timer.set_end(1.0);

        KeywordEntry {
            entered_text: "|".to_string(),
            caret_timer: timer,
        }
    }

    /// Reset our state for the start of a new day
    pub fn reset_state(&mut self) {
        self.entered_text.clear();
        self.entered_text.push('|');
    }

    pub fn think(&mut self, game: &dyn BaseGame) {
        if self.caret_timer.advance(game.elapsed_time()) {
            if self.caret_timer.cur() == 0.0 {
                self.caret_timer.set_end(1.0);
            }
            else {
                self.caret_timer.set_end(0.0);
            }
        }
    }

    pub fn draw(&self, cfg_ui: &KeywordEntryUiConfig, game: &dyn BaseGame) {
        // Decide whether we want to include the ending '|' in the text.  This serves as our caret.
        let to_draw: &str;
        if self.caret_timer.cur() >= 0.5 {
            to_draw = &self.entered_text;
        }
        else {
            to_draw = &self.entered_text[..self.entered_text.len()-1];
        }

        game.painter().draw_area_background(&cfg_ui.pos, &cfg_ui.bg);
        game.painter().draw_text(to_draw, &cfg_ui.pos, cfg_ui.bg.width, &cfg_ui.text);
    }

    pub fn handle_key(&mut self, key: &str, entered_keywords: &mut Vec<String>) -> bool {
        if key.len() == 1 {
            self.entered_text.pop();
            self.entered_text.push(key.chars().nth(0).unwrap());
            self.entered_text.push('|');
        }
        else if key == "Backspace" {
            if self.entered_text.len() > 1 {
                self.entered_text.pop();
                self.entered_text.pop();
                self.entered_text.push('|');
            }
        }
        else if key != "Enter" {
            log(&format!("Unhandled key: {}", key));
            return false;
        }

        entered_keywords.splice(.., self.entered_text[..self.entered_text.len()-1].split_whitespace().map(String::from));

        if key == "Enter" {
            self.entered_text.clear();
            self.entered_text.push('|');
            return true;
        }

        false
    }

    pub fn update_config(&mut self, cfg_ui: &KeywordEntryUiConfig) {
        self.caret_timer.set_speed(cfg_ui.caret_speed);
    }

    pub fn default_ui_config() -> KeywordEntryUiConfig {
        KeywordEntryUiConfig {
            pos: (20, 1300).into(),
            caret_speed: 3.0,
            bg: BackgroundConfig {
                offset: (-10, -25).into(),
                width: 1000.0,
                height: 100.0,
                corner_radius: 30.0,
                border_style: "black".to_string(),
                border_alpha: 0.3,
                border_width: 5.0,
                bg_style: "white".to_string(),
                bg_alpha: 0.8
            },
            text: TextConfig {
                offset: (0, 0).into(),
                stroke: false,
                style: "black".to_string(),
                font: "comic sans".to_string(),
                size: 48,
                center_and_fit: false,
                alpha: 1.0,
                is_command: false,
            }
        }
    }
}