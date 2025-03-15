
use crate::traits::{Image, BaseGame};
use crate::interpolable::{Interpolable, Pos2d, InterpolableStore};


pub struct MovableIngredient {
    pub image: Image,
    pub pos: Interpolable<Pos2d>,
}

impl MovableIngredient {
    pub fn new(image: Image, xpos: f64, ypos: f64, speed: f64) -> Self {
        MovableIngredient {
            image: image,
            pos: Interpolable::<Pos2d>::new(Pos2d{xpos, ypos}, speed),
        }
    }
    
    pub fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        dest.interpolables_2d.push(self.pos.clone());
    }

    pub fn draw(&self, base_pos: &Pos2d, state: &dyn BaseGame) {
        let pos = self.pos.cur();
        state.draw_image(&self.image,
                         base_pos.xpos + pos.xpos,
                         base_pos.ypos + pos.ypos);
    }
}

///////// IngredientStack
pub struct IngredientStack {
    pub ingredients: Vec<MovableIngredient>,
    pub pos: Interpolable<Pos2d>,
}

impl IngredientStack {
    pub fn new(xpos: f64, ypos: f64) -> Self {
        IngredientStack {
            ingredients: Vec::new(),
            pos: Interpolable::<Pos2d>::new(Pos2d{xpos: xpos, ypos: ypos}, 500.0),
        }
    }

    pub fn height() -> f64 {
        150.0
    }

    pub fn collect_interpolables(&self, dest: &mut InterpolableStore) {
        dest.interpolables_2d.push(self.pos.clone());
        for item in self.ingredients.iter() {
            item.collect_interpolables(dest);
        }
    }

    pub fn draw(&self, state: &dyn BaseGame) {
        for item in self.ingredients.iter() {
            item.draw(&self.pos.cur(), state);
        }
    }

    pub fn add_ingredient(&mut self, ingredient: MovableIngredient, cur_base_pos: &Pos2d, immediate: bool) {
        let end = Pos2d {
            xpos: 0.0,
            ypos: IngredientStack::height() - (((self.ingredients.len()+1) as f64) *35.0)
        };

        let my_base = self.pos.cur();

        let cur_pos = ingredient.pos.cur();
        let cur_x_transformed = cur_base_pos.xpos + cur_pos.xpos - my_base.xpos;
        let cur_y_transformed = cur_base_pos.ypos + cur_pos.ypos - my_base.ypos;

        if immediate {
            ingredient.pos.set_cur(end.clone());
        }
        else {
            ingredient.pos.set_cur(Pos2d{xpos: cur_x_transformed, ypos: cur_y_transformed});
        }
        ingredient.pos.set_end(end);

        self.ingredients.push(ingredient);
    }
}
