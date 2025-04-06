mod images;
mod ingredients;
mod ingredient_area;
mod interpolable;
mod order_bar;
mod preparation_area;
mod traits;
mod utils;

use images::{Image, Images};
use ingredient_area::IngredientArea;
use ingredients::MovableIngredient;
use interpolable::{Pos2d, Interpolable};
use order_bar::OrderBar;
use preparation_area::PreparationArea;
use traits::{BackgroundConfig, BaseGame, GameConfig, MoneyConfig, ProgressBarConfig, TextConfig};
use utils::{set_panic_hook, WordBank};

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, OffscreenCanvas, OffscreenCanvasRenderingContext2d};
use js_sys::JsString;
use core::f64;
use std::cell::RefCell;
use std::rc::Rc;
use web_time::Instant;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

///////// GameState
struct GameImp {
    canvas: OffscreenCanvasRenderingContext2d,
    cur_money: RefCell<i32>,
    images: Images,
    words_bank: WordBank,
    config: GameConfig,
    elapsed_time: f64,  // seconds since previous frame start (for calculating current frame)
    entered_text: String,
    entered_keywords: Vec<String>, // space-split version of entered_text
    keyword_r: Interpolable<f64>,
    keyword_g: Interpolable<f64>,
    keyword_b: Interpolable<f64>,
}

impl BaseGame for GameImp {
    fn set_global_alpha(&self, alpha: f64) {
        self.canvas.set_global_alpha(alpha);
    }

    fn add_money(&self, amt: i32) {
        *self.cur_money.borrow_mut() += amt;
    }

    fn draw_image(&self, image: &Image, pos: &Pos2d) {
        self.images.draw_image(&self.canvas, image, pos.x, pos.y);
    }

    fn draw_gray_image(&self, image: &Image, pos: &Pos2d) {
        self.images.draw_gray_image(&self.canvas, image, pos.x, pos.y);
    }

    fn draw_area_background(&self, pos: &Pos2d, cfg: &BackgroundConfig) {
        let c = &self.canvas;

        c.set_stroke_style_str(&cfg.border_style);
        c.set_fill_style_str(&cfg.bg_style);
        c.set_line_width(cfg.border_width);

        // Draw backgound first
        c.set_global_alpha(cfg.bg_alpha);
        c.begin_path();
        c.round_rect_with_f64(
            pos.x + cfg.offset.x,
            pos.y + cfg.offset.y,
            cfg.width,
            cfg.height,
            cfg.corner_radius).expect("bg");
        c.fill();

        // Draw border
        c.set_global_alpha(cfg.border_alpha);
        c.begin_path();
        c.round_rect_with_f64(
            pos.x + cfg.offset.x,
            pos.y + cfg.offset.y,
            cfg.width,
            cfg.height,
            cfg.corner_radius).expect("border");
        c.stroke();

        c.set_global_alpha(1.0);
    }

    fn draw_progress_bar(&self, pos: &Pos2d, pct: f64, cfg: &ProgressBarConfig) {
        self.draw_area_background(pos, &cfg.bg);

        // Draw the progress indicator
        self.canvas.set_global_alpha(cfg.done_alpha);
        self.canvas.set_fill_style_str(&cfg.done_style);
        self.canvas.begin_path();
        self.canvas.round_rect_with_f64(
            pos.x + cfg.bg.offset.x,
            pos.y + cfg.bg.offset.y,
            cfg.bg.width * pct,
            cfg.bg.height,
            cfg.bg.corner_radius).expect("progress");
        self.canvas.fill();

        self.canvas.set_global_alpha(1.0);
    }

    /*
    fn draw_halo(&self, xpos: f64, ypos: f64, width: f64, height: f64) {
        let middle_x = xpos + width/2.0;
        let middle_y = (ypos + height/2.0) * 2.0;
        let gradient = self.canvas.create_radial_gradient(middle_x, middle_y, 10.0, middle_x, middle_y, width/2.0).unwrap();
        gradient.add_color_stop(0.0, "rgba(255, 255, 255, .5)").unwrap();
        gradient.add_color_stop(1.0, "rgba(255,255,255,0)").unwrap();
        self.canvas.set_fill_style_canvas_gradient(&gradient);
        self.canvas.set_transform(1.0,0.0, 0.0, 0.5, 0.0, 0.0).unwrap();

        self.canvas.begin_path();
        self.canvas.ellipse(middle_x, middle_y, width/2.0, width/2.0, 0.0, 0.0, 2.0*f64::consts::PI).unwrap();
        self.canvas.fill();
        self.canvas.reset_transform().unwrap();
    }
    */

    fn draw_text(&self, text: &String, pos: &Pos2d, width: f64, cfg: &TextConfig) {
        let mut font_size: usize = cfg.size as usize;

        self.canvas.set_global_alpha(cfg.alpha);
        self.canvas.set_fill_style_str(&cfg.style);
        self.canvas.set_stroke_style_str(&cfg.style);
        self.canvas.set_text_baseline("top");

        let mut draw_pos = *pos;
        if cfg.center_and_fit {
            // Figure out where to draw the text, and at what size

            font_size += 1;
            let mut text_width = width + 1.0;
            while text_width > width {
                font_size -= 1;
                self.canvas.set_font(&format!("{}px {}", font_size, cfg.font));
                text_width = self.canvas.measure_text(text).expect("measure text").width();
            }

            // Senter horizontally
            draw_pos = draw_pos + ((width-text_width)/2.0, 0).into();
        }

        draw_pos = draw_pos + cfg.offset;

        let draw_fn: Box<dyn Fn(&str, f64, f64)>;
        if cfg.stroke {
            draw_fn = Box::new(|text, xpos, ypos| {
                self.canvas.stroke_text(text, xpos, ypos).expect("text");
            });
        }
        else {
            draw_fn = Box::new(|text, xpos, ypos| {
                self.canvas.fill_text(text, xpos, ypos).expect("text");
            });
        }
    
        let mut drawn = false;
        if cfg.is_command {
            if self.entered_keywords.iter().find(|x| *x == text).is_some() {
                self.canvas.set_fill_style_str(
                    &format!("rgb({},{},{})", self.keyword_r.cur() as i32, self.keyword_g.cur() as i32, self.keyword_b.cur() as i32));
                self.canvas.set_font(&format!("bold {}px {}", font_size, cfg.font));
                draw_fn(text, draw_pos.x, draw_pos.y);
                drawn = true;
            }
            else if !self.entered_keywords.is_empty() &&
                    text.starts_with(self.entered_keywords.last().unwrap())
            {
                let last_keyword = self.entered_keywords.last().unwrap();
                // Underline the matching part of the word
                self.canvas.set_font(&format!("italic {}px {}", font_size, cfg.font));
                draw_fn(last_keyword, draw_pos.x, draw_pos.y);

                let underlined_width = self.canvas.measure_text(last_keyword).expect("measure text");
                //let new_x = draw_pos.xpos + underlined_width.actual_bounding_box_left() + underlined_width.actual_bounding_box_right();
                let new_x = draw_pos.x + underlined_width.width();

                self.canvas.set_font(&format!("{}px {}", font_size, cfg.font));
                draw_fn(&text[last_keyword.len()..], new_x, draw_pos.y);
                drawn = true;
            }
        }

        if !drawn {
            self.canvas.set_font(&format!("{}px {}", font_size, cfg.font));
            draw_fn(text, draw_pos.x, draw_pos.y);
        }

        self.canvas.set_global_alpha(1.0);
    }

    fn config<'a>(&'a self) -> &'a GameConfig {
        &self.config
    } 

    fn images<'a>(&'a self) -> &'a Images {
        &self.images
    }

    fn word_bank<'a>(&'a self) -> &'a WordBank {
        &self.words_bank
    }

    fn elapsed_time(&self) -> f64 {
        self.elapsed_time
    }
}

impl GameImp {
    fn think(&mut self) {
        let advance_color = |intr: &mut Interpolable<f64>, elapsed_time: f64| {
            intr.advance(elapsed_time);
            if !intr.is_moving() {
                if intr.cur() == 0.0 {
                    intr.set_end(255.0);
                }
                else {
                    intr.set_end(0.0);
                }
            }
        };

        advance_color(&mut self.keyword_r, self.elapsed_time);
        advance_color(&mut self.keyword_g, self.elapsed_time);
        advance_color(&mut self.keyword_b, self.elapsed_time);
    }
}

struct GameState {
    screen_canvas: HtmlCanvasElement,
    offscreen_canvas: OffscreenCanvas,
    order_bar: OrderBar,
    ingredient_area: IngredientArea,
    preparation_area: PreparationArea,
    frame_start: Instant,  // time when previous frame started
    imp: GameImp,
}

impl GameState {
    fn think(&mut self) {
        self.imp.think();
        self.order_bar.think(&self.imp);
        self.ingredient_area.think(&self.imp);
        self.preparation_area.think(&self.imp);
    }

    fn draw(&self) {
        self.imp.canvas.set_fill_style_str("DimGrey");
        self.imp.canvas.clear_rect(0.0, 0.0, 2560.0, 1440.0);
        self.imp.canvas.fill_rect(0.0, 0.0, 2560.0, 1440.0);

        self.order_bar.draw(&self.imp);
        self.ingredient_area.draw(&self.imp);
        self.preparation_area.draw(&self.imp);
    
        // Draw current input
        self.imp.canvas.set_fill_style_str("yellow");
        self.imp.canvas.set_font("48px serif");
        self.imp.canvas.fill_text(&self.imp.entered_text, 20.0, 1300.0).expect("text");

        // Draw current money
        let money_pos = self.imp.config.money.pos;
        self.imp.draw_area_background(&money_pos, &self.imp.config.money.bg);
        self.imp.draw_text(&format!("$ {}", *self.imp.cur_money.borrow()), &money_pos, self.imp.config.money.bg.width, &self.imp.config.money.text);

        let screen_context = self.screen_canvas
        .get_context("2d").unwrap().unwrap()
        .dyn_into::<CanvasRenderingContext2d>().unwrap();

        screen_context.clear_rect(0.0, 0.0, self.screen_canvas.width() as f64, self.screen_canvas.height() as f64);

        screen_context.draw_image_with_offscreen_canvas_and_dw_and_dh(
            &self.offscreen_canvas,
            0.0, 0.0,
            self.screen_canvas.width() as f64, self.screen_canvas.height() as f64)
        .expect("draw offscreen canvas");


    }

    fn handle_command(&mut self) {
        let keywords = self.imp.entered_text.split(' ').collect::<Vec<&str>>();

        let mut selected_ings: Vec<MovableIngredient> = Vec::new();

        self.ingredient_area.handle_command(
            &keywords,
            &mut selected_ings,
            &self.imp);

        let handled = self.preparation_area.handle_command(&keywords, &mut selected_ings, &self.imp);
        if !handled {
            self.order_bar.handle_command(&keywords, &mut selected_ings, &self.imp);
        }
    }

    fn handle_key(&mut self, key: &str, _state_rc: &Rc<RefCell<GameState>>) {
        if key.len() == 1 {
            self.imp.entered_text.push(key.chars().nth(0).unwrap());
        }
        else if key == "Backspace" {
            if self.imp.entered_text.len() > 0 {
                self.imp.entered_text.pop();
            }
        }
        else if key == "Enter" {
            self.handle_command();
            self.imp.entered_text.clear();
        }
        else {
            log(&format!("Unhandled key: {}", key));
        }

        self.imp.entered_keywords = self.imp.entered_text.split_whitespace().map(String::from).collect();
    }

    fn update_config(&mut self, cfg: &GameConfig) {
        self.imp.config = cfg.clone();
        self.imp.images.update_config(&cfg.images);
        self.order_bar.update_config(&cfg.order_bar, &self.imp);
        self.ingredient_area.update_config(&self.imp);
        self.preparation_area.update_config(&self.imp);
    }

}

static mut S_STATE: Option<Rc<RefCell<GameState>>> = None;

#[wasm_bindgen]
pub fn init_state(config: JsValue, canvas: JsValue, images: JsValue, words_db: JsValue, bad_words_db: JsValue) {
    set_panic_hook();
    
    let game_config: GameConfig = serde_wasm_bindgen::from_value(config).unwrap();

    let order_bar = OrderBar::new(&game_config.order_bar);

    let words_bank = WordBank::new(
        &words_db.dyn_into::<JsString>().expect("wordsDb").into(),
        &bad_words_db.dyn_into::<JsString>().expect("badWords").into(),
        game_config.word_level as usize);

    let offscreen_canvas = OffscreenCanvas::new(2560, 1440).expect("offscreen canvas");
    let offscreen_context = offscreen_canvas.get_context("2d").unwrap().unwrap()
                        .dyn_into::<OffscreenCanvasRenderingContext2d>().unwrap();

    let screen_canvas= canvas.dyn_into::<HtmlCanvasElement>().expect("canvas");

    let game_imp = GameImp {
        canvas: offscreen_context,
        images: Images::new(images, &game_config.images),
        cur_money: RefCell::new(0),
        words_bank: words_bank,
        config: game_config,
        elapsed_time: 0.0, 
        entered_text: String::new(),
        entered_keywords: Vec::new(),
        keyword_r: Interpolable::new(72.0, 111.0),
        keyword_g: Interpolable::new(23.0, 79.0),
        keyword_b: Interpolable::new(219.0, 231.0),
    };

    let preparation_area = PreparationArea::new(&game_imp);

    let ingredient_area= IngredientArea::new(&game_imp);

    let state = GameState{
        screen_canvas: screen_canvas,
        offscreen_canvas: offscreen_canvas,
        order_bar: order_bar,
        ingredient_area: ingredient_area,
        preparation_area: preparation_area,
        frame_start: Instant::now(),
        imp: game_imp,
    };

    unsafe {
        S_STATE = Some(Rc::new(RefCell::new(state)));
    }
    
}

fn run_frame_imp(state_rc: &Rc<RefCell<GameState>>) {
    let mut state = state_rc.borrow_mut();

    state.imp.elapsed_time = state.frame_start.elapsed().as_secs_f64();
    state.frame_start = Instant::now();

    state.think();
    state.draw();
}

#[wasm_bindgen]
pub fn run_frame() {
    unsafe {
        #[allow(static_mut_refs)]
        let state: &Rc<RefCell<GameState>> = S_STATE.as_mut().unwrap();
        run_frame_imp(state);
    }
}


#[wasm_bindgen]
pub fn report_keypress(key: &str) {
    unsafe {
        #[allow(static_mut_refs)]
        let state: &Rc<RefCell<GameState>> = S_STATE.as_mut().unwrap();
        state.borrow_mut().handle_key(key, state);
    }
}

#[wasm_bindgen]
pub fn default_config() -> JsValue {
    let cfg = GameConfig {
        word_level: 0,
        images: Images::default_config(),
        order_bar: OrderBar::default_config(),
        ingredient_area: IngredientArea::default_config(),
        preparation_area: PreparationArea::default_config(),
        money: MoneyConfig {
            pos: (50, 50).into(),
            bg: BackgroundConfig {
                offset: (0, -20).into(),
                width: 400.0,
                height: 250.0,
                corner_radius: 30.0,
                border_style: "black".to_string(),
                border_alpha: 0.3,
                border_width: 5.0,
                bg_style: "light green".to_string(),
                bg_alpha: 0.2
            },
            text: TextConfig {
                offset: (40, 40).into(),
                stroke: true,
                style: "gold".to_string(),
                font: "comic sans".to_string(),
                size: 128,
                center_and_fit: false,
                alpha: 1.0,
                is_command: false,
            }
        }
    };

    serde_wasm_bindgen::to_value(&cfg).unwrap()
}

#[wasm_bindgen]
pub fn update_config(config: JsValue) {

    match serde_wasm_bindgen::from_value::<GameConfig>(config) {
        Ok(cfg) => {
            unsafe {
                #[allow(static_mut_refs)]
                let state: &Rc<RefCell<GameState>> = S_STATE.as_mut().unwrap();
                state.borrow_mut().update_config(&cfg);
            }
        }
        Err(e) => {
            log(&format!("Failed parsing config: {}", e));
        }
    }
}