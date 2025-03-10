mod utils;

use utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use std::cell::RefCell;
use std::cmp::min;
use std::collections::HashMap;
use std::rc::Rc;
use std::ops::Deref;
use web_time::Instant;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

struct ImageProps {
    image: HtmlImageElement,
    width: f64,
    height: f64,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
enum Image {
    BurgerTop,
    BurgerBottom,
}

// An object that can be drawn
trait Entity {
    fn draw(&self, _game: &GameState) {}
}

//////////// Interpolable
trait Advanceable<T> {
    fn advance(&mut self, end: T, speed: f64, elapsed_time: f64);
}

impl Advanceable<f64> for f64 {
    fn advance(self:&mut f64, end: f64, speed: f64, elapsed_time: f64) {
        if *self < end {
            let move_amt = speed*elapsed_time;
            if *self + move_amt > end {
                *self = end;
            }
            else {
                *self += move_amt;
            }
        }
        else if *self > end {
            let move_amt = speed*elapsed_time;
            if *self - move_amt < end {
                *self = end;
            }
            else {
                *self -= move_amt;
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct Pos2d {
    xpos: f64,
    ypos: f64,
}

impl Advanceable<Pos2d> for Pos2d {
    fn advance(self:&mut Pos2d, end: Pos2d, speed: f64, elapsed_time: f64) {
        let x_diff = end.xpos - self.xpos;
        let y_diff = end.ypos - self.ypos;
        let dist = ((x_diff.powf(2.0) + y_diff).powf(2.0)).sqrt();
        let move_amt = speed*elapsed_time;

        if (dist < move_amt) {
            *self = end;
        }
        else {
            let x_prop = x_diff.abs() / (x_diff.abs() + y_diff.abs());
            let x_move_amt = f64::min(x_diff.abs(), move_amt * x_prop);
            if x_diff < 0.0 {
                self.xpos -= x_move_amt;
            }
            else if x_diff > 0.0 {
                self.xpos += x_move_amt;
            }

            let y_move_amt = f64::min(y_diff.abs(), move_amt * (1.0-x_prop));
            if y_diff < 0.0 {
                self.ypos -= y_move_amt;
            }
            else if y_diff > 0.0 {
                self.ypos += y_move_amt;
            }
        }
    }
}

struct InterpolableImp<T> {
    cur: T,
    end: T,
    speed: f64, // units per second
    moved_handler: RefCell<Box<dyn FnMut()>>,
}

impl<T> InterpolableImp<T>
where T: Clone {
    fn new(val: T, speed: f64) -> Self{
        InterpolableImp {
            cur: val.clone(),
            end: val,
            speed: speed,
            moved_handler: RefCell::new(Box::new(|| {})),
        }
    }
}

#[derive(Clone)]
struct Interpolable<T> {
    imp: Rc<RefCell<InterpolableImp<T>>>,
}

impl<T> Interpolable<T>
where T: Clone + PartialEq + Advanceable<T> {
    fn new (val: T, speed: f64) -> Self {
        Interpolable {
            imp: Rc::new(RefCell::new(InterpolableImp::new(val, speed)))
        }
    }

    fn is_moving(&self) -> bool {
        let imp = self.imp.borrow();
        imp.cur != imp.end
    }

    fn cur(&self) -> T {
        self.imp.borrow().cur.clone()
    }

    fn end(&self) -> T {
        self.imp.borrow().end.clone()
    }

    fn advance(&self, elapsed_time:f64) {
        let mut done_cb: Box<dyn FnMut()> = Box::new(||{});
        let mut have_done_cb = false;

        {
            let mut imp = self.imp.borrow_mut();
            if imp.cur != imp.end {
                let end = imp.end.clone();
                let speed = imp.speed;
                imp.cur.advance(end, speed, elapsed_time);
                if imp.cur == imp.end {
                    done_cb = imp.moved_handler.replace(done_cb);
                    have_done_cb = true;
                }
            }
        }

        if have_done_cb {
            done_cb();

            let mut imp = self.imp.borrow_mut();
            imp.moved_handler.replace(done_cb);
        }
    }

    fn set_cur(&self, cur: T) {
        self.imp.borrow_mut().cur = cur;
    }

    fn set_end(&self, end: T) {
        self.imp.borrow_mut().end = end;
    }

    fn set_moved_handler(&self, handler: Box<dyn FnMut()>) {
        self.imp.borrow_mut().moved_handler = RefCell::new(handler);
    }
}

struct InterpolableStore {
    interpolables_1d: Vec<Interpolable<f64>>,
    interpolables_2d: Vec<Interpolable<Pos2d>>,
}

impl InterpolableStore {
    fn new() -> Self {
        InterpolableStore {
            interpolables_1d: Vec::new(),
            interpolables_2d: Vec::new(),
        }
    }

    fn advance_all(&self, elapsed_time: f64) {
        for intr in self.interpolables_1d.iter() {
            intr.advance(elapsed_time);
        }
        for intr in self.interpolables_2d.iter() {
            intr.advance(elapsed_time);
        }
    }
}

///////// GameState

struct GameState {
    canvas: CanvasRenderingContext2d,
    images: HashMap<Image, ImageProps>,
    entities: Vec<Rc<dyn Entity>>,
    order_bar: Rc<RefCell<OrderBar>>,
    ingredient_area: IngredientArea,
    preparation_area: PreparationArea,
    frame_start: Instant,  // time when previous frame started
    elapsed_time: f64,  // seconds since previous frame start (for calculating current frame)
    entered_text: String,
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

    fn draw(&self) {
        self.order_bar.borrow().draw(self);
        self.ingredient_area.draw(self);
        self.preparation_area.draw(self);
    
        // Draw current input
        self.canvas.set_fill_style_str("yellow");
        self.canvas.set_font("48px serif");
        self.canvas.fill_text(&self.entered_text, 20.0, 600.0).expect("text");
    }

    fn handle_command(&mut self) {
        if  self.entered_text == "trash" {
            self.preparation_area.plate.ingredients.clear();
        }
        else if self.entered_text == "send" {
            let order_bar_rc= self.order_bar.clone();

            let done_order = &mut self.preparation_area.plate;
            let mut new_order = IngredientStack::new(done_order.pos.cur().xpos,
                                                                      done_order.pos.cur().ypos); 
            std::mem::swap(&mut new_order.ingredients, &mut done_order.ingredients);

            self.order_bar.borrow_mut().try_submit_order(new_order,
                                                         order_bar_rc);
        }
        else {
            self.ingredient_area.find_ingredient(&self.entered_text,
                                                 &mut self.preparation_area);
        }
    }

    fn handle_key(&mut self, key: &str, state_rc: &Rc<RefCell<GameState>>) {
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

    fn try_submit_order(&mut self, mut order: IngredientStack, self_rc: Rc<RefCell<Self>>) {
        for i in 0..self.orders.len() {
            let bar_order = &self.orders[i];
            if order.ingredients.len() != bar_order.ingredients.len() {
                continue;
            }

            let mut is_match = true;
            for ing_idx in 0..order.ingredients.len() {
                if order.ingredients[i].image != bar_order.ingredients[i].image {
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
            break;
        }
    }

    fn serve_order(&mut self, order_idx: usize, self_rc: Rc<RefCell<Self>>) {
        let mut order = &self.orders[order_idx];
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
    }

    fn create_order(&mut self) {

        let mut new_order = IngredientStack::new(self.pos.xpos + 1000.0, self.pos.ypos);
        new_order.add_ingredient(
            MovableIngredient::new(Image::BurgerBottom, 0.0, 0.0, 800.0),
            &self.pos,
            true);
        new_order.add_ingredient(
            MovableIngredient::new(Image::BurgerTop, 0.0, 0.0, 800.0),
            &self.pos,
            true);

        let end_xpos = 20.0 + 120.0*self.orders.len() as f64;
        new_order.pos.set_end(Pos2d{xpos: end_xpos, ypos: self.pos.ypos});

        self.orders.push(new_order);

        if self.orders.len() < 7 {
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
    ingredient_words: Vec<String>,
    pos: Pos2d,
}

impl IngredientArea {
    fn new(ingredients: Vec<Image>, xpos: f64, ypos: f64) -> Self {
        let word_bank = ["foo", "bar", "baz"];

        let ingredient_words:Vec<String> = (0..ingredients.len()).into_iter().map(|idx| word_bank[idx].to_string()).collect();

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

            // Draw the word
            state.canvas.set_stroke_style_str("yellow");
            state.canvas.set_font("48px serif");
            state.canvas.stroke_text(&self.ingredient_words[i], xpos, ypos + 80.0).expect("text");
        }
    }

    fn find_ingredient(&self, keyword: &String, prep:&mut PreparationArea) {
        for i in 0..self.ingredients.len() {
            if self.ingredient_words[i] == *keyword {
                log(&format!("Found ingredient: {}", keyword));
                prep.plate.add_ingredient(
                    MovableIngredient::new(
                        self.ingredients[i],
                        (120.0 * ((i%6) as f64)),
                        (80.0 * ((i/6) as f64)),
                        500.0,
                    ),
                    &self.pos,
                    false);
                return;
            }
        }

        log(&format!("Didn't find ingredient: {}", keyword));
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

    fn is_moving(&self) -> bool {
        self.pos.is_moving()
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

    fn add_ingredient(&mut self, mut ingredient: MovableIngredient, cur_base_pos: &Pos2d, immediate: bool) {
        let end = Pos2d {
            xpos: 0.0,
            ypos: 300.0 - (((self.ingredients.len()+1) as f64) *50.0)
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

///////// PreparationArea
struct PreparationArea {
    xpos: f64,
    ypos: f64,
    plate: IngredientStack,
}

impl PreparationArea {
    fn new(xpos: f64, ypos: f64) -> Self {
        PreparationArea {
            xpos: xpos,
            ypos: ypos,
            plate: IngredientStack::new(xpos+10.0, ypos+10.0),
        }
    }

    fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        self.plate.collect_interpolables(dest);
    }

    fn draw(&self, state: &GameState) {
        self.plate.draw(state);
    }
}

#[wasm_bindgen]
pub fn init_state(canvas: JsValue, images: JsValue) {

    set_panic_hook();

    let mut image_map: HashMap<Image, HtmlImageElement> = HashMap::new();

    let image_names = HashMap::from([
        (Image::BurgerTop, "burger_top.png"),
        (Image::BurgerBottom, "burger_bottom.png"),
    ]);

    for (imgtype, imgname) in image_names {
        let imgjs = js_sys::Reflect::get(&images, &imgname.into()).expect(imgname);
        let htmlimg = imgjs.dyn_into::<HtmlImageElement>().expect(imgname);
        image_map.insert(imgtype, htmlimg);
    }

    let mut image_def = |image: Image, width: f64, height: f64| {
        (image, ImageProps{image: image_map.remove(&image).unwrap(), width: width, height: height})
    };

    let state_images = HashMap::from([
        image_def(Image::BurgerTop, 100.0, 30.0),
        image_def(Image::BurgerBottom, 100.0, 30.0),
    ]);

    let order_bar = OrderBar::new(10.0, 20.0);

    let state = GameState{
        canvas: canvas.dyn_into::<HtmlCanvasElement>().expect("canvas")
                .get_context("2d").unwrap().unwrap()
                .dyn_into::<CanvasRenderingContext2d>().unwrap(),
        images: state_images,
        entities: Vec::new(),
        order_bar: order_bar,
        ingredient_area: IngredientArea::new(vec![Image::BurgerTop, Image::BurgerBottom], 60.0, 300.0),
        preparation_area: PreparationArea::new(800.0, 300.0),
        frame_start: Instant::now(),
        elapsed_time: 0.0,
        entered_text: String::new(),
    };

    unsafe {
        S_STATE = Some(Rc::new(RefCell::new(state)));
    }
    
}

fn run_frame_imp(state_rc: &Rc<RefCell<GameState>>) {
    let mut state = state_rc.borrow_mut();

    state.elapsed_time = state.frame_start.elapsed().as_secs_f64();
    state.frame_start = Instant::now();

    state.canvas.rect(0.0, 0.0, 1000.0, 700.0);
    state.canvas.set_fill_style_str("black");
    state.canvas.fill();

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
        state.preparation_area.collect_interpolables(&mut interpolables);
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