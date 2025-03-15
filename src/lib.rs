mod ingredients;
mod ingredient_area;
mod interpolable;
mod order_bar;
mod preparation_area;
mod traits;
mod utils;

use ingredient_area::IngredientArea;
use interpolable::InterpolableStore;
use order_bar::OrderBar;
use preparation_area::PreparationArea;
use traits::{Image, ImageProps, BaseGame, GameConfig};
use utils::{set_panic_hook, WordBank};

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, OffscreenCanvas, OffscreenCanvasRenderingContext2d};
use js_sys::JsString;
use core::f64;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use web_time::Instant;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

///////// GameState

struct GameState {
    screen_canvas: HtmlCanvasElement,
    offscreen_canvas: OffscreenCanvas,
    canvas: OffscreenCanvasRenderingContext2d,
    images: HashMap<Image, ImageProps>,
    order_bar: Rc<RefCell<OrderBar>>,
    ingredient_area: IngredientArea,
    preparation_area: Rc<RefCell<PreparationArea>>,
    frame_start: Instant,  // time when previous frame started
    elapsed_time: f64,  // seconds since previous frame start (for calculating current frame)
    entered_text: String,
    words_bank: WordBank,
    config: GameConfig,
}

impl BaseGame for GameState {
    fn draw_image(&self, image: &Image, xpos: f64, ypos: f64) {

        let props = self.images.get(image).unwrap();

        self.canvas.draw_image_with_html_image_element_and_dw_and_dh(
            &props.image,
            xpos,
            ypos,
            props.width,
            props.height,
        )
        .expect("draw");
    }

    fn draw_border(&self, xpos: f64, ypos: f64, width: f64, height: f64) {
        self.canvas.set_stroke_style_str("red");
        self.canvas.begin_path();
        self.canvas.rect(xpos, ypos,width, height);
        self.canvas.close_path();
        self.canvas.stroke();
    }

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

    fn draw_command_text(&self, xpos: f64, ypos: f64, text: &String) {
        self.canvas.set_stroke_style_str("yellow");

        let mut font_size = 48;
        if text.len() > 3 {
            font_size -= (text.len() - 3) * 3;
        }

        self.canvas.set_font(&format!("{}px serif", font_size));
        self.canvas.stroke_text(&text, xpos, ypos).expect("text");
    }

    fn config<'a>(&'a self) -> &'a GameConfig {
        &self.config
    } 

    fn image_props<'a>(&'a self, image: &Image) -> &'a ImageProps {
        &self.images[image]
    }
}

impl GameState {
    fn draw(&self) {

        self.canvas.rect(0.0, 0.0, 2560.0, 1440.0);
        self.canvas.set_fill_style_str("DimGrey");
        self.canvas.fill();

        //self.canvas.set_image_smoothing_enabled(false);

        self.order_bar.borrow().draw(self);
        self.ingredient_area.draw(self);
        self.preparation_area.borrow().draw(self);
    
        // Draw current input
        self.canvas.set_fill_style_str("yellow");
        self.canvas.set_font("48px serif");
        self.canvas.fill_text(&self.entered_text, 20.0, 1300.0).expect("text");

        let screen_context = self.screen_canvas
                .get_context("2d").unwrap().unwrap()
                .dyn_into::<CanvasRenderingContext2d>().unwrap();

        //screen_context.set_image_smoothing_enabled(false);
        screen_context.draw_image_with_offscreen_canvas_and_dw_and_dh(
            &self.offscreen_canvas,
            0.0, 0.0,
            self.screen_canvas.width() as f64, self.screen_canvas.height() as f64)
        .expect("draw offscreen canvas");
    }

    fn handle_command(&mut self) {

        let handled = self.ingredient_area.handle_command(
            &self.entered_text,
            &mut self.preparation_area.borrow_mut(),
            &self.words_bank);
        if !handled {
            self.preparation_area.borrow_mut().handle_command(&self.entered_text, &self.order_bar, &self.words_bank);
        }
    }

    fn handle_key(&mut self, key: &str, _state_rc: &Rc<RefCell<GameState>>) {
        if key.len() == 1 {
            self.entered_text.push(key.chars().nth(0).unwrap());
        }
        else if key == "Backspace" {
            if self.entered_text.len() > 0 {
                self.entered_text.pop();
            }
        }
        else if key == "Enter" {
            self.handle_command();
            self.entered_text.clear();
        }
        else {
            log(&format!("Unhandled key: {}", key));
        }
    }
}

static mut S_STATE: Option<Rc<RefCell<GameState>>> = None;

#[wasm_bindgen]
pub fn init_state(config: JsValue, canvas: JsValue, images: JsValue, words_db: JsValue, bad_words_db: JsValue) {

    set_panic_hook();

    let mut image_map: HashMap<Image, HtmlImageElement> = HashMap::new();

    let image_names = HashMap::from([
        (Image::Plate, "plate.png"),
        (Image::Pan, "pan.png"),
        (Image::BurgerTop, "burger_top.png"),
        (Image::BurgerBottom, "burger_bottom.png"),
        (Image::LettuceLeaf, "lettuce_leaf.png"),
        (Image::TomatoSlice, "tomato_slice.png"),
        (Image::RawPatty, "raw_patty.png"),
        (Image::CookedPatty, "cooked_patty.png"),
    ]);

    for (imgtype, imgname) in image_names {
        let imgjs = js_sys::Reflect::get(&images, &imgname.into()).expect(imgname);
        let htmlimg = imgjs.dyn_into::<HtmlImageElement>().expect(imgname);
        image_map.insert(imgtype, htmlimg);
    }

    let mut image_def = |image: Image, width: f64, height: f64, cooked_image: Option<Image>| {
        (image, ImageProps{image: image_map.remove(&image).unwrap(), width: width, height: height, cooked_image: cooked_image})
    };

    let state_images = HashMap::from([
        image_def(Image::Plate, 100.0, 30.0, None),
        image_def(Image::Pan, 200.0, 30.0, None),
        image_def(Image::BurgerTop, 100.0, 30.0, None),
        image_def(Image::BurgerBottom, 100.0, 30.0, None),
        image_def(Image::LettuceLeaf, 100.0, 30.0, None),
        image_def(Image::TomatoSlice, 100.0, 30.0, None),
        image_def(Image::RawPatty, 100.0, 30.0, Some(Image::CookedPatty)),
        image_def(Image::CookedPatty, 100.0, 30.0, None),
    ]);

    let order_bar = OrderBar::new(10.0, 20.0);

    let game_config: GameConfig = serde_wasm_bindgen::from_value(config).unwrap();

    let words_bank = WordBank::new(
        &words_db.dyn_into::<JsString>().expect("wordsDb").into(),
        &bad_words_db.dyn_into::<JsString>().expect("badWords").into(),
        game_config.word_level as usize);

        
    let ingredient_area= IngredientArea::new(
        vec![Image::BurgerBottom, Image::BurgerTop, Image::RawPatty, Image::LettuceLeaf, Image::TomatoSlice],
        30.0,
        900.0,
        &words_bank);

    let offscreen_canvas = OffscreenCanvas::new(2560, 1440).expect("offscreen canvas");
    let offscreen_context = offscreen_canvas.get_context("2d").unwrap().unwrap()
                        .dyn_into::<OffscreenCanvasRenderingContext2d>().unwrap();

    let screen_canvas= canvas.dyn_into::<HtmlCanvasElement>().expect("canvas");

    let preparation_area = PreparationArea::new(30.0, 300.0, &words_bank);

    let state = GameState{
        screen_canvas: screen_canvas,
        offscreen_canvas: offscreen_canvas,
        canvas: offscreen_context,
        images: state_images,
        order_bar: order_bar,
        ingredient_area: ingredient_area,
        preparation_area: preparation_area,
        frame_start: Instant::now(),
        elapsed_time: 0.0,
        entered_text: String::new(),
        words_bank: words_bank,
        config: game_config,

    };

    unsafe {
        S_STATE = Some(Rc::new(RefCell::new(state)));
    }
    
}

fn run_frame_imp(state_rc: &Rc<RefCell<GameState>>) {
    let mut state = state_rc.borrow_mut();

    state.elapsed_time = state.frame_start.elapsed().as_secs_f64();
    state.frame_start = Instant::now();

    // Let every entitity think
    //for i in 0..state.entities.len() {
    //    let entity = state.entities[i].as_ref();
        //entity.think(state);
    //}

    // Advance all interpolables
    let mut interpolables = InterpolableStore::new();
    {
        let order_bar = state.order_bar.borrow();
        order_bar.collect_interpolables(&mut interpolables);
        state.preparation_area.borrow().collect_interpolables(&mut interpolables);
    }

    interpolables.advance_all(state.elapsed_time);

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