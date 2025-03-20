
use std::cell::RefCell;
use std::fmt;
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

#[derive(Clone, Copy, PartialEq)]
pub struct Pos2d {
    pub xpos: f64,
    pub ypos: f64,
}

impl fmt::Debug for Pos2d {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.xpos, self.ypos)
    }
}

impl Pos2d {
    pub fn new(xpos: f64, ypos: f64) -> Self {
        Pos2d{xpos: xpos, ypos: ypos}
    }
}

impl std::ops::Add<Pos2d> for Pos2d {
    type Output = Pos2d;
    fn add(self, rhs: Pos2d) -> Pos2d {
        Pos2d::new(self.xpos + rhs.xpos, self.ypos + rhs.ypos)
    }
}

impl std::ops::Sub<Pos2d> for Pos2d {
    type Output = Pos2d;
    fn sub(self, rhs: Pos2d) -> Pos2d {
        Pos2d::new(self.xpos - rhs.xpos, self.ypos - rhs.ypos)
    }
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
    base: Option<Rc<RefCell<InterpolableImp<T>>>>,
    moved_handler: Rc<RefCell<Box<dyn FnMut()>>>,
}

impl<T> InterpolableImp<T>
where T: Clone + Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> {
    fn new(val: T, speed: f64) -> Self{
        InterpolableImp {
            cur: val.clone(),
            end: val,
            speed: speed,
            base: None,
            moved_handler: Rc::new(RefCell::new(Box::new(|| {}))),
        }
    }

    fn calc_cur(&self) -> T {
        if let Some(base) = &self.base {
            return self.cur + base.borrow().calc_cur();
        }
        else {
            return self.cur;
        }
    }
}

impl<T> fmt::Debug for InterpolableImp<T>
where T: std::fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} => <{:?}->{:?}@{:?}", self.base, self.cur, self.end, self.speed)
    }
}

#[derive(Clone)]
pub struct Interpolable<T> {
    imp: Rc<RefCell<InterpolableImp<T>>>
}

impl<T> Interpolable<T>
where T: Clone + Copy + PartialEq + Advanceable<T> + std::ops::Add<Output = T> + std::ops::Sub<Output = T> {
    pub fn new (val: T, speed: f64) -> Self {
        Interpolable {
            imp: Rc::new(RefCell::new(InterpolableImp::new(val, speed))),
        }
    }

    pub fn new_b (val: T, speed: f64, base: &Interpolable<T>) -> Self {
        let ret = Interpolable {
            imp: Rc::new(RefCell::new(InterpolableImp::new(val, speed)))
        };

        ret.imp.borrow_mut().base = Some(base.imp.clone());

        ret
    }

    pub fn is_moving(&self) -> bool {
        let imp = self.imp.borrow();
        imp.cur != imp.end
    }

    /*
    pub fn set_base(&mut self, base: &Interpolable<T>) {
        self.imp.borrow_mut().base = Some(base.imp.clone());
    }
    */
    
    pub fn rebase(&mut self, new_base: &Interpolable<T>, new_offset: T, immediate: bool) {
        let cur_pos = self.cur();

        let imp = &mut *self.imp.borrow_mut();

        imp.base = Some(new_base.imp.clone());

        imp.end = new_offset;

        if immediate {
            imp.cur = new_offset;
        }
        else {
            imp.cur = cur_pos - new_base.cur();
        }
    }

    pub fn own_cur(&self) -> T {
        return self.imp.borrow().cur;
    }

    pub fn cur(&self) -> T {
        return self.imp.borrow().calc_cur();
    }

    /*
    pub fn end(&self) -> T {
        self.imp.borrow().end.clone()
    }
    */

    pub fn advance(&self, elapsed_time:f64) -> Option<Rc<RefCell<Box<dyn FnMut()>>>> {
        let imp = &mut self.imp.borrow_mut();
        if imp.cur != imp.end {
            let end = imp.end.clone();
            let speed = imp.speed;
            imp.cur.advance(end, speed, elapsed_time);
            if imp.cur == imp.end {
                return Some(imp.moved_handler.clone());
            }
        }

        None
    }

    pub fn set_cur(&self, cur: T) {
        self.imp.borrow_mut().cur = cur;
    }

    pub fn set_end(&self, end: T) {
        self.imp.borrow_mut().end = end;
    }

    /*
    pub fn set_speed(&self, speed: f64) {
        self.imp.borrow_mut().speed = speed;
    }
    */

    pub fn set_moved_handler(&self, handler: Box<dyn FnMut()>) {
        self.imp.borrow_mut().moved_handler = Rc::new(RefCell::new(handler));
    }
}

impl<T> fmt::Debug for Interpolable<T>
where T: Clone + Copy + PartialEq + Advanceable<T> + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interpolable")
         .field("imp", &self.imp)
         .field("effective_cur", &self.cur())
         .finish()
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
        let mut done_cbs: Vec<Rc<RefCell<Box<dyn FnMut()>>>> = Vec::new();

        for intr in self.interpolables_1d.iter() {
            if let Some(cb) = intr.advance(elapsed_time) {
                done_cbs.push(cb);
            }
        }
        for intr in self.interpolables_2d.iter() {
            if let Some(cb) = intr.advance(elapsed_time) {
                done_cbs.push(cb);
            }
        }

        for cb in done_cbs {
            cb.borrow_mut()();
        }
    }
}