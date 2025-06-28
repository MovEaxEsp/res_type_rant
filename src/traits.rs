
use crate::painter::Painter;
use crate::utils::WordBank;

use engine_p::sounds::Sounds;
use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum Sound {
    Coins,
    Frying,
    Done,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Image {
    BaconCooked,
    BaconRaw,
    BurgerBottom,
    BurgerTop,
    ClosedSign,
    CookedPatty,
    Curry,
    CurryCrab,
    Dumplings,
    EggsFried,
    EggsRaw,
    Flour,
    LettuceLeaf,
    MoneyBag,
    OpenSign,
    OverlayArrowUp,
    OverlayPlus,
    Pan,
    Plate,
    RawCrab,
    RawPatty,
    TomatoSlice,
    TriniPot,
}

pub trait BaseGame {
    //fn set_global_alpha(&self, alpha: f64);

    fn get_money(&self) -> i32;

    fn add_money(&self, amt: i32);

    //fn config<'a>(&'a self) ->  &'a GameConfig;

    fn word_bank<'a>(&'a self) -> &'a WordBank;

    //fn images<'a>(&'a self) -> &'a Images;

    fn painter<'a>(&'a self) -> &'a Painter;

    fn sounds(&self) -> &Sounds<Sound>;

    //fn image_props<'a>(&'a self, image: &Image) -> &'a ImageProps;

    fn elapsed_time(&self) -> f64;
}