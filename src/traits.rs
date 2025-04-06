use serde::{Serialize,Deserialize};

use crate::{interpolable::Pos2d, utils::WordBank};
use crate::images::{Image, Images, ImagesConfig};

#[derive(Serialize, Deserialize, Clone)]
pub struct CookingRecipe {
    pub inputs: Vec<Image>,
    pub outputs: Vec<Image>,
    pub cook_time: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CookerConfig {
    pub recipes: Vec<CookingRecipe>,
    pub base_image: Image,
    pub base_offset: Pos2d,
    pub instances: Vec<Pos2d>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BackgroundConfig {
    pub offset: Pos2d,
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
    pub offset: Pos2d,
    pub stroke: bool,
    pub style: String,
    pub font: String,
    pub size: i32,
    pub center_and_fit: bool,
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
pub struct OrderIngredientConfig {
    pub ing: Image,
    pub chance: f64,
    pub price: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderConfig {
    pub ings: Vec<OrderIngredientConfig>,
    pub weight: f64, // how likely this order is to be chosen
    pub depreciation_seconds: f64, // seconds until order price is reduced
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderBarConfig {
    pub pos: Pos2d,
    pub order_margin: f64,
    pub bg: BackgroundConfig,
    pub text_price: TextConfig,
    pub text_keyword: TextConfig,
    pub text_remaining: TextConfig,
    pub progress_bar: ProgressBarConfig,
    pub orders: Vec<OrderConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IngredientAreaConfig {
    pub pos: Pos2d,
    pub grid_width: usize,
    pub grid_item_width: f64,
    pub grid_item_height: f64,
    pub ingredients: Vec<Image>,
    pub bg: BackgroundConfig,
    pub text: TextConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PreparationAreaConfig {
    pub pos: Pos2d,
    pub cookers: Vec<CookerConfig>,
    pub bg: BackgroundConfig,
    pub text: TextConfig,
    pub progress: ProgressBarConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MoneyConfig {
    pub pos: Pos2d,
    pub bg: BackgroundConfig,
    pub text: TextConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameConfig {
    pub word_level: i32,
    pub images: ImagesConfig,
    pub order_bar: OrderBarConfig,
    pub ingredient_area: IngredientAreaConfig,
    pub preparation_area: PreparationAreaConfig,
    pub money: MoneyConfig,
}

pub trait BaseGame {
    fn set_global_alpha(&self, alpha: f64);

    fn draw_image(&self, image: &Image, pos: &Pos2d);

    fn draw_gray_image(&self, image: &Image, pos: &Pos2d);

    fn draw_area_background(&self, pos: &Pos2d, cfg: &BackgroundConfig);

    fn draw_progress_bar(&self, pos: &Pos2d, pct: f64, cfg: &ProgressBarConfig);

    //fn draw_halo(&self, xpos: f64, ypos: f64, width: f64, height: f64);

    fn draw_text(&self, text: &String, pos: &Pos2d, width: f64, cfg: &TextConfig);

    fn add_money(&self, amt: i32);

    fn config<'a>(&'a self) ->  &'a GameConfig;

    fn word_bank<'a>(&'a self) -> &'a WordBank;

    fn images<'a>(&'a self) -> &'a Images;

    //fn image_props<'a>(&'a self, image: &Image) -> &'a ImageProps;

    fn elapsed_time(&self) -> f64;
}