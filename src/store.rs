
use crate::images::Image;
use crate::ingredients::{IngredientStack, MovableIngredient};
use crate::interpolable::{Interpolable, Pos2d};
use crate::traits::{BackgroundConfig, BaseGame, TextConfig};
use crate::utils::WordBank;

use serde::{Serialize,Deserialize};

use std::rc::Rc;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum StoreUpgradeAction {
    UnlockIngredient,
    UnlockCooker,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoreUpgradeConfig {
    pub img: Image,
    pub overlay: Image,
    pub cost: i32,
    pub action: StoreUpgradeAction
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StoreConfig {
    pub pos: Pos2d,
    pub bg: BackgroundConfig,
    pub text_keyword: TextConfig,
    pub text_price: TextConfig,
    pub upgrades: Vec<Vec<StoreUpgradeConfig>>,
}

struct UpgradeStackInfo {
    idx: usize,
    keyword: Rc<String>,
}

pub struct UpgradeStore {
    upgrades: Vec<UpgradeStackInfo>, // status about each upgrade stack
}

impl UpgradeStore {
    pub fn new(game: &dyn BaseGame, cfg: &StoreConfig) -> Self {
        UpgradeStore {
            upgrades: cfg.upgrades
                .iter()
                .map(|_upgr| UpgradeStackInfo {
                    idx: 0,
                    keyword: game.word_bank().get_new_word()
                })
                .collect(),
        }
    }

    pub fn draw(&self, game: &dyn BaseGame, cfg: &StoreConfig) {
        game.draw_area_background(&cfg.pos, &cfg.bg);

        let mut pos: Pos2d = (0,0).into();

        // Draw the current upgrade of each upgrade sequence
        for (cfg_upgrs, upgr_info) in cfg.upgrades.iter().zip(self.upgrades.iter()) {
            if upgr_info.idx >= cfg_upgrs.len() {
                continue;
            }

            let upgr = &cfg_upgrs[upgr_info.idx];

            let mut draw_stack = IngredientStack::new(Interpolable::new(pos, 0.0));
            draw_stack.add_ingredient(
                MovableIngredient::new(upgr.img, Interpolable::new((0, 0).into(), 0.0)),
                true,
                game);

            draw_stack.ingredients[0].image = upgr.img;
            draw_stack.overlay = Some(upgr.overlay);
            draw_stack.text = Some(upgr_info.keyword.clone());
            draw_stack.sub_text = Some(Rc::new(format!("$ {}", upgr.cost)));
            draw_stack.pos.set_cur(pos + cfg.pos);

            draw_stack.draw(game, None, Some(&cfg.text_keyword), Some(&cfg.text_price));

            pos = pos + (draw_stack.width(game) + 20.0, 0.0).into();
        }
    }

    pub fn handle_command(&mut self, keywords: &Vec<String>, upgrades: &mut Vec<StoreUpgradeConfig>, word_bank: &WordBank, game: &dyn BaseGame, cfg: &StoreConfig) {
        for keyword in keywords.iter() {
            for (cfg_upgrs, upgr_info) in cfg.upgrades.iter().zip(self.upgrades.iter_mut()) {
                if *upgr_info.keyword != **keyword {
                    continue;
                }

                let upgr = &cfg_upgrs[upgr_info.idx];

                let money = game.get_money();
                if money < upgr.cost {
                    continue;
                }

                game.add_money(-upgr.cost);
            
                upgrades.push(upgr.clone());
                upgr_info.idx += 1;
                upgr_info.keyword = word_bank.get_new_word();
            }
        }
    }

    pub fn default_config() -> StoreConfig {
        fn suc(img: Image, overlay: Image, cost: i32, action: StoreUpgradeAction) -> StoreUpgradeConfig {
            StoreUpgradeConfig {
                img: img,
                overlay: overlay,
                cost: cost,
                action: action,
            }
        }
        
        StoreConfig {
            pos: (40, 600).into(),
            bg: BackgroundConfig {
                offset: (-20, -180).into(),
                width: 2000.0,
                height: 500.0,
                corner_radius: 30.0,
                border_style: "black".to_string(),
                border_alpha: 0.3,
                border_width: 5.0,
                bg_style: "gold".to_string(),
                bg_alpha: 0.2
            },
            text_keyword: TextConfig {
                offset: (0, 0).into(),
                stroke: false,
                style: "yellow".to_string(),
                font: "comic sans".to_string(),
                size: 48,
                center_and_fit: true,
                alpha: 0.4,
                is_command: true,
            },
            text_price: TextConfig {
                offset: (0, 40).into(),
                stroke: false,
                style: "gold".to_string(),
                font: "comic sans".to_string(),
                size: 48,
                center_and_fit: true,
                alpha: 0.8,
                is_command: false,
            },
            upgrades: vec![
                vec![
                    suc(Image::BurgerBottom, Image::OverlayPlus, 10, StoreUpgradeAction::UnlockIngredient),
                ],
                vec![
                    suc(Image::BurgerTop, Image::OverlayPlus, 10, StoreUpgradeAction::UnlockIngredient),
                ],
                vec![
                    suc(Image::RawPatty, Image::OverlayPlus, 40, StoreUpgradeAction::UnlockIngredient),
                ],
                vec![
                    suc(Image::BaconRaw, Image::OverlayPlus, 30, StoreUpgradeAction::UnlockIngredient),
                ],
                vec![
                    suc(Image::EggsRaw, Image::OverlayPlus, 30, StoreUpgradeAction::UnlockIngredient),
                ],
                vec![
                    suc(Image::Flour, Image::OverlayPlus, 20, StoreUpgradeAction::UnlockIngredient),
                ],
                vec![
                    suc(Image::Curry, Image::OverlayPlus, 20, StoreUpgradeAction::UnlockIngredient),
                ],
                vec![
                    suc(Image::RawCrab, Image::OverlayPlus, 100, StoreUpgradeAction::UnlockIngredient),
                ],
                vec![
                    suc(Image::Pan, Image::OverlayPlus, 50, StoreUpgradeAction::UnlockCooker),
                    suc(Image::Pan, Image::OverlayPlus, 200, StoreUpgradeAction::UnlockCooker),
                    suc(Image::Pan, Image::OverlayPlus, 300, StoreUpgradeAction::UnlockCooker),
                ],
                vec![
                    suc(Image::TriniPot, Image::OverlayPlus, 200, StoreUpgradeAction::UnlockCooker),
                    suc(Image::TriniPot, Image::OverlayPlus, 300, StoreUpgradeAction::UnlockCooker),
                    suc(Image::TriniPot, Image::OverlayPlus, 400, StoreUpgradeAction::UnlockCooker),
                ],
            ],
        }
    }
}