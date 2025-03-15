mod utils;
use utils::{set_panic_hook, WordBank};

mod interpolable;
use interpolable::{Pos2d, Interpolable, InterpolableStore};

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, OffscreenCanvas, OffscreenCanvasRenderingContext2d};
use js_sys::JsString;
use core::f64;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use web_time::Instant;
use serde::{Serialize,Deserialize};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

struct ImageProps {
    image: HtmlImageElement,
    cooked_image: Option<Image>,
    width: f64,
    height: f64,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
enum Image {
    Plate,
    Pan,
    BurgerTop,
    BurgerBottom,
    LettuceLeaf,
    TomatoSlice,
    RawPatty,
    CookedPatty,
}

// An object that can be drawn
trait Entity {
    fn draw(&self, _game: &GameState) {}
}

//////////// Interpolable



#[derive(Serialize, Deserialize)]
struct GameConfig {
    word_level: i32,
    draw_borders: bool,
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

impl GameState {
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

///////////// OrderBar

// Manager for the list of orders at the top of the screen

struct OrderBar {
    orders: Vec<IngredientStack>,
    pos: Pos2d,
    new_item_timer: Interpolable<f64>,
    submitted_order: Option<IngredientStack>,
}

impl Entity for OrderBar {
    fn draw(&self, state: &GameState) {
        for i in 0..self.orders.len() {
            self.orders[i].draw(state);
        }

        for submitted in self.submitted_order.iter() {
            submitted.draw(state);
        }

        if state.config.draw_borders {
            state.draw_border(self.pos.xpos, self.pos.ypos, 600.0, IngredientStack::height());
        }
    }
}

impl OrderBar {
    fn new(xpos: f64, ypos: f64) -> Rc<RefCell<Self>> {
        let new_item_timer = Interpolable::new(0.0, 1.0);
        new_item_timer.set_end(3.0);

        let ret= Rc::new(RefCell::new(OrderBar {
            orders: Vec::new(),
            pos: Pos2d{xpos:xpos, ypos:ypos},
            new_item_timer: Interpolable::new(0.0, 1.0),
            submitted_order: None,
        }));

        let cb_ret: Rc<RefCell<OrderBar>> = ret.clone();

        ret.borrow().new_item_timer.set_moved_handler(Box::new(move || {
            let inner_ret = cb_ret.clone();
            inner_ret.borrow_mut().create_order();
        }));
        ret.borrow_mut().new_item_timer.set_end(2.0);

        ret
    }

    fn try_submit_order(&mut self, order: IngredientStack, self_rc: Rc<RefCell<Self>>) -> bool{
        for i in 0..self.orders.len() {
            let bar_order = &self.orders[i];
            if order.ingredients.len() != bar_order.ingredients.len() {
                continue;
            }

            let mut is_match = true;
            for ing_idx in 0..order.ingredients.len() {
                if order.ingredients[ing_idx].image != bar_order.ingredients[ing_idx].image {
                    is_match = false;
                    break; 
                }
            }

            if !is_match {
                continue;
            }

            // found a matching order
            order.pos.set_end(bar_order.pos.cur());

            let local_self_rc = self_rc.clone();

            order.pos.set_moved_handler(Box::new(move || {
                let next_local_self_rc =local_self_rc.clone();
                let mut order_bar = local_self_rc.borrow_mut();
                order_bar.serve_order(i, next_local_self_rc);
                order_bar.submitted_order = None;
            }));

            self.submitted_order = Some(order);
            return true;
        }

        return false;
    }

    fn serve_order(&mut self, order_idx: usize, self_rc: Rc<RefCell<Self>>) {
        let order = &self.orders[order_idx];
        let cur_pos = order.pos.cur();
        order.pos.set_end(Pos2d{xpos:cur_pos.xpos, ypos: -100.0});

        for i in (order_idx+1)..self.orders.len() {
            self.orders[i].pos.set_end(self.orders[i-1].pos.cur());
        }

        let local_self_rc = self_rc.clone();

        order.pos.set_moved_handler(Box::new(move || {
            let mut order_bar = local_self_rc.borrow_mut();

            order_bar.orders.remove(order_idx);
        }));

        if self.orders.len() < 2 {
            self.new_item_timer.set_cur(0.0);
        }
    }

    fn create_order(&mut self) {

        struct Ing {
            image: Image,
            chance: f64,
        }

        fn ing(image: Image, chance: f64) -> Ing {
            Ing {
                image: image,
                chance: chance,
            }
        }

        type Order = Vec<Ing>;
        let orders:Vec<Order> = vec![
            vec![ing(Image::BurgerBottom, 1.0), ing(Image::CookedPatty, 1.0), ing(Image::LettuceLeaf, 0.5), ing(Image::TomatoSlice, 0.5), ing(Image::BurgerTop, 1.0)],
        ];

        let ord_idx = (js_sys::Math::random() * (orders.len() as f64)) as usize;

        let mut new_order = IngredientStack::new(self.pos.xpos + 1000.0, self.pos.ypos);
        for ing in orders[ord_idx].iter() {
            let ing_chance = js_sys::Math::random();
            if ing_chance > ing.chance {
                continue;
            }

            new_order.add_ingredient(
                MovableIngredient::new(ing.image, 0.0, 0.0, 800.0),
                &self.pos,
                true);
        }

        let end_xpos = 20.0 + 120.0*self.orders.len() as f64;
        new_order.pos.set_end(Pos2d{xpos: end_xpos, ypos: self.pos.ypos});

        self.orders.push(new_order);

        if self.orders.len() < 5 {
            self.new_item_timer.set_cur(0.0);
        }
    }
}

impl OrderBar {
    
    fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        dest.interpolables_1d.push(self.new_item_timer.clone());

        for order in self.orders.iter() {
            order.collect_interpolables(dest);
        }

        for submitted in self.submitted_order.iter() {
            submitted.collect_interpolables(dest);
        }
    }
}

//////////////// IngredientArea

struct IngredientArea {
    ingredients: Vec<Image>,
    ingredient_words: Vec<Rc<String>>,
    pos: Pos2d,
}

impl IngredientArea {
    fn new(ingredients: Vec<Image>, xpos: f64, ypos: f64, word_bank: &WordBank) -> Self {
        let ingredient_words:Vec<Rc<String>> = (0..ingredients.len()).into_iter().map(|_idx| word_bank.get_new_word()).collect();

        IngredientArea {
            ingredients: ingredients,
            ingredient_words: ingredient_words,
            pos: Pos2d { xpos: xpos, ypos: ypos },
        }
    }

    fn draw(&self, state: &GameState) {
        // Draw the ingredients in a 6x3 grid

        for i in 0..self.ingredients.len() {
            let xpos = self.pos.xpos + (120.0 * ((i % 6) as f64));
            let ypos = self.pos.ypos + (80.0 * ((i/6) as f64));

            state.draw_image(&self.ingredients[i], xpos, ypos);

            state.draw_command_text(xpos, ypos + 80.0, &self.ingredient_words[i]);
        }

        if state.config.draw_borders {
            state.draw_border(self.pos.xpos, self.pos.ypos, (120*6) as f64, (80*3) as f64);
        }
    }

    fn handle_command(&mut self, keyword: &String, prep: &mut PreparationArea, word_bank: &WordBank) -> bool{
        for i in 0..self.ingredients.len() {
            let ing_word: &String = &self.ingredient_words[i];
            if ing_word == keyword {
                self.ingredient_words[i] = word_bank.get_new_word();
                prep.send_ingredient(
                    MovableIngredient::new(
                        self.ingredients[i],
                        120.0 * ((i%6) as f64),
                        80.0 * ((i/6) as f64),
                        500.0,
                    ),
                    &self.pos,
                    word_bank);
                return true;
            }
        }

        return false;
    }

}


struct MovableIngredient {
    image: Image,
    pos: Interpolable<Pos2d>,
}

impl MovableIngredient {
    fn new(image: Image, xpos: f64, ypos: f64, speed: f64) -> Self {
        MovableIngredient {
            image: image,
            pos: Interpolable::<Pos2d>::new(Pos2d{xpos, ypos}, speed),
        }
    }
    
    fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        dest.interpolables_2d.push(self.pos.clone());
    }

    fn draw(&self, base_pos: &Pos2d, state: &GameState) {
        let pos = self.pos.cur();
        state.draw_image(&self.image,
                         base_pos.xpos + pos.xpos,
                         base_pos.ypos + pos.ypos);
    }
}

///////// IngredientStack
struct IngredientStack {
    ingredients: Vec<MovableIngredient>,
    pos: Interpolable<Pos2d>,
}

impl IngredientStack {
    fn new(xpos: f64, ypos: f64) -> Self {
        IngredientStack {
            ingredients: Vec::new(),
            pos: Interpolable::<Pos2d>::new(Pos2d{xpos: xpos, ypos: ypos}, 500.0),
        }
    }

    fn height() -> f64 {
        150.0
    }

    fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        dest.interpolables_2d.push(self.pos.clone());
        for item in self.ingredients.iter() {
            item.collect_interpolables(dest);
        }
    }

    fn draw(&self, state: &GameState) {
        for item in self.ingredients.iter() {
            item.draw(&self.pos.cur(), state);
        }
    }

    fn add_ingredient(&mut self, ingredient: MovableIngredient, cur_base_pos: &Pos2d, immediate: bool) {
        let end = Pos2d {
            xpos: 0.0,
            ypos: IngredientStack::height() - (((self.ingredients.len()+1) as f64) *35.0)
        };

        let my_base = self.pos.cur();

        let cur_pos = ingredient.pos.cur();
        let cur_x_transformed = cur_base_pos.xpos + cur_pos.xpos - my_base.xpos;
        let cur_y_transformed = cur_base_pos.ypos + cur_pos.ypos - my_base.ypos;

        if immediate {
            ingredient.pos.set_cur(end.clone());
        }
        else {
            ingredient.pos.set_cur(Pos2d{xpos: cur_x_transformed, ypos: cur_y_transformed});
        }
        ingredient.pos.set_end(end);

        self.ingredients.push(ingredient);
    }
}

///////// PreparationAreaStack
struct PreparationAreaStack {
    stack: IngredientStack,
    base_image: Image,
    is_selected: bool,
    keyword: Rc<String>,
    end_progress: Interpolable<f64>,
}

impl PreparationAreaStack {
    fn new(xpos: f64, ypos: f64, base_image: &Image, keyword: Rc<String>) -> Self {
        PreparationAreaStack {
            stack: IngredientStack::new(xpos, ypos),
            base_image: *base_image,
            is_selected: false,
            keyword: keyword,
            end_progress: Interpolable::new(0.0, 0.2),
        }
    }

    fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        self.stack.collect_interpolables(dest);

        dest.interpolables_1d.push(self.end_progress.clone());
    }

    fn draw(&self, state: &GameState) {

        let props = &state.images[&self.base_image];
        if self.is_selected{
            state.draw_halo(
                self.stack.pos.cur().xpos,
                self.stack.pos.cur().ypos + IngredientStack::height(),
                props.width*1.2,
                props.height*1.2);  
        }
        else {
            state.draw_command_text(
                self.stack.pos.cur().xpos,
                self.stack.pos.cur().ypos + IngredientStack::height() + props.height + 50.0,
                &self.keyword);
        }
        state.draw_image(
            &self.base_image,
            self.stack.pos.cur().xpos + 10.0,
            self.stack.pos.cur().ypos + IngredientStack::height());

        if self.end_progress.is_moving() {
            // Draw the ingredients transitioning to their cooked versions, if they have one

            // Make a temporary stack of the cooked versions of our ingredients
            let cooked_stack = IngredientStack::new(self.stack.pos.cur().xpos, self.stack.pos.cur().ypos);
            for ing in self.stack.ingredients.iter() {
                let mut cooked_ing = ing.image;
                if let Some(cooked_img) = &state.images[&ing.image].cooked_image {
                    cooked_ing = *cooked_img;
                }

                cooked_stack.add_ingredient(cooked_ing, &Pos2d{xpos: 0.0, ypos: 0.0}, false);
                todo finish here
            }
        }
    }
}

///////// PreparationArea
struct PreparationArea {
    xpos: f64,
    ypos: f64,
    plate: PreparationAreaStack,
    pan: PreparationAreaStack,
}

impl PreparationArea {
    fn new(xpos: f64, ypos: f64, word_bank: &WordBank) -> Rc<RefCell<Self>> {

        let mut ret = Rc::new(RefCell::new(PreparationArea {
            xpos: xpos,
            ypos: ypos,
            plate: PreparationAreaStack::new(xpos+10.0, ypos+10.0, &Image::Plate, word_bank.get_new_word()),
            pan: PreparationAreaStack::new(xpos + 180.0, ypos+10.0,&Image::Pan, word_bank.get_new_word()),
        }));

        ret.borrow_mut().plate.is_selected = true;

        let cb_ref = ret.clone();
        ret.borrow().pan.end_progress.set_moved_handler(Box::new(move || {
            let cb_self = cb_ref.borrow_mut();
            cb_self.handle_pan_done();
        }));

        ret
    }

    fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        self.plate.collect_interpolables(dest);
        self.pan.collect_interpolables(dest);
    }

    fn handle_pan_done(&self) {
        // Nothing for now?
    }

    fn send_ingredient(&mut self, ingredient: MovableIngredient, cur_base_pos: &Pos2d, word_bank: &WordBank) {
        if self.plate.is_selected {

            if ingredient.image != Image::RawPatty {
                self.plate.stack.add_ingredient(ingredient, cur_base_pos, false);
            }
        }
        else if self.pan.is_selected {
            // Always go back to plate after sending something to pan
            self.pan.keyword = word_bank.get_new_word();
            self.pan.is_selected = false;
            self.plate.is_selected = true;

            if ingredient.image == Image::RawPatty {
                self.pan.stack.add_ingredient(ingredient, cur_base_pos, false);
                self.pan.end_progress.set_end(1.0);
            }
        }
    }

    fn handle_command(&mut self, command: &String, order_bar: &Rc<RefCell<OrderBar>>, word_bank: &WordBank) -> bool {
        if  command == "trash" {
            self.plate.stack.ingredients.clear();
            return true;
        }
        
        if command == "send" {
            let order_bar_rc= order_bar.clone();

            let done_order = &mut self.plate.stack;
            let mut new_order = IngredientStack::new(done_order.pos.cur().xpos,
                                                                      done_order.pos.cur().ypos); 
            std::mem::swap(&mut new_order.ingredients, &mut done_order.ingredients);

            order_bar.borrow_mut().try_submit_order(new_order,
                                                    order_bar_rc);
            return true;
        }
        
        if !self.plate.is_selected {
            if *self.plate.keyword == *command {
                self.plate.is_selected = true;
                self.plate.keyword = word_bank.get_new_word();
                self.pan.is_selected = false;
                return true;
            }
        }

        if !self.pan.is_selected {
            if *self.pan.keyword == *command {
                self.pan.is_selected = true;
                self.pan.keyword = word_bank.get_new_word();
                self.plate.is_selected = false;
                return true;
            }
        }

        return false;
    }

    fn draw(&self, state: &GameState) {

        let plate_props = &state.images[&Image::Plate];
        if let Some(word) = &self.plate_word {
            state.draw_command_text(self.xpos, self.ypos + IngredientStack::height() + plate_props.height + 50.0,&word);
        }
        else {
            state.draw_halo(self.xpos+10.0, self.ypos + IngredientStack::height(), plate_props.width*1.2, plate_props.height*1.2);
        }
        state.draw_image(&Image::Plate, self.xpos + 10.0, self.ypos + IngredientStack::height());

        let pan_props = &state.images[&Image::Pan];
        if let Some(word) = &self.pan_word {
            state.draw_command_text(self.xpos + 190.0, self.ypos + IngredientStack::height() + pan_props.height + 50.0,&word);
        }
        else {
            state.draw_halo(self.xpos+180.0, self.ypos + IngredientStack::height(), pan_props.width*1.2, pan_props.height*1.2);
        }
        state.draw_image(&Image::Pan, self.xpos + 180.0, self.ypos + IngredientStack::height());

        self.plate.draw(state);
        self.pan.draw(state);

        if state.config.draw_borders {
            state.draw_border(self.xpos, self.ypos, 300.0, IngredientStack::height());
        }
    }
}

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