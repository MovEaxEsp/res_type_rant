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