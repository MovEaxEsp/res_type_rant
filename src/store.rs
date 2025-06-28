
use crate::ingredients::{IngredientStack, MovableIngredient};
use crate::painter::{BackgroundConfig, TextConfig};
use crate::traits::{BaseGame, Image};
use crate::utils::WordBank;

use engine_p::interpolable::{Interpolable, Pos2d};
use serde::{Serialize,Deserialize};

use std::rc::Rc;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum StoreUpgradeAction {
    UnlockIngredient,
    UnlockCooker,
    IncreaseLimit,
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
        game.painter().draw_area_background(&cfg.pos, &cfg.bg);

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

    pub fn unlock_all(&mut self, upgrades: &mut Vec<StoreUpgradeConfig>, cfg: &StoreConfig) {
        for (cfg_upgrs, upgr_info) in cfg.upgrades.iter().zip(self.upgrades.iter_mut()) {
            for upgr in cfg_upgrs {
                upgrades.push(upgr.clone());
                upgr_info.idx += 1;
            }
        }
    }
}