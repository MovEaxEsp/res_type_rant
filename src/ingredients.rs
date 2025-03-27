
use crate::traits::{BaseGame, Image, ProgressBarConfig, TextConfig};
use crate::interpolable::{Interpolable, Pos2d};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

struct MovableIngredientThinkResult {
    pos_done: bool,
    incoming_ing_done: bool,
}

pub struct MovableIngredient {
    pub image: Image,
    pub pos: Interpolable<Pos2d>,
    pub grayed_out: bool,
    pub incoming_ing: Box<Option<MovableIngredient>>,
}

impl MovableIngredient {
    pub fn new(image: Image, pos: Interpolable<Pos2d>) -> Self {
        MovableIngredient {
            image: image,
            pos: pos,
            grayed_out: false,
            incoming_ing: Box::new(None),
        }
    }
    
    fn think(&mut self, game: &dyn BaseGame) -> MovableIngredientThinkResult {
        let mut ret = MovableIngredientThinkResult {
            pos_done: false,
            incoming_ing_done: false,
        };

        if let Some(inc) = &mut *self.incoming_ing {
            if inc.think(game).pos_done {
                self.image = inc.image;
                self.grayed_out = inc.grayed_out;
                ret.incoming_ing_done = true;
            }
        }

        if ret.incoming_ing_done {
            *self.incoming_ing = None;
        }

        ret.pos_done = self.pos.advance(game.elapsed_time());

        ret
    }

    pub fn draw(&self, game: &dyn BaseGame) {
        if self.grayed_out {
            game.draw_gray_image(&self.image, &self.pos.cur());
        }
        else {
            game.draw_image(&self.image, &self.pos.cur());
        }

        if let Some(inc) = &*self.incoming_ing {
            inc.draw(game);
        }
    }

    pub fn set_incoming_ing(&mut self, mut ing: MovableIngredient) {
        ing.pos.rebase(self.pos.base(), self.pos.own_cur(), false);
        *self.incoming_ing = Some(ing); 
    }
}

pub struct IngredientStackThinkResult {
    pub progress_done: bool,
    pub pos_done: bool,
    pub all_ungrayed: bool,
}

///////// IngredientStack
pub struct IngredientStack {
    pub ingredients: Vec<MovableIngredient>,
    pub pos: Interpolable<Pos2d>,
    pub text: Option<String>,
    pub progress: Option<Interpolable<f64>>,
    pub sub_text: Option<String>,
}

impl IngredientStack {
    pub fn new(pos: Interpolable<Pos2d>) -> Self {
        IngredientStack {
            ingredients: Vec::new(),
            pos: pos,
            text: None,
            progress: None,
            sub_text: None,
        }
    }

    pub fn height() -> f64 {
        150.0
    }

    pub fn think(&mut self, game: &dyn BaseGame) -> IngredientStackThinkResult {
        let mut ret = IngredientStackThinkResult {
            progress_done: false,
            pos_done: false,
            all_ungrayed: false
        };

        ret.pos_done = self.pos.advance(game.elapsed_time());

        let mut have_ungrayed = false;
        let mut have_gray = false;
        for item in self.ingredients.iter_mut() {
            let ing_ret = item.think(game);
            if ing_ret.incoming_ing_done {
                have_ungrayed = true;
            }

            if item.grayed_out {
                have_gray = true;
            }
        }

        if have_ungrayed && !have_gray {
            ret.all_ungrayed = true;
        }

        if let Some(progress) = &self.progress {
            ret.progress_done = progress.advance(game.elapsed_time());
        }

        return ret;
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

    pub fn draw_sub_text(&self, game: &dyn BaseGame, cfg: &TextConfig) {
        if let Some(text) = &self.sub_text {
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

        ingredient.pos.rebase(Some(self.pos.clone()), end, immediate);
        self.ingredients.push(ingredient);
    }

    pub fn try_ungray_ingredients(&mut self, ings: &mut Vec<MovableIngredient>) {
        for my_ing in self.ingredients.iter_mut() {
            if !my_ing.grayed_out || my_ing.incoming_ing.is_some() {
                continue;
            }

            for i in 0..ings.len() {
                if my_ing.image == ings[i].image {
                    my_ing.set_incoming_ing(ings.remove(i));
                    break;
                }
            }
        }
    }
}
