
use serde::{Serialize,Deserialize};

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{ImageData, HtmlImageElement, OffscreenCanvas, OffscreenCanvasRenderingContext2d};

use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Image {
    BaconCooked,
    BaconRaw,
    BurgerBottom,
    BurgerTop,
    CookedPatty,
    Curry,
    CurryCrab,
    Dumplings,
    EggsFried,
    EggsRaw,
    Flour,
    LettuceLeaf,
    OpenSign,
    OverlayPlus,
    Pan,
    Plate,
    RawCrab,
    RawPatty,
    TomatoSlice,
    TriniPot,
}

// The base images used for drawing an image
struct ImageProps {
    pub image: HtmlImageElement,
    pub gray_image: OffscreenCanvas,
    pub cfg: ImageConfig,
}

// The configuration for an image
#[derive(Serialize, Deserialize, Clone)]
pub struct ImageConfig {
    pub image: Image,
    pub image_name: String,
    pub width: f64,
    pub height: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ImagesConfig {
    pub images: Vec<ImageConfig>,
    pub scale: f64,
}

pub struct Images {
    images: HashMap<Image, ImageProps>,
    scale: f64,
}

impl Images {
    pub fn new(js_images: JsValue, cfg: &ImagesConfig) -> Self {
        let mut self_images: HashMap<Image, ImageProps> = HashMap::new();

        for img_cfg in cfg.images.iter() {
            let imgjs = js_sys::Reflect::get(&js_images, &((&img_cfg.image_name).into())).expect("img");
            let htmlimg = imgjs.dyn_into::<HtmlImageElement>().expect("htmlimg");

            // Create gray image
            let gray_canvas = OffscreenCanvas::new(htmlimg.width(), htmlimg.height()).expect("gray canvas");
            let gray_context = gray_canvas.get_context("2d").unwrap().unwrap()
                                .dyn_into::<OffscreenCanvasRenderingContext2d>().unwrap();

            gray_context.draw_image_with_html_image_element(
                &htmlimg,
                0.0,
                0.0)
            .expect("draw");
            
            let img_data = gray_context.get_image_data(0.0, 0.0, htmlimg.width() as f64, htmlimg.height() as f64).expect("gray imgdata");
            let mut img_arr = img_data.data();
            
            for i in 0..img_arr.len()/4 {
                // From https://tannerhelland.com/2011/10/01/grayscale-image-algorithm-vb6.html
                let pixel_val = (img_arr[i*4] as f64* 0.299 + img_arr[i*4+1] as f64 * 0.587 + img_arr[i*4+2] as f64 * 0.114) as u8;
                img_arr[i*4] = pixel_val;
                img_arr[i*4 + 1] = pixel_val;
                img_arr[i*4 + 2] = pixel_val;
            }
            
            let image_data_temp = ImageData::new_with_u8_clamped_array(Clamped(&img_arr.0[..]), htmlimg.width()).expect("new imagedata");
            gray_context.put_image_data(&image_data_temp, 0.0, 0.0).expect("pub image data");

            self_images.insert(
                img_cfg.image,
                ImageProps {
                    image: htmlimg.clone(),
                    gray_image: gray_canvas.clone(),
                    cfg: img_cfg.clone()
                });
        }

        Images {
            images: self_images,
            scale: cfg.scale,
        }
    }

    pub fn draw_image(&self, canvas: &OffscreenCanvasRenderingContext2d, image: &Image, x: f64, y: f64) {

        let props = self.images.get(image).unwrap();

        canvas.draw_image_with_html_image_element_and_dw_and_dh(
            &props.image,
            x,
            y,
            props.cfg.width * self.scale,
            props.cfg.height * self.scale,
        )
        .expect("draw");
    }

    pub fn draw_gray_image(&self, canvas: &OffscreenCanvasRenderingContext2d, image: &Image, x: f64, y:f64) {

        let props = self.images.get(image).unwrap();

        canvas.draw_image_with_offscreen_canvas_and_dw_and_dh(
            &props.gray_image,
            x,
            y,
            props.cfg.width * self.scale,
            props.cfg.height * self.scale)
        .expect("draw gray");
    }

    pub fn image_height(&self, image: &Image) -> f64 {
        self.images.get(image).unwrap().cfg.height * self.scale
    }

    pub fn image_width(&self, image: &Image) -> f64 {
        self.images.get(image).unwrap().cfg.width * self.scale
    }

    pub fn update_config(&mut self, cfg: &ImagesConfig) {
        self.scale = cfg.scale;
        for img_cfg in cfg.images.iter() {
            self.images.get_mut(&img_cfg.image).unwrap().cfg = img_cfg.clone();
        }
    }

    pub fn default_config() -> ImagesConfig {
        let image_def = |img, name: &str, width, height| ImageConfig {
            image: img,
            image_name: name.to_string(),
            width: width,
            height: height
        };

        ImagesConfig {
            scale: 1.0,
            images: vec![
                image_def(Image::BaconCooked,  "bacon_cooked.png",  100.0, 70.0),
                image_def(Image::BaconRaw,     "bacon_raw.png",     100.0, 60.0),
                image_def(Image::BurgerBottom, "burger_bottom.png", 100.0, 30.0),
                image_def(Image::BurgerTop,    "burger_top.png",    100.0, 30.0),
                image_def(Image::CookedPatty,  "cooked_patty.png",  100.0, 30.0),
                image_def(Image::Curry,        "curry.png",         100.0, 140.0),
                image_def(Image::CurryCrab,    "curry_crab.png",    150.0, 100.0),
                image_def(Image::Dumplings,    "dumplings.png",     100.0, 60.0),
                image_def(Image::EggsFried,    "eggs_fried.png",    100.0, 70.0),
                image_def(Image::EggsRaw,      "eggs_raw.png",      100.0, 60.0),
                image_def(Image::Flour,        "flour.png",         100.0, 100.0),
                image_def(Image::LettuceLeaf,  "lettuce_leaf.png",  100.0, 30.0),
                image_def(Image::OpenSign,     "open_sign.png",     300.0, 200.0),
                image_def(Image::OverlayPlus,  "overlay_plus.png",   40.0, 40.0),
                image_def(Image::Pan,          "pan.png",           200.0, 30.0),
                image_def(Image::Plate,        "plate.png",         100.0, 30.0),
                image_def(Image::RawCrab,      "raw_crab.png",      100.0, 60.0),
                image_def(Image::RawPatty,     "raw_patty.png",     100.0, 30.0),
                image_def(Image::TomatoSlice,  "tomato_slice.png",  100.0, 30.0),
                image_def(Image::TriniPot,     "trini_pot.png",     180.0, 100.0),
            ]
        }
    }
}