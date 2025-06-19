use itertools::Itertools;
use std::collections::HashSet;
use std::rc::Rc;
use js_sys::Math;
use wasm_bindgen::prelude::*;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub struct WordBank {
    words: Vec<Rc<String>>,
}

impl WordBank {
    // Create a WordBank with words at the specified 'word_level' using the specified 'words_db'
    // and excluding any words in the specified 'bad_words_db'. 'words_db' 
    // contains a word per line, along with a 'frequency' count of how often that
    // word is used.
    pub fn new(words_db: &String, bad_words_db: &String, word_level: usize) -> Self {
        if word_level == 0 {
            // Level 0 is just 2-letter combinations of characters
            return WordBank {
                words: ('a'..'z').cartesian_product('a'..'z')
                       .map(|e| Rc::new([e.0, e.1].iter().collect()))
                       .collect()
            };
        }

        let mut bad_words: HashSet<String> = HashSet::new();
        for line in bad_words_db.split('\n') {
            if line.contains(' ') {
                continue; // ignore multi-word lines
            }

            bad_words.insert(line.to_string());
        }

        let mut processed_words= 0;
        let mut ret:Vec<Rc<String>> = Vec::new();
        for line in words_db.split('\n') {
            if line.len() == 0 {
                continue;
            }
            let mut line_iter = line.split(' ');
            let word = line_iter.next().unwrap();

            let mut skip_word = false;
            for char in word.chars() {
                if char  < 'a' || char > 'z' {
                    skip_word = true;
                    break;
                }
            }

            if skip_word {
                continue;
            }

            if word.chars().nth(0).unwrap() == '\'' ||  // skip entries starting with apostrophe
            word.len() == 1 || // skip single character "words"
            bad_words.contains(word) // skip bad words
            { 
                continue;
            }

            processed_words += 1;

            // Figure out the word's difficulty, from 1(easiest) to 5(hardest)
            // for 4+ chars, difficulty is 'len - 4'
            // words within 70% of max_count get +0, 50% +1, 30% +2, <30% +3

            let mut word_score = processed_words/20000 + 1;
            if word.len() > 4 {
                word_score += word.len() - 4;
            }

            if word_score > 5 {
                word_score = 5;
            }

            if word_score == word_level {
                ret.push(Rc::new(word.to_string()));
            }
        }

        WordBank {
            words: ret,
        }
    }

    pub fn get_new_word(&self) -> Rc<String> {
        let mut idx: usize;
        loop {
            idx = (Math::random() *(self.words.len() as f64)) as usize;
            //idx = rng.gen_range(0, self.words.len());
            if Rc::strong_count(&self.words[idx]) == 1 {
                break;
            }
        }

        return self.words[idx].clone();
    }
}


/////// DEAD CODE

///////////// BaseDrawable

/*
struct BaseDrawable {
    image: Image,
    xpos: f64,
    ypos: f64,
    std_width: f64,
    std_height: f64,
    scale: f64,
}

impl BaseDrawable {
    fn new(image: Image, xpos: i32, ypos: i32, std_width: i32, std_height: i32, scale: f64) -> Self{
        BaseDrawable {
            image: image,
            xpos: xpos.into(),
            ypos: ypos.into(),
            std_width: std_width.into(),
            std_height: std_height.into(),
            scale: scale,
        }
    }

    fn burger_top(xpos: i32, ypos: i32, scale: f64) -> Self {
        BaseDrawable::new(Image::BurgerTop, xpos, ypos, 100, 30, scale)
    }
    fn burger_bottom(xpos: i32, ypos: i32, scale: f64) -> Self {
        BaseDrawable::new(Image::BurgerBottom, xpos, ypos, 100, 30, scale)
    }
}

impl Entity for BaseDrawable {
    fn draw(&self, game: &GameState) {
        game.canvas.draw_image_with_html_image_element_and_dw_and_dh(
            &game.images.get(&self.image).unwrap().image,
            self.xpos,
            self.ypos,
            self.std_width*self.scale,
            self.std_height*self.scale
        )
        .expect("draw");
    }
}

*/


/*
//////////// DrawableStack

struct DrawableStack {
    drawables: Vec<Image>,
    xpos: f64,
    ypos: f64,
    std_width: f64,
    std_height: f64,
    scale: f64,
    margin: f64,
}

impl DrawableStack {
    fn pushDrawable(&mut self, drawable: Image) {
        self.drawables.push(drawable);
        
        let mut totalHeight = 0.0;
        for d in self.drawables.iter() {
            totalHeight += d.std_height;
        }

        self.margin = (self.std_height - totalHeight)/((self.drawables.len() as f64) + 1.0); 

        let mut cur_y = self.margin;
        for d in self.drawables.iter_mut(){
            d.xpos = self.xpos;
            d.ypos = cur_y;
            cur_y += d.std_height + self.margin;
        }
    }
}

impl Entity for DrawableStack {
    fn draw(&self, state: &GameState) {
        for d in self.drawables.iter() {
            d.draw(state);
        }
    }
}
*/




/* spinning overlapping ingredients


        // Testing clipping
        let c = &self.canvas;
        let side = 200.0;
        let x = 500.0;
        let y = 300.0;
        let pi = f64::consts::PI;
        let cx = x+side/2.0;
        let cy = y+side/2.0;

        if !self.spin_start.is_moving() {
            self.spin_start.set_end(2.0*pi);
            self.spin_start.set_cur(0.0);
        }

        self.spin_start.advance(self.elapsed_time);
        let mut arc_pos = self.spin_start.cur();

        let turn_amt = (pi*2.0)/6.0;

        let mut draw = |img: Image, color: &str| {



            c.save();
            c.begin_path();
            c.arc(cx, cy, side/2.0, arc_pos, arc_pos+turn_amt).expect("f");
            c.line_to(cx, cy);
            c.close_path();
            arc_pos += turn_amt;
            c.set_fill_style_str(color);
            //c.fill();
            //c.stroke();
            c.clip();


            let props = self.images.get(&img).unwrap();

            let img_width = props.width*2.0;
            let img_height = props.height*4.0;
            self.canvas.draw_image_with_html_image_element_and_dw_and_dh(
                &props.image,
                x + (side-img_width)/2.0,
                y + (side-img_height)/2.0,
                img_width,
                img_height
            )
            .expect("draw");
        
            c.restore();

            let line_x = cx + (side/2.0)*arc_pos.cos();
            let line_y = cy + (side/2.0)*arc_pos.sin();
            c.begin_path();
            c.move_to(cx, cy);
            c.line_to(line_x, line_y);
            c.stroke();
        };

        //c.save();

        draw(Image::BurgerBottom, "red");
        draw(Image::CookedPatty, "green");
        draw(Image::LettuceLeaf, "blue");
        draw(Image::BurgerTop, "yellow");
        draw(Image::RawPatty, "yellow");
        draw(Image::TomatoSlice, "yellow");

        //c.restore();


*/ 