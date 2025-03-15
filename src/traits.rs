use web_sys::HtmlImageElement;

use serde::{Serialize,Deserialize};

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Image {
    Plate,
    Pan,
    BurgerTop,
    BurgerBottom,
    LettuceLeaf,
    TomatoSlice,
    RawPatty,
    CookedPatty,
}

pub struct ImageProps {
    pub image: HtmlImageElement,
    pub cooked_image: Option<Image>,
    pub width: f64,
    pub height: f64,
}


#[derive(Serialize, Deserialize)]
pub struct GameConfig {
    pub word_level: i32,
    pub draw_borders: bool,
}

pub trait BaseGame {
    fn draw_image(&self, image: &Image, xpos: f64, ypos: f64);

    fn draw_border(&self, xpos: f64, ypos: f64, width: f64, height: f64);

    fn draw_halo(&self, xpos: f64, ypos: f64, width: f64, height: f64);

    fn draw_command_text(&self, xpos: f64, ypos: f64, text: &String);

    fn config<'a>(&'a self) ->  &'a GameConfig;

    fn image_props<'a>(&'a self, image: &Image) -> &'a ImageProps;
}