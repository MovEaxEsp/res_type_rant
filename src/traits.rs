
use crate::{interpolable::Pos2d, utils::WordBank};
use crate::images::{Image, Images};

use serde::{Serialize,Deserialize};

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
pub struct MoneyConfig {
    pub pos: Pos2d,
    pub bg: BackgroundConfig,
    pub text: TextConfig,
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

    //fn config<'a>(&'a self) ->  &'a GameConfig;

    fn word_bank<'a>(&'a self) -> &'a WordBank;

    fn images<'a>(&'a self) -> &'a Images;

    //fn image_props<'a>(&'a self, image: &Image) -> &'a ImageProps;

    fn elapsed_time(&self) -> f64;
}