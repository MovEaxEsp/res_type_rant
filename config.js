
// Helper functions for generating sub-configs
function pos(x,y) {
    return {x, y};
}

function bgCfg(x, y, width, height, border_style, bg_style, args = {}) {
    return {
        ...{
            offset: {x, y},
            width,
            height,
            corner_radius: 30.0,
            border_style,
            border_alpha: 1.0,
            border_width: 5.0,
            bg_style,
            bg_alpha: 0.2
        },
        ...args};
};

function textCfg(x, y, size, args = {}) {
    return {
        ...{
            offset: {x, y},
            font: "comic sans",
            style: "yellow",
            stroke: false,
            size,
            center_and_fit: false, 
            alpha: .4,
            is_command: false },
        ...args };
}

function progressCfg(x, y, width, height) {
    return {
        done_alpha: 1.0,
        done_style: "yellow",
        bg: bgCfg(x, y, width, height, "black", "black", 
                  { corner_radius: 5, border_alpha: 0, border_width: 0, bg_alpha: .4}),
    };
}

function playbackCfg(sound, args={}) {
    return {
        ...{
            sound,
            play_length: null,
            random_start: false
        },
        ...args};
}

function genUiConfig(args) {
    // Images
    const img = (image, image_name, width, height) => ({image, image_name, width, height});
    const images = {
        scale: 1.0,
        images: [
            img("BaconCooked",    "bacon_cooked.png",     100.0, 70.0 ),
            img("BaconRaw",       "bacon_raw.png",        100.0, 60.0 ),
            img("BurgerBottom",   "burger_bottom.png",    100.0, 30.0 ),
            img("BurgerTop",      "burger_top.png",       100.0, 30.0 ),
            img("ClosedSign",     "closed_sign.png",      300.0, 200.0 ),
            img("CookedPatty",    "cooked_patty.png",     100.0, 30.0 ),
            img("Curry",          "curry.png",            100.0, 140.0 ),
            img("CurryCrab",      "curry_crab.png",       150.0, 100.0 ),
            img("Dumplings",      "dumplings.png",        100.0, 60.0 ),
            img("EggsFried",      "eggs_fried.png",       100.0, 70.0 ),
            img("EggsRaw",        "eggs_raw.png",         100.0, 60.0 ),
            img("Flour",          "flour.png",            100.0, 100.0 ),
            img("LettuceLeaf",    "lettuce_leaf.png",     100.0, 30.0 ),
            img("MoneyBag",       "money_bag.png",        100.0, 120.0 ),
            img("OpenSign",       "open_sign.png",        300.0, 200.0 ),
            img("OverlayArrowUp", "overlay_arrow_up.png", 40.0, 40.0 ),
            img("OverlayPlus",    "overlay_plus.png",     40.0, 40.0 ),
            img("Pan",            "pan.png",              200.0, 30.0 ),
            img("Plate",          "plate.png",            100.0, 30.0 ),
            img("RawCrab",        "raw_crab.png",         100.0, 60.0 ),
            img("RawPatty",       "raw_patty.png",        100.0, 30.0 ),
            img("TomatoSlice",    "tomato_slice.png",     100.0, 30.0 ),
            img("TriniPot",       "trini_pot.png",        180.0, 100.0 ),
        ]
    };

    // Sounds
    const snd = (sound, sound_names) => ({sound, sound_names});
    const sounds = {
        sounds: [
            snd("Coins", ["coins_1.mp3", "coins_2.mp3", "coins_3.mp3"]),
            snd("Frying", ["frying_1.mp3"]),
            snd("Done", ["done_1.mp3"]),
        ]
    };

    // Order Bar
    const ordIng = (ing, chance, price) => ({ing, chance, price});
    const orderCfg = (weight, depreciation_seconds, ings) => ({weight, depreciation_seconds, ings});
    const order_bar = {
        pos: pos(1200, 400),
        order_margin: 20,
        bg: bgCfg(-50, -300, 1340, 500, "black", "pink"),
        text_price: textCfg(0, 40, 48, {
            center_and_fit: true}),
        text_keyword: textCfg(0, 100, 48, {
            center_and_fit: true, is_command: true}),
        text_remaining: textCfg(10, -270, 48, {
            style: "white"}),
        progress_bar: progressCfg(0, 30, 100, 5),
        money_sound: playbackCfg("Coins"),
        orders: [
            orderCfg(1, 5, [ // Burger
                ordIng("BurgerBottom", 1, 3),
                ordIng("CookedPatty", 1, 8),
                ordIng("LettuceLeaf", .7, 4),
                ordIng("TomatoSlice", .6, 5),
                ordIng("BurgerTop", 1, 3)]),
            orderCfg(.5, 5, [ // Salad
                ordIng("LettuceLeaf", 1, 8),
                ordIng("TomatoSlice", 1, 10)]),
            orderCfg(.5, 8, [ // Curry Crab
                ordIng("CurryCrab", 1, 30),
                ordIng("Dumplings", 1, 10)]),
            orderCfg(1, 8, [ // Egg Sandwich
                ordIng("BurgerBottom", 1, 5),
                ordIng("EggsFried", 1, 7),
                ordIng("BaconCooked", .3, 8),
                ordIng("BurgerTop", 1, 5) ]),
            orderCfg(1, 8, [ // Bacon Sandwich
                ordIng("BurgerBottom", 1, 5),
                ordIng("BaconCooked", 1, 8),
                ordIng("LettuceLeaf", .8, 3),
                ordIng("TomatoSlice", .7, 4),
                ordIng("BurgerTop", 1, 5)]),
        ]
    }

    // Ingredient Area
    const ingredient_area = {
        pos: pos(80, 800),
        grid_width: 5,
        grid_item_width: 170,
        grid_item_height: 200,
        bg: bgCfg(-50, -150, 900, 500, "black", "orange", {
            border_alpha: .3, border_width: 5}),
        text: textCfg(0, 0, 48, {
            center_and_fit: true, is_command: true }),
    };

    // Preparation Area
    const cookerCfg = (base_image, base_offset, cooking_sound, done_cooking_sound, recipes, instances) => 
        ({base_image, base_offset, cooking_sound, done_cooking_sound, recipes, instances, num_unlocked: 0});
    const recipe = (inputs, outputs, cook_time) => ({inputs, outputs, cook_time});
    const preparation_area = {
        pos: pos(1200, 800),
        bg: bgCfg(-50, -70, 1300, 700, "black", "orange", {
            border_alpha: 0.3 }),
        text: textCfg(0, 0, 48, {
            center_and_fit: true, is_command: true }),
        progress: progressCfg(0, 30, 100, 5),
        cookers:[
            cookerCfg("Pan",
                      pos(-10, 10),
                      playbackCfg("Frying", {random_start: true}),
                      playbackCfg("Done"),
                      [
                        recipe(["RawPatty"], ["CookedPatty"], 10),
                        recipe(["EggsRaw"], ["EggsFried"], 6),
                        recipe(["BaconRaw"], ["BaconCooked"], 8),
                      ],
                    [pos(0,100), pos(300, 100), pos(600, 100)]),
            cookerCfg("TriniPot",
                      pos(0, 10),
                      playbackCfg("Frying", {random_start: true}),
                      playbackCfg("Done"),
                      [
                        recipe(["RawCrab", "Curry"], ["CurryCrab"], 15),
                        recipe(["Flour"], ["Dumplings"], 5),
                      ],
                      [pos(0, 550), pos(300, 550), pos(600, 550)]),
        ]
    };
    
    // Store
    const ingUpgr = (img, cost) => ({img, cost, overlay: "OverlayPlus", action: "UnlockIngredient"});
    const cookerUpgr = (img, cost) => ({img, cost, overlay: "OverlayPlus", action: "UnlockCooker"});
    const limitUpgr = (img, cost) => ({img, cost, overlay: "OverlayArrowUp", action: "IncreaseLimit"});
    const store = {
        pos: pos(40, 600),
        bg: bgCfg(-20, -180, 2000, 500, "black", "gold", {
            border_alpha: .3 }),
        text_keyword: textCfg(0, 0, 48, {
            center_and_fit: true, is_command: true }),
        text_price: textCfg(0, 40, 48, {
            style: "gold", center_and_fit: true }),
        upgrades: [
            [ingUpgr("BurgerBottom", 10)],
            [ingUpgr("BurgerTop", 10)],
            [ingUpgr("RawPatty", 40)],
            [ingUpgr("BaconRaw", 30)],
            [ingUpgr("EggsRaw", 30)],
            [ingUpgr("Flour", 20)],
            [ingUpgr("Curry", 20)],
            [ingUpgr("RawCrab", 100)],
            [cookerUpgr("Pan", 50), cookerUpgr("Pan", 200), cookerUpgr("Pan", 300)],
            [cookerUpgr("TriniPot", 200), cookerUpgr("TriniPot", 300), cookerUpgr("TriniPot", 400)],
            [limitUpgr("MoneyBag", 80), limitUpgr("MoneyBag", 180), limitUpgr("MoneyBag",380)],
        ]
    };

    // Keyword Entry
    const keyword_entry = {
        pos: pos(20, 1300),
        caret_speed: 3,
        bg: bgCfg(-10, -25, 1000, 100, "black", "white", {
            border_alpha: .3, border_width: 5, bg_alpha: .8 }),
        text: textCfg(0,0, 48, {
            style: "black", alpha: 1 }),
    };
    
    // State
    const state = {
        pos: pos(650, 250),
        bg: bgCfg(-50, -70, 500, 500, "black", "orange", {
            birder_alpha: .3 }),
        clock_r1: 150,
        clock_r2: 50,
        text: textCfg(0, 0, 48, {
            center_and_fit: true, is_command: true }),
        progress: progressCfg(0, 0, 200, 5),
    };

    // Money
    const money = {
        pos: pos(50, 50),
        bg: bgCfg(0, -20, 400, 250, "black", "green", {
            border_alpha: .3 }),
        text: textCfg(40, 40, 128, {
            style: "black", filled_style: "gold", stroke: true, alpha: 1 }),
    };
    
    // FPS
    const fps = textCfg(0, 0, 30, {
            style: "black", alpha: .7 });

    return {images, sounds, order_bar, ingredient_area, preparation_area, store, keyword_entry, state, money, fps};
}

function genGameConfig(args) {
    // Ingredient area
    const ingredient_area = {
        ingredients: ["LettuceLeaf", "TomatoSlice"],
    };
    
    // Order Bar
    const order_bar = {
        order_period: 6,
    };
    
    // State
    const state = {
        day_length: 90,
        money_down_sec: 3,
        money_down_amt: -1,
    };
    
    // Money
    const money = {
        starting_money: 0,
        max_money: 100,
    };
    
    return {
        word_level: 0,
        unlock_all: false,
        ingredient_area, order_bar, state, money
    };
}

export function genConfig(args) {
    return {
        ui: genUiConfig(args),
        game: genGameConfig(args),
    };
}