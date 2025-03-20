
use crate::traits::{BaseGame, Image, ProgressBarConfig, TextConfig};
use crate::interpolable::{Interpolable, Pos2d, InterpolableStore};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub struct MovableIngredient {
    pub image: Image,
    pub pos: Interpolable<Pos2d>,
}

impl MovableIngredient {
    pub fn new(image: Image, pos: Interpolable<Pos2d>) -> Self {
        MovableIngredient {
            image: image,
            pos: pos,
        }
    }
    
    pub fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        dest.interpolables_2d.push(self.pos.clone());
    }

    pub fn draw(&self, state: &dyn BaseGame) {
        state.draw_image(&self.image, &self.pos.cur());
    }
}

///////// IngredientStack
pub struct IngredientStack {
    pub ingredients: Vec<MovableIngredient>,
    pub pos: Interpolable<Pos2d>,
    pub text: Option<String>,
    pub progress: Option<Interpolable<f64>>,
}

impl IngredientStack {
    pub fn new(pos: Interpolable<Pos2d>) -> Self {
        IngredientStack {
            ingredients: Vec::new(),
            pos: pos,
            text: None,
            progress: None,
        }
    }

    pub fn height() -> f64 {
        150.0
    }

    pub fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        dest.interpolables_2d.push(self.pos.clone());
        for item in self.ingredients.iter() {
            item.collect_interpolables(dest);
        }

        if let Some(progress) = &self.progress {
            dest.interpolables_1d.push(progress.clone());
        }
    }

    pub fn draw_stack(&self, game: &dyn BaseGame) {
        for item in self.ingredients.iter() {
            item.draw(game);
        }
    }

    pub fn draw_text(&self, game: &dyn BaseGame, cfg: &TextConfig) {
        if let Some(text) = &self.text {
            game.draw_text(&text, &self.pos.cur(), cfg);
        }
    }

    pub fn draw_progress(&self, game: &dyn BaseGame, cfg: &ProgressBarConfig) {
        if let Some(progress) = &self.progress {
            if progress.is_moving() {
                game.draw_progress_bar(&self.pos.cur(), progress.cur(), cfg);
            }
        }
    }

    pub fn draw(&self, game: &dyn BaseGame, text_cfg: &TextConfig, progress_cfg: &ProgressBarConfig) {
        self.draw_stack(game);
        self.draw_text(game, text_cfg);
        self.draw_progress(game, progress_cfg);
    }

    pub fn add_ingredient(&mut self, mut ingredient: MovableIngredient, immediate: bool) {
        let end = Pos2d::new(
            0.0,
            IngredientStack::height() - (((self.ingredients.len()+1) as f64) *35.0)
        );

        ingredient.pos.rebase(&self.pos, end, immediate);
        self.ingredients.push(ingredient);
    }
}
