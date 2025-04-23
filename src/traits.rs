
use crate::painter::Painter;
use crate::utils::WordBank;

pub trait BaseGame {
    //fn set_global_alpha(&self, alpha: f64);

    fn get_money(&self) -> i32;

    fn add_money(&self, amt: i32);

    //fn config<'a>(&'a self) ->  &'a GameConfig;

    fn word_bank<'a>(&'a self) -> &'a WordBank;

    //fn images<'a>(&'a self) -> &'a Images;

    fn painter<'a>(&'a self) -> &'a Painter;

    //fn image_props<'a>(&'a self, image: &Image) -> &'a ImageProps;

    fn elapsed_time(&self) -> f64;
}