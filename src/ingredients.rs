
use crate::images::Image;
use crate::traits::{BaseGame, ProgressBarConfig, TextConfig};
use crate::interpolable::{Interpolable, Pos2d};

use std::rc::Rc;

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
    
    // Create a deep clone of 'self' with a new Interpolable at the same position as 'self'
    pub fn deep_clone(&self) -> MovableIngredient {
        MovableIngredient {
            image: self.image,
            pos: Interpolable::new(self.pos.cur(), self.pos.speed()),
            grayed_out: self.grayed_out,
            incoming_ing: Box::new(None)
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
    pub ingredient_arrived: bool,
    pub all_ungrayed: bool,
}

///////// IngredientStack
pub struct IngredientStack {
    pub ingredients: Vec<MovableIngredient>,
    pub pos: Interpolable<Pos2d>,
    pub text: Option<Rc<String>>,
    pub progress: Option<Interpolable<f64>>,
    pub sub_text: Option<Rc<String>>,
    pub overlay: Option<Image>,
}

impl IngredientStack {
    pub fn new(pos: Interpolable<Pos2d>) -> Self {
        IngredientStack {
            ingredients: Vec::new(),
            pos: pos,
            text: None,
            progress: None,
            sub_text: None,
            overlay: None,
        }
    }

    pub fn think(&mut self, game: &dyn BaseGame) -> IngredientStackThinkResult {
        let mut ret = IngredientStackThinkResult {
            progress_done: false,
            pos_done: false,
            ingredient_arrived: false,
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

            if ing_ret.pos_done {
                ret.ingredient_arrived = true;
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

    pub fn draw(&self, game: &dyn BaseGame, progress_cfg: Option<&ProgressBarConfig>, text_cfg: Option<&TextConfig>, subtext_cfg: Option<&TextConfig>) {
        for item in self.ingredients.iter() {
            item.draw(game);
        }

        let mut y_off = 0.0;
        for (progress, cfg) in self.progress.iter().zip(progress_cfg.iter()) {
            if progress.is_moving() {
                let x_off = (self.width(game) - cfg.bg.width)/2.0;
                game.draw_progress_bar(&(self.pos.cur() + (x_off, 0.0).into()), progress.cur(), cfg);
                y_off += cfg.bg.height;
            }
        }

        // Draw the overlay in the bottom-right corner of the first ingredient
        for (ing, overlay) in self.ingredients.iter().zip(self.overlay.iter()) {
            let x_off = game.images().image_width(&ing.image) - (game.images().image_width(overlay)/2.0);
            let y_off = game.images().image_height(&ing.image) - (game.images().image_height(overlay)/2.0);
            let overlay_pos = ing.pos.cur() + (x_off, y_off).into();

            game.draw_image(overlay, &overlay_pos);
        }

        for (text, cfg) in self.text.iter().zip(text_cfg.iter()) {
            game.draw_text(&text, &(self.pos.cur() + (0, y_off).into()), self.width(game), cfg);
            // TODO figure out text height
        }

        for (text, cfg) in self.sub_text.iter().zip(subtext_cfg.iter()) {
            game.draw_text(&text, &(self.pos.cur() + (0, y_off).into()), self.width(game), cfg);
            // TODO figure out text height
        }
    }

    pub fn add_ingredient(&mut self, mut ingredient: MovableIngredient, immediate: bool, game: &dyn BaseGame) {

        let cur_height: f64 = self.ingredients.iter().map(|ing| game.images().image_height(&ing.image)).sum();

        // Account for padding between ingredients
        //cur_height += 10.0 *self.ingredients.len() as f64;

        let end = Pos2d::new(
            0.0,
            -cur_height - game.images().image_height(&ingredient.image)
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

    // Return the width of our stack of ingredients
    pub fn width(&self, game: &dyn BaseGame) -> f64 {
        let mut cur_max: f64 = 0.0;
        let imgs = game.images();
        for ing in self.ingredients.iter() {
            cur_max = cur_max.max(imgs.image_width(&ing.image));
        }

        cur_max
    }
}