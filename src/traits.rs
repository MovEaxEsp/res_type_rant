use web_sys::{HtmlImageElement, OffscreenCanvas};

use serde::{Serialize,Deserialize};

use crate::{interpolable::Pos2d, utils::WordBank};

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
    pub gray_image: OffscreenCanvas,
    pub cooked_image: Option<Image>,
    pub width: f64,
    pub height: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BackgroundConfig {
    pub x_off: f64,
    pub y_off: f64,
    pub width: f64,
    pub height: f64,
    pub corner_radius: f64,
    pub border_style: String,
    pub border_alpha: f64,
    pub border_width: f64,
    pub bg_style: String,
    pub bg_alpha: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TextConfig {
    pub x_off: f64,
    pub y_off: f64,
    pub stroke: bool,
    pub style: String,
    pub font: String,
    pub size: i32,
    pub scale_text: bool,
    pub is_command: bool,
    pub alpha: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProgressBarConfig {
    pub bg: BackgroundConfig,
    pub done_style: String,
    pub done_alpha: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderBarConfig {
    pub xpos: f64,
    pub ypos: f64,
    pub depreciation_seconds: f64,
    pub bg: BackgroundConfig,
    pub text_price: TextConfig,
    pub text_keyword: TextConfig,
    pub text_remaining: TextConfig,
    pub progress_bar: ProgressBarConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IngredientAreaConfig {
    pub xpos: f64,
    pub ypos: f64,
    pub grid_width: usize,
    pub grid_item_width: f64,
    pub grid_item_height: f64,
    pub bg: BackgroundConfig,
    pub text: TextConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PreparationAreaStackConfig {
    pub xpos: f64,
    pub ypos: f64,
    pub base_x_off: f64,
    pub base_y_off: f64,
    pub cook_time: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PreparationAreaConfig {
    pub xpos: f64,
    pub ypos: f64,
    pub plate: PreparationAreaStackConfig,
    pub pan: PreparationAreaStackConfig,
    pub bg: BackgroundConfig,
    pub text: TextConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MoneyConfig {
    pub xpos: f64,
    pub ypos: f64,
    pub bg: BackgroundConfig,
    pub text: TextConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameConfig {
    pub word_level: i32,
    pub draw_borders: bool,
    pub order_bar: OrderBarConfig,
    pub ingredient_area: IngredientAreaConfig,
    pub preparation_area: PreparationAreaConfig,
    pub money: MoneyConfig,
    pub draw_gray: bool,
}

pub trait BaseGame {
    fn set_global_alpha(&self, alpha: f64);

    fn draw_image(&self, image: &Image, pos: &Pos2d);

    fn draw_gray_image(&self, image: &Image, pos: &Pos2d);

    fn draw_border(&self, xpos: f64, ypos: f64, width: f64, height: f64);

    fn draw_area_background(&self, pos: &Pos2d, cfg: &BackgroundConfig);

    fn draw_progress_bar(&self, pos: &Pos2d, pct: f64, cfg: &ProgressBarConfig);

    fn draw_halo(&self, xpos: f64, ypos: f64, width: f64, height: f64);

    fn draw_text(&self, text: &String, pos: &Pos2d, cfg: &TextConfig);

    fn add_money(&self, amt: i32);

    fn config<'a>(&'a self) ->  &'a GameConfig;

    fn word_bank<'a>(&'a self) -> &'a WordBank;

    fn image_props<'a>(&'a self, image: &Image) -> &'a ImageProps;

    fn elapsed_time(&self) -> f64;
}