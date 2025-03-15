
use std::cell::RefCell;
use std::rc::Rc;

pub trait Advanceable<T> {
    fn advance(&mut self, end: T, speed: f64, elapsed_time: f64);
}

impl Advanceable<f64> for f64 {
    fn advance(self:&mut f64, end: f64, speed: f64, elapsed_time: f64) {
        if *self < end {
            let move_amt = speed*elapsed_time;
            if *self + move_amt > end {
                *self = end;
            }
            else {
                *self += move_amt;
            }
        }
        else if *self > end {
            let move_amt = speed*elapsed_time;
            if *self - move_amt < end {
                *self = end;
            }
            else {
                *self -= move_amt;
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Pos2d {
    pub xpos: f64,
    pub ypos: f64,
}

impl Advanceable<Pos2d> for Pos2d {
    fn advance(self:&mut Pos2d, end: Pos2d, speed: f64, elapsed_time: f64) {
        let x_diff = end.xpos - self.xpos;
        let y_diff = end.ypos - self.ypos;
        let dist = ((x_diff.powf(2.0) + y_diff).powf(2.0)).sqrt();
        let move_amt = speed*elapsed_time;

        if dist < move_amt {
            *self = end;
        }
        else {
            let x_prop = x_diff.abs() / (x_diff.abs() + y_diff.abs());
            let x_move_amt = f64::min(x_diff.abs(), move_amt * x_prop);
            if x_diff < 0.0 {
                self.xpos -= x_move_amt;
            }
            else if x_diff > 0.0 {
                self.xpos += x_move_amt;
            }

            let y_move_amt = f64::min(y_diff.abs(), move_amt * (1.0-x_prop));
            if y_diff < 0.0 {
                self.ypos -= y_move_amt;
            }
            else if y_diff > 0.0 {
                self.ypos += y_move_amt;
            }
        }
    }
}

struct InterpolableImp<T> {
    cur: T,
    end: T,
    speed: f64, // units per second
    moved_handler: RefCell<Box<dyn FnMut()>>,
}

impl<T> InterpolableImp<T>
where T: Clone {
    fn new(val: T, speed: f64) -> Self{
        InterpolableImp {
            cur: val.clone(),
            end: val,
            speed: speed,
            moved_handler: RefCell::new(Box::new(|| {})),
        }
    }
}

#[derive(Clone)]
pub struct Interpolable<T> {
    imp: Rc<RefCell<InterpolableImp<T>>>,
}

impl<T> Interpolable<T>
where T: Clone + PartialEq + Advanceable<T> {
    pub fn new (val: T, speed: f64) -> Self {
        Interpolable {
            imp: Rc::new(RefCell::new(InterpolableImp::new(val, speed)))
        }
    }

    pub fn is_moving(&self) -> bool {
        let imp = self.imp.borrow();
        imp.cur != imp.end
    }

    pub fn cur(&self) -> T {
        self.imp.borrow().cur.clone()
    }

    /*
    pub fn end(&self) -> T {
        self.imp.borrow().end.clone()
    }
    */

    pub fn advance(&self, elapsed_time:f64) {
        let mut done_cb: Box<dyn FnMut()> = Box::new(||{});
        let mut have_done_cb = false;

        {
            let mut imp = self.imp.borrow_mut();
            if imp.cur != imp.end {
                let end = imp.end.clone();
                let speed = imp.speed;
                imp.cur.advance(end, speed, elapsed_time);
                if imp.cur == imp.end {
                    done_cb = imp.moved_handler.replace(done_cb);
                    have_done_cb = true;
                }
            }
        }

        if have_done_cb {
            done_cb();

            let imp = self.imp.borrow_mut();
            imp.moved_handler.replace(done_cb);
        }
    }

    pub fn set_cur(&self, cur: T) {
        self.imp.borrow_mut().cur = cur;
    }

    pub fn set_end(&self, end: T) {
        self.imp.borrow_mut().end = end;
    }

    pub fn set_moved_handler(&self, handler: Box<dyn FnMut()>) {
        self.imp.borrow_mut().moved_handler = RefCell::new(handler);
    }
}

pub struct InterpolableStore {
    pub interpolables_1d: Vec<Interpolable<f64>>,
    pub interpolables_2d: Vec<Interpolable<Pos2d>>,
}

impl InterpolableStore {
    pub fn new() -> Self {
        InterpolableStore {
            interpolables_1d: Vec::new(),
            interpolables_2d: Vec::new(),
        }
    }

    pub fn advance_all(&self, elapsed_time: f64) {
        for intr in self.interpolables_1d.iter() {
            intr.advance(elapsed_time);
        }
        for intr in self.interpolables_2d.iter() {
            intr.advance(elapsed_time);
        }
    }
}