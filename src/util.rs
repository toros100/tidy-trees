use crate::Tree;
use std::ops::DerefMut;

// should probably have Visit and VisitMut or something like that
pub trait Visit {
    fn visit(&mut self, t: &mut Tree);
}

pub fn visit<V: Visit>(v: &mut V, tree: &mut Tree) {
    v.visit(tree);
    for ch in tree.children.iter() {
        visit(v, ch.borrow_mut().deref_mut());
    }
}

pub fn visit_fn(tree: &mut Tree, mut f: impl FnMut(&mut Tree)) {
    fn visit_fn_inner(tree: &mut Tree, f: &mut impl FnMut(&mut Tree)) {
        f(tree);
        for c in tree.children.iter() {
            visit_fn_inner(&mut c.borrow_mut(), f);
        }
    }
    visit_fn_inner(tree, &mut f);
}

pub fn visit_postorder_fn(tree: &mut Tree, mut f: impl FnMut(&mut Tree)) {
    fn visit_postorder_fn_inner(tree: &mut Tree, f: &mut impl FnMut(&mut Tree)) {
        for c in tree.children.iter() {
            visit_postorder_fn_inner(&mut c.borrow_mut(), f);
        }
        f(tree);
    }
    visit_postorder_fn_inner(tree, &mut f);
}

#[derive(Debug)]
pub struct Bounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

pub fn get_bounds(tree: &mut Tree) -> Bounds {
    let mut bounds = Bounds::default();
    visit(&mut bounds, tree);
    bounds
}

pub fn translate(tree: &mut Tree, x: f64, y: f64) {
    visit_fn(tree, |f| {
        f.x += x;
        f.y += y;
    });
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
        }
    }
}

impl Visit for Bounds {
    fn visit(&mut self, t: &mut Tree) {
        let (tl_x, tl_y) = t.top_left();
        let (br_x, br_y) = t.bottom_right();
        self.min_x = self.min_x.min(tl_x);
        self.min_y = self.min_y.min(tl_y);
        self.max_x = self.max_x.max(br_x);
        self.max_y = self.max_y.max(br_y);
    }
}
