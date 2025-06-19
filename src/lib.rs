mod images;
mod ingredients;
mod ingredient_area;
mod interpolable;
mod keyword_entry;
mod order_bar;
mod painter;
mod preparation_area;
mod sounds;
mod state_area;
mod store;
mod traits;
mod utils;

use images::{Image, Images, ImagesConfig};
use ingredient_area::{IngredientArea, IngredientAreaGameConfig, IngredientAreaUiConfig};
use ingredients::MovableIngredient;
use interpolable::Pos2d;
use js_sys::JsString;
use keyword_entry::{KeywordEntry, KeywordEntryUiConfig};
use order_bar::{OrderBar, OrderBarGameConfig, OrderBarUiConfig};
use painter::{BackgroundConfig, Painter, TextConfig};
use preparation_area::{PreparationArea, PreparationAreaConfig};
use serde::{Serialize,Deserialize};
use sounds::{Sounds, SoundsConfig};
use state_area::{StateArea, StateGameConfig, StateUiConfig};
use store::{StoreConfig, StoreUpgradeAction, StoreUpgradeConfig, UpgradeStore};
use traits::BaseGame;
use utils::{set_panic_hook, WordBank};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, OffscreenCanvas, OffscreenCanvasRenderingContext2d};
use web_time::Instant;

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MoneyUiConfig {
    pub pos: Pos2d,
    pub bg: BackgroundConfig,
    pub text: TextConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MoneyGameConfig {
    pub starting_money: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub images: ImagesConfig,
    pub sounds: SoundsConfig,
    pub order_bar: OrderBarUiConfig,
    pub ingredient_area: IngredientAreaUiConfig,
    pub preparation_area: PreparationAreaConfig,
    pub money: MoneyUiConfig,
    pub store: StoreConfig,
    pub keyword_entry: KeywordEntryUiConfig,
    pub fps: TextConfig,
    pub state: StateUiConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameConfig {
    pub word_level: i32,
    pub unlock_all: bool,
    pub ingredient_area: IngredientAreaGameConfig,
    pub order_bar: OrderBarGameConfig,
    pub state: StateGameConfig,
    pub money: MoneyGameConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OuterConfig {
    pub ui: UiConfig,
    pub game: GameConfig,
}

///////// GameState
struct GameImp {
    cur_money: RefCell<i32>,
    words_bank: WordBank,
    painter: Painter,
    sounds: Sounds,
    config: OuterConfig,
    elapsed_time: f64,  // seconds since previous frame start (for calculating current frame)
}

impl BaseGame for GameImp {
    fn get_money(&self) -> i32 {
        *self.cur_money.borrow()
    }

    fn add_money(&self, amt: i32) {
        *self.cur_money.borrow_mut() += amt;
    }

    fn painter<'a>(&'a self) -> &'a Painter {
        &self.painter
    }

    fn sounds(&self) -> &Sounds {
        &self.sounds
    }

    fn word_bank<'a>(&'a self) -> &'a WordBank {
        &self.words_bank
    }

    fn elapsed_time(&self) -> f64 {
        self.elapsed_time
    }
}

impl GameImp {
    fn think(&mut self) {
        self.painter.think(self.elapsed_time);
    }
}

struct GameState {
    screen_canvas: HtmlCanvasElement,
    offscreen_canvas: OffscreenCanvas,
    order_bar: OrderBar,
    ingredient_area: IngredientArea,
    preparation_area: PreparationArea,
    store: UpgradeStore,
    state_area: StateArea,
    keyword_entry: KeywordEntry,
    got_first_input: bool,
    frame_times: Vec<(Instant, Instant)>, // for measuring elapsed_time, fps
    fps_str: String,
    imp: GameImp,
}

impl GameState {
    fn think(&mut self) {

        // Update frame time and FPS status
        let prev_frame = &self.frame_times[self.frame_times.len() - 2];
        let cur_frame = self.frame_times.last().unwrap();
        self.imp.elapsed_time = (cur_frame.0 - prev_frame.0).as_secs_f64();

        let frames_per_update = 10;
        if self.frame_times.len() > frames_per_update + 2 {
            // Update the FPS occasionally
            let fps_frames: Vec<(Instant, Instant)> = self.frame_times.drain(..frames_per_update).collect();
            let processing_time: f64 = fps_frames.iter().map(|v|(v.1-v.0).as_secs_f64()).sum();

            let elapsed_time = (fps_frames.last().unwrap().1 - fps_frames[0].0).as_secs_f64();
            let fps = frames_per_update as f64/elapsed_time;
            let processing_pct = (processing_time/elapsed_time) * 100.0;
            self.fps_str = format!("{:.2} FPS ({:2.2} %)", fps, processing_pct);
        }

        self.imp.think();

        self.state_area.think(&self.imp.config.game.state, &self.imp);
        self.keyword_entry.think(&self.imp);

        if self.state_area.in_store() {
            // TODO
        }
        else {
            self.order_bar.think(&self.imp, &self.imp.config.ui.order_bar, &self.imp.config.game.order_bar);
            self.ingredient_area.think(&self.imp);
            self.preparation_area.think(&self.imp.config.ui.preparation_area, &self.imp);
        }
    }

    fn draw(&self) {
        self.imp.painter().canvas().set_fill_style_str("DimGrey");
        self.imp.painter().canvas().clear_rect(0.0, 0.0, 2560.0, 1440.0);
        self.imp.painter().canvas().fill_rect(0.0, 0.0, 2560.0, 1440.0);

        self.state_area.draw(&self.imp.config.ui.state, &self.imp.config.game.state, &self.imp);

        if self.state_area.in_store() {
            self.store.draw(&self.imp, &self.imp.config.ui.store);
        }
        else {
            self.order_bar.draw(&self.imp, &self.imp.config.ui.order_bar);
            self.ingredient_area.draw(&self.imp, &self.imp.config.ui.ingredient_area);
            self.preparation_area.draw(&self.imp, &self.imp.config.ui.preparation_area);
        }
    
        self.keyword_entry.draw(&self.imp.config.ui.keyword_entry, &self.imp);

        // Draw current money
        let money_pos = self.imp.config.ui.money.pos;
        self.imp.painter().draw_area_background(&money_pos, &self.imp.config.ui.money.bg);
        self.imp.painter().draw_text(&format!("$ {}", *self.imp.cur_money.borrow()), &money_pos, self.imp.config.ui.money.bg.width, &self.imp.config.ui.money.text);

        // Draw FPS
        self.imp.painter().draw_text(&self.fps_str, &(2000, 10).into(), 300.0, &self.imp.config.ui.fps);

        let screen_context = self.screen_canvas
        .get_context("2d").unwrap().unwrap()
        .dyn_into::<CanvasRenderingContext2d>().unwrap();

        screen_context.clear_rect(0.0, 0.0, self.screen_canvas.width() as f64, self.screen_canvas.height() as f64);

        screen_context.draw_image_with_offscreen_canvas_and_dw_and_dh(
            &self.offscreen_canvas,
            0.0, 0.0,
            self.screen_canvas.width() as f64, self.screen_canvas.height() as f64)
        .expect("draw offscreen canvas");
    }

    fn update_recipes(&mut self) {
        let mut ings: HashSet<Image> = HashSet::new();
        self.ingredient_area.load_ingredients(&mut ings);
        self.preparation_area.append_possible_ingredients(&mut ings, &self.imp.config.ui.preparation_area);
        self.order_bar.set_available_ingredients(ings);
    }

    fn process_store_upgrades(&mut self, upgrades: &Vec<StoreUpgradeConfig>) {
        for upgr in upgrades.iter() {
            match upgr.action {
                StoreUpgradeAction::UnlockIngredient =>
                    self.imp.config.game.ingredient_area.ingredients.push(upgr.img),
                StoreUpgradeAction::UnlockCooker => 
                    self.imp.config.ui.preparation_area.cookers.iter_mut()
                        .filter(|c| c.base_image == upgr.img)
                        .for_each(|c| c.num_unlocked += 1),
            }
        }

        self.update_config(&self.imp.config.clone());

        self.update_recipes();
    }

    fn handle_command(&mut self) {
        let keywords = self.imp.painter.entered_keywords().clone();

        let was_in_store= self.state_area.in_store();
        self.state_area.handle_command(&keywords,&self.imp);

        if was_in_store {
            let mut upgrades: Vec<StoreUpgradeConfig> = Vec::new();

            self.store.handle_command(&keywords, &mut upgrades, self.imp.word_bank(), &self.imp, &self.imp.config.ui.store);

            self.process_store_upgrades(&upgrades);

            // If we're not in the store now, then we've transitioned back to the restaurant
            if !self.state_area.in_store() {
                self.order_bar.reset_state();
                self.preparation_area.reset_state();
                self.keyword_entry.reset_state();
            }
        }
        else {
            let mut selected_ings: Vec<MovableIngredient> = Vec::new();

            self.ingredient_area.handle_command(
                &keywords,
                &mut selected_ings,
                &self.imp);

            let handled = self.preparation_area.handle_command(&keywords, &mut selected_ings, &self.imp, &self.imp.config.ui.preparation_area);
            if !handled {
                self.order_bar.handle_command(&keywords, &mut selected_ings, &self.imp);
            }
        }
    }

    fn handle_key(&mut self, key: &str, _state_rc: &Rc<RefCell<GameState>>) {
        if !self.got_first_input {
            self.imp.sounds.handle_first_input();
            self.got_first_input = true;
        }

        if self.keyword_entry.handle_key(key, self.imp.painter.entered_keywords()) {
            self.handle_command();
            self.imp.painter.entered_keywords().clear();
        }
    }

    fn update_config(&mut self, cfg: &OuterConfig) {
        self.imp.config = cfg.clone();
        self.imp.painter.update_config(&cfg.ui.images);
        self.order_bar.update_config(&cfg.ui.order_bar, &cfg.game.order_bar, &self.imp);
        self.ingredient_area.update_config(&self.imp, &self.imp.config.ui.ingredient_area, &self.imp.config.game.ingredient_area);
        self.preparation_area.update_config(&self.imp, &self.imp.config.ui.preparation_area);
        self.state_area.update_config(&self.imp.config.ui.state, &self.imp.config.game.state);
        self.keyword_entry.update_config(&self.imp.config.ui.keyword_entry);
    }

}

static mut S_STATE: Option<Rc<RefCell<GameState>>> = None;

#[wasm_bindgen]
pub fn init_state(config: JsValue, canvas: JsValue, images: JsValue, audio_ctx: JsValue, sounds: JsValue, words_db: JsValue, bad_words_db: JsValue) {
    set_panic_hook();
    
    let game_config: OuterConfig = serde_wasm_bindgen::from_value(config).unwrap();

    let order_bar = OrderBar::new(&game_config.ui.order_bar, &game_config.game.order_bar);

    let words_bank = WordBank::new(
        &words_db.dyn_into::<JsString>().expect("wordsDb").into(),
        &bad_words_db.dyn_into::<JsString>().expect("badWords").into(),
        game_config.game.word_level as usize);

    let offscreen_canvas = OffscreenCanvas::new(2560, 1440).expect("offscreen canvas");
    let offscreen_context = offscreen_canvas.get_context("2d").unwrap().unwrap()
                        .dyn_into::<OffscreenCanvasRenderingContext2d>().unwrap();

    let screen_canvas= canvas.dyn_into::<HtmlCanvasElement>().expect("canvas");

    let painter_images = Images::new(images, &game_config.ui.images);

    let painter = Painter::new(painter_images, offscreen_context);

    let sounds = Sounds::new(audio_ctx, sounds, &game_config.ui.sounds);

    let game_imp = GameImp {
        cur_money: RefCell::new(game_config.game.money.starting_money),
        words_bank: words_bank,
        painter: painter,
        sounds: sounds,
        config: game_config,
        elapsed_time: 0.0,
    };

    let preparation_area = PreparationArea::new(&game_imp, &game_imp.config.ui.preparation_area);

    let ingredient_area = IngredientArea::new(&game_imp, &game_imp.config.ui.ingredient_area, &game_imp.config.game.ingredient_area);

    let store = UpgradeStore::new(&game_imp, &game_imp.config.ui.store);

    let state_area = StateArea::new(&game_imp.config.ui.state, &game_imp.config.game.state, &game_imp);

    let keyword_entry = KeywordEntry::new(&game_imp.config.ui.keyword_entry);

    let mut state = GameState{
        screen_canvas: screen_canvas,
        offscreen_canvas: offscreen_canvas,
        order_bar: order_bar,
        ingredient_area: ingredient_area,
        preparation_area: preparation_area,
        store: store,
        keyword_entry: keyword_entry,
        got_first_input: false,
        state_area: state_area,
        frame_times: Vec::new(),
        imp: game_imp,
        fps_str: "".to_string(),
    };

    state.frame_times.push((Instant::now(), Instant::now()));

    state.update_recipes();

    if state.imp.config.game.unlock_all {
        let mut upgrades: Vec<StoreUpgradeConfig> = Vec::new();
        state.store.unlock_all(&mut upgrades, &state.imp.config.ui.store);
        state.process_store_upgrades(&upgrades);
    }

    unsafe {
        S_STATE = Some(Rc::new(RefCell::new(state)));
    }
    
}

fn run_frame_imp(state_rc: &Rc<RefCell<GameState>>) {
    let mut state = state_rc.borrow_mut();

    let now = Instant::now();
    state.frame_times.push((now, now));

    state.think();
    state.draw();

    state.frame_times.last_mut().unwrap().1 = Instant::now();
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

pub fn build_default_config() -> OuterConfig {
    OuterConfig {
        ui: UiConfig {
            images: Images::default_config(),
            sounds: Sounds::default_config(),
            order_bar: OrderBar::default_ui_config(),
            ingredient_area: IngredientArea::default_ui_config(),
            preparation_area: PreparationArea::default_config(),
            store: UpgradeStore::default_config(),
            keyword_entry: KeywordEntry::default_ui_config(),
            state: StateArea::default_ui_config(),
            money: MoneyUiConfig {
                pos: (50, 50).into(),
                bg: BackgroundConfig {
                    offset: (0, -20).into(),
                    width: 400.0,
                    height: 250.0,
                    corner_radius: 30.0,
                    border_style: "black".to_string(),
                    border_alpha: 0.3,
                    border_width: 5.0,
                    bg_style: "green".to_string(),
                    bg_alpha: 0.2
                },
                text: TextConfig {
                    offset: (40, 40).into(),
                    stroke: true,
                    style: "gold".to_string(),
                    font: "comic sans".to_string(),
                    size: 128,
                    center_and_fit: false,
                    alpha: 1.0,
                    is_command: false,
                }
            },
            fps: TextConfig {
                offset: (0, 0).into(),
                stroke: false,
                style: "black".to_string(),
                font: "comic sans".to_string(),
                size: 30,
                center_and_fit: false,
                alpha: 0.7,
                is_command: false,
            },
        },
        game: GameConfig {
            word_level: 0,
            unlock_all: false,
            ingredient_area: IngredientArea::default_game_config(),
            order_bar: OrderBar:: default_game_config(),
            state: StateArea::default_game_config(),
            money: MoneyGameConfig {
                starting_money: 0,
            },
        }
    }
}

#[wasm_bindgen]
pub fn default_config() -> JsValue {
    serde_wasm_bindgen::to_value(&build_default_config()).unwrap()
}

#[wasm_bindgen]
pub fn update_config(config: JsValue) {
    match serde_wasm_bindgen::from_value::<OuterConfig>(config) {
        Ok(cfg) => {
            unsafe {
                #[allow(static_mut_refs)]
                let state: &Rc<RefCell<GameState>> = S_STATE.as_mut().unwrap();
                state.borrow_mut().update_config(&cfg);
            }
        }
        Err(e) => {
            log(&format!("Failed parsing config: {}", e));
        }
    }
}

#[wasm_bindgen]
pub fn resource_names() -> JsValue {
    #[derive(Serialize)]
    pub struct ResourceList {
        pub images: Vec<String>,
        pub sounds: Vec<String>,
    }

    let cfg = build_default_config();

    let resources = ResourceList {
        images: cfg.ui.images.images.iter().map(|img| img.image_name.clone()).collect(),
        sounds: cfg.ui.sounds.sounds.iter().flat_map(|snd| snd.sound_names.iter().cloned()).collect(),
    };

    serde_wasm_bindgen::to_value(&resources).unwrap()
}