mod utils;

use utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use std::collections::HashMap;
use std::rc::Rc;
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

struct Interpolable {
    cur: f64,
    end: f64,
    speed: f64, // units per second
}

impl Interpolable {
    fn new(val: f64, speed: f64) -> Self{
        Interpolable {
            cur: val,
            end: val,
            speed: speed
        }
    }

    fn advance(&mut self, elapsed_time: f64) {
        if self.cur < self.end {
            let move_amt = self.speed*elapsed_time;
            if self.cur + move_amt > self.end {
                self.cur = self.end;
            }
            else {
                self.cur += move_amt;
            }
        }
        else if self.cur > self.end {
            let move_amt = self.speed*elapsed_time;
            if self.cur - move_amt < self.end {
                self.cur = self.end;
            }
            else {
                self.cur -= move_amt;
            }
        }
    }

}

///////////// BaseDrawable

/*
struct BaseDrawable {
    image: Image,
    xpos: f64,
    ypos: f64,
    std_width: f64,
    std_height: f64,
    scale: f64,
}

impl BaseDrawable {
    fn new(image: Image, xpos: i32, ypos: i32, std_width: i32, std_height: i32, scale: f64) -> Self{
        BaseDrawable {
            image: image,
            xpos: xpos.into(),
            ypos: ypos.into(),
            std_width: std_width.into(),
            std_height: std_height.into(),
            scale: scale,
        }
    }

    fn burger_top(xpos: i32, ypos: i32, scale: f64) -> Self {
        BaseDrawable::new(Image::BurgerTop, xpos, ypos, 100, 30, scale)
    }
    fn burger_bottom(xpos: i32, ypos: i32, scale: f64) -> Self {
        BaseDrawable::new(Image::BurgerBottom, xpos, ypos, 100, 30, scale)
    }
}

impl Entity for BaseDrawable {
    fn draw(&self, game: &GameState) {
        game.canvas.draw_image_with_html_image_element_and_dw_and_dh(
            &game.images.get(&self.image).unwrap().image,
            self.xpos,
            self.ypos,
            self.std_width*self.scale,
            self.std_height*self.scale
        )
        .expect("draw");
    }
}

*/
///////// GameState

struct GameState {
    canvas: CanvasRenderingContext2d,
    images: HashMap<Image, ImageProps>,
    entities: Vec<Rc<dyn Entity>>,
    order_bar: OrderBar,
    frame_start: Instant,  // time when previous frame started
    elapsed_time: f64,  // seconds since previous frame start (for calculating current frame)
}

impl GameState {
    fn draw(&self, image: &Image, xpos: f64, ypos: f64) {

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
}

static mut S_STATE: Option<GameState> = None;

/*
//////////// DrawableStack

struct DrawableStack {
    drawables: Vec<Image>,
    xpos: f64,
    ypos: f64,
    std_width: f64,
    std_height: f64,
    scale: f64,
    margin: f64,
}

impl DrawableStack {
    fn pushDrawable(&mut self, drawable: Image) {
        self.drawables.push(drawable);
        
        let mut totalHeight = 0.0;
        for d in self.drawables.iter() {
            totalHeight += d.std_height;
        }

        self.margin = (self.std_height - totalHeight)/((self.drawables.len() as f64) + 1.0); 

        let mut cur_y = self.margin;
        for d in self.drawables.iter_mut(){
            d.xpos = self.xpos;
            d.ypos = cur_y;
            cur_y += d.std_height + self.margin;
        }
    }
}

impl Entity for DrawableStack {
    fn draw(&self, state: &GameState) {
        for d in self.drawables.iter() {
            d.draw(state);
        }
    }
}
*/

///////////// OrderBar

struct OrderBarOrder {
    items: Vec<Image>,
}

// Manager for the list of orders at the top of the screen
struct OrderBar {
    orders: Vec<OrderBarOrder>,
    last_item_x: Interpolable,
}

impl Entity for OrderBar {
    fn draw(&self, state: &GameState) {
        let mut xpos = 20.0;
        
        for i in 0..self.orders.len() {
            if i == self.orders.len() - 1 {
                xpos = self.last_item_x.cur;
            }

            let order = &self.orders[i];
            let mut ypos = 150.0;
            for item in (&order.items).iter().rev() {
                state.draw(item, xpos, ypos);
                ypos -= 50.0;
            }

            xpos += 120.0;
        }
    }
}

impl OrderBar {
    fn think(&mut self, elapsed_time: f64) {
        // Advance interporable.  when reaches end, make new order until there's 5.

        self.last_item_x.advance(elapsed_time);
        if self.last_item_x.cur == self.last_item_x.end && self.orders.len() < 7 {
            self.last_item_x.end = (20 + self.orders.len()*120) as f64;
            self.last_item_x.cur = 1000.0;

            self.orders.push(OrderBarOrder { items: vec![
                Image::BurgerTop, Image::BurgerBottom
            ]});
        }

    }
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
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

    let mut image_def = |image: Image, width: f64, height: f64| -> (Image, ImageProps) {
        (image, ImageProps{image: image_map.remove(&image).unwrap(), width: width, height: height})
    };

    let state_images = HashMap::from([
        image_def(Image::BurgerTop, 100.0, 30.0),
        image_def(Image::BurgerBottom, 100.0, 30.0),
    ]);

    let order_bar = OrderBar {
        orders: vec![
        ],
        last_item_x: Interpolable::new(20.0, 800.0)
     };

    let state = GameState{
        canvas: canvas.dyn_into::<HtmlCanvasElement>().expect("canvas")
                .get_context("2d").unwrap().unwrap()
                .dyn_into::<CanvasRenderingContext2d>().unwrap(),
        images: state_images,
        entities: Vec::new(),
        order_bar: order_bar,
        frame_start: Instant::now(),
        elapsed_time: 0.0
    };

    //state.entities.push(Rc::new(BaseDrawable::burger_top(20, 30, 1.0)));
    //state.entities.push(Rc::new(BaseDrawable::burger_bottom(20, 90, 1.0)));

    unsafe {
        S_STATE = Some(state);
    }
    
}

fn run_frame_imp(state: &mut GameState) {

    state.elapsed_time = state.frame_start.elapsed().as_secs_f64();
    state.frame_start = Instant::now();

    state.canvas.rect(0.0, 0.0, 1000.0, 700.0);
    state.canvas.fill();

    // Let every entitity think
    //for i in 0..state.entities.len() {
    //    let entity = state.entities[i].as_ref();
        //entity.think(state);
    //}

    state.order_bar.think(state.elapsed_time);

    // Let every entitity draw
    for i in 0..state.entities.len() {
        let entity = state.entities[i].as_ref();
        entity.draw(state);
    }

    state.order_bar.draw(state);

    /*
    let burger_top = state.images.get(&ImageType::BurgerTop).unwrap();
    let burger_bottom = state.images.get(&ImageType::BurgerBottom).unwrap(); 

    state.canvas.draw_image_with_html_image_element_and_dw_and_dh(&burger_top, 100.0, 100.0 ,burger_top.width().into(), burger_top.height().into()).expect("draw");
    state.canvas.draw_image_with_html_image_element_and_dw_and_dh(&burger_bottom, 120.0, 250.0, burger_bottom.width().into(), burger_bottom.height().into()).expect("draw");
*/

}

#[wasm_bindgen]
pub fn run_frame() {
    unsafe {
        #[allow(static_mut_refs)]
        let state: &mut GameState = S_STATE.as_mut().unwrap();
        run_frame_imp(state);
    }
}
