use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};
mod util;
use util::*;

mod svg_draw;

mod text_draw;
use crate::{svg_draw::draw_to_svg, text_draw::layout_and_print};
use util::visit_fn;

macro_rules! tree {
    ($width:expr, $height:expr) => {
        TreeRef::new($width, $height)
    };
    ($width:expr, $height:expr; $($ch:expr),+) => {
        TreeRef::new_with_children($width, $height, vec![$($ch),+])
    };
}

fn main() {
    draw_svg_trees();
    print_trees();
}

fn draw_svg_trees() {
    let n = 50f64; // "node" size

    let opt = svg_draw::DrawOptions {
        show_bounding_boxes: true,
        ..Default::default()
    };

    let tree_0 = tree!(55.,20.; tree!(40.,40. ; tree!(30.,55.)), tree!(200.,130.; tree!(400.,20.)), tree!(55.,20.; tree!(60.,80.)));
    draw_to_svg(tree_0, "out_0.svg", opt).unwrap();

    let tree_0_mirror = tree!(55.,20.;
    tree!(55.,20.; tree!(60.,80.)),
    tree!(200.,130.; tree!(400.,20.)),
    tree!(40.,40. ; tree!(30.,55.))
    );
    draw_to_svg(tree_0_mirror, "out_0_mirror.svg", opt).unwrap();

    let tree_1 = tree!(n,n;
        tree!(n,n; tree!(n,n), tree!(n,n)),
        tree!(n,n; tree!(n,n), tree!(n,n))
    );
    draw_to_svg(tree_1, "out_1.svg", opt).unwrap();

    let tree_2 = tree!(4. * n, 2. * n; tree!(2.*n, 2.*n; tree!(n, 2.*n), tree!(n, 2.*n)), tree!(2.*n, 2.*n; tree!(n,n; tree!(n,n), tree!(n,n))), tree!(2.*n, (4./3.)*n; tree!(2.*n, (4./3.)*n; tree!(2.*n, (4./3.)*n))));
    draw_to_svg(tree_2, "out_2.svg", opt).unwrap();

    let tree_3 = tree!(n,n;
        tree!(n,n; tree!(n,n), tree!(n,n)),
        tree!(n,n; tree!(2.*n, 2.*n)),
        tree!(2.5*n, 2.*n; tree!(n,n), tree!(n,n), tree!(n,n), tree!(n,n))
    );
    draw_to_svg(tree_3, "out_3.svg", opt).unwrap();

    let tree_4 = tree!(3.*n,3.*n;
    tree!(n, 3.*n; tree!(n, n)),
    tree!(n, 2.*n; tree!(n, 2.*n; tree!(3.*n, 3.*n))),
    tree!(n, n; tree!(n, 3.*n))
    );

    draw_to_svg(tree_4, "out_4.svg", opt).unwrap();

    let tree_5 = full_k_ary_tree(3, 3, |d: usize| {
        let f = 1.8f64.powi(d as i32);
        (300f64 / f, 300f64 / f)
    });
    draw_to_svg(tree_5, "out_5.svg", opt).unwrap();

    let tree_6 = tree!(50., 50.;tree!(50., 100.;tree!(50., 100.),tree!(100., 50.),tree!(50., 50.;tree!(50., 50.;tree!(50., 50.),tree!(50., 100.),tree!(50., 50.)),
tree!(100., 50.))),tree!(50., 50.));
    draw_to_svg(tree_6, "out_6.svg", opt).unwrap();

    let tree_7 = tree!(25., 25.; tree!(100., 100.; tree!(50., 100.),tree!(50., 100.; tree!(50., 100.), tree!(50., 100.)),tree!(50., 100.)));
    draw_to_svg(tree_7, "out_7.svg", opt).unwrap();
}

fn print_trees() {
    let tree = text_tree!("hello"; text_tree!("wow"), text_tree!("what a pretty tree"));
    print_tree(tree.clone());

    let tree = text_tree!("root"; text_tree!("two\nlines"; text_tree!("o\no\no")), text_tree!("this is a big one\n\n\n\n\n\n\n\n"; text_tree!("very very very very very very long node")), text_tree!("x"; text_tree!("v\ne\nr\ny\n cool")));
    print_tree(tree);

    let tree = text_tree!("aaaa";
        text_tree!("b"),
        text_tree!("c";
            text_tree!("eeee"), text_tree!("f")), text_tree!("ddd"; text_tree!("g"))
    );
    print_tree(tree);

    let tree = text_tree!("xxxxx";
        text_tree!("xxxx";
            text_tree!("xxx";
                text_tree!("xx";
                    text_tree!("x"; 
                        text_tree!("xx";
                            text_tree!("xxx")))))));
    print_tree(tree);

    let tree = text_tree!("abcd" ; text_tree!("abc"), text_tree!("abc\nhello world\n"; text_tree!("ab"), text_tree!("ab")));
    print_tree(tree);

    let tree = text_tree!("root"; text_tree!("a"; text_tree!("aaa"; text_tree!("aaa"))), text_tree!("b"; text_tree!("bbbbbb"; text_tree!("bbbbbbbbbbbbb"))), text_tree!("c"; text_tree!("cccc")));
    print_tree(tree);

    let tree = text_tree!("a"; text_tree!("b"), text_tree!("c"), text_tree!("d"));
    print_tree(tree);

    let tree = text_tree!("a"; text_tree!("a\nb\nc"; text_tree!("j\nk\nl"), text_tree!("asdasd"), text_tree!("m"; text_tree!("t"), text_tree!("uvw uvw"))), text_tree!("a b c\nd e f\ng h i"), text_tree!("v"; text_tree!("y"), text_tree!("e\ne\ne"), text_tree!("z")));
    print_tree(tree);

    let tree = text_tree!("a"; text_tree!("a"), text_tree!("aa"), text_tree!("aaa"), text_tree!("aaaa"; text_tree!(""), text_tree!("\n")));
    print_tree(tree);
}

fn print_tree(tree: TreeRef) {
    layout_and_print(tree, text_draw::DrawOptions::default());
    println!()
}

fn full_k_ary_tree(
    k: usize,
    max_depth: usize,
    mut node_size: impl FnMut(usize) -> (f64, f64),
) -> TreeRef {
    fn full_k_ary_tree_inner(
        k: usize,
        depth: usize,
        max_depth: usize,
        node_size: &mut impl FnMut(usize) -> (f64, f64),
    ) -> TreeRef {
        assert!(k > 0);

        let (width, height) = node_size(depth);

        if depth >= max_depth {
            return TreeRef::new(width, height);
        }
        let mut ch = Vec::<TreeRef>::with_capacity(k);

        for _ in 0..k {
            ch.push(full_k_ary_tree_inner(k, depth + 1, max_depth, node_size));
        }
        TreeRef::new_with_children(width, height, ch)
    }
    full_k_ary_tree_inner(k, 0, max_depth, &mut node_size)
}

// i know
// (i wanted to get it to work first, then make it not awful)
#[derive(Default, Clone)]
struct Tree {
    width: f64,
    height: f64,
    x: f64,
    y: f64,
    prelim: f64,
    modifier: f64,
    shift: f64,
    change: f64,
    thread_left: Option<TreeRef>,
    thread_right: Option<TreeRef>,
    extreme_left: Option<WeakTreeRef>,
    extreme_right: Option<WeakTreeRef>,
    msel: f64,
    mser: f64,
    children: Vec<TreeRef>,
    content: String,
}

#[derive(Clone)]
struct TreeRef(Rc<RefCell<Tree>>);

#[derive(Clone)]
struct WeakTreeRef(Weak<RefCell<Tree>>);

impl Deref for WeakTreeRef {
    type Target = Weak<RefCell<Tree>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for TreeRef {
    type Target = RefCell<Tree>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Tree> for TreeRef {
    fn from(value: Tree) -> Self {
        TreeRef(Rc::new(RefCell::new(value)))
    }
}

impl TreeRef {
    fn downgrade(&self) -> WeakTreeRef {
        WeakTreeRef(Rc::downgrade(&self.0))
    }

    fn new(width: f64, height: f64) -> Self {
        debug_assert!(width >= 1.);
        debug_assert!(height >= 1.);
        Tree {
            width,
            height,
            ..Default::default()
        }
        .into()
    }

    fn new_with_children(width: f64, height: f64, children: Vec<TreeRef>) -> Self {
        debug_assert!(width >= 1.);
        debug_assert!(height >= 1.);
        Tree {
            width,
            height,
            children,
            ..Default::default()
        }
        .into()
    }

    fn layout(&self) {
        self.first_walk();
        self.borrow_mut().second_walk(0f64);
    }

    fn first_walk(&self) {
        if self.borrow().children.is_empty() {
            self.set_extremes();
        } else {
            let self_borrow = self.borrow();
            let s = &self_borrow.children[0];
            s.first_walk();

            let mut lookup = LeftSiblingLookup::default();

            // lol, lmao even
            let min_y = s
                .borrow()
                .extreme_left
                .as_ref()
                .unwrap()
                .upgrade()
                .unwrap()
                .borrow()
                .bottom();

            lookup.update(min_y, 0);

            for i in 1..self_borrow.children.len() {
                let c = &self_borrow.children[i];
                c.first_walk();

                let min_y = c
                    .borrow()
                    .extreme_right
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .borrow()
                    .bottom();
                self.separate(i, lookup.iter());
                lookup.update(min_y, i);
            }

            drop(self_borrow);

            self.borrow_mut().position_root();
            self.set_extremes();
        }
    }

    fn separate(&self, i: usize, mut it: LeftSiblingLookupIterator) {
        debug_assert!(i > 0);

        let l_sib = self.borrow().children[i - 1].clone();
        let mut r_mod_sum = l_sib.borrow().modifier;
        let current_subtree = self.borrow().children[i].clone();
        let mut l_mod_sum = current_subtree.borrow().modifier;

        let mut r_contour = Some(l_sib);
        let mut l_contour = Some(current_subtree);

        let mut first = false;
        while let Some(r_contour_ref) = r_contour.as_ref()
            && let Some(l_contour_ref) = l_contour.as_ref()
        {
            debug_assert_ne!(it.len(), 0);

            let sib = it.peek().unwrap();

            if r_contour_ref.borrow().bottom() > sib.lowest_y {
                _ = it.next();
                debug_assert_ne!(it.len(), 0);
            }

            let dist = (r_mod_sum + r_contour_ref.borrow().prelim)
                - (l_mod_sum + l_contour_ref.borrow().prelim)
                + r_contour_ref.borrow().width / 2.0
                + l_contour_ref.borrow().width / 2.0;

            if first && dist < 0. || dist > 0f64 {
                l_mod_sum += dist;
                self.borrow()
                    .move_subtree(i, it.peek().unwrap().index, dist);
            }
            first = false;

            let r_bottom = r_contour_ref.borrow().bottom();
            let l_bottom = l_contour_ref.borrow().bottom();

            if r_bottom <= l_bottom {
                r_contour = {
                    let b = r_contour_ref.borrow();
                    b.next_right_contour()
                };
                if let Some(ref sr) = r_contour {
                    r_mod_sum += sr.borrow().modifier;
                }
            }
            if r_bottom >= l_bottom {
                l_contour = {
                    let b = l_contour_ref.borrow();
                    b.next_left_contour()
                };

                if let Some(ref cl) = l_contour {
                    l_mod_sum += cl.borrow().modifier;
                }
            }

            if r_contour.is_none()
                && let Some(ref cl) = l_contour
            {
                self.borrow().set_left_thread(i, cl.clone(), l_mod_sum);
            } else if let Some(ref sr) = r_contour
                && l_contour.is_none()
            {
                self.borrow().set_right_thread(i, sr.clone(), r_mod_sum);
            }
        }
    }

    /// very icky: needs to be able to store self (or at least some kind of ref to self)
    fn set_extremes(&self) {
        let mut s = self.borrow_mut();
        if s.children.is_empty() {
            s.extreme_left = Some(self.downgrade());
            s.extreme_right = Some(self.downgrade());
            s.msel = 0f64;
            s.mser = 0f64;
        } else {
            let first_child = s.children[0].clone();
            s.extreme_left = first_child.borrow().extreme_left.clone();
            debug_assert!(s.extreme_left.is_some());
            s.msel = first_child.borrow().msel;
            let last_child = s.children[s.children.len() - 1].clone();
            s.extreme_right = last_child.borrow().extreme_right.clone();
            debug_assert!(s.extreme_right.is_some());
            s.mser = last_child.borrow().mser;
        }
    }
}

impl Tree {
    /// pre-order traversal mutating each node
    fn second_walk(&mut self, mut mod_sum: f64) {
        mod_sum += self.modifier;
        self.x = self.prelim + mod_sum;
        self.add_child_spacing();

        for ch in self.children.iter_mut() {
            ch.borrow_mut().second_walk(mod_sum);
        }
    }

    /// interior mutation on all children
    fn add_child_spacing(&self) {
        let mut d = 0f64;
        let mut mod_sum_delta = 0f64;
        for c in self.children.iter() {
            let mut c_mut = c.borrow_mut();
            d += c_mut.shift;
            mod_sum_delta += d + c_mut.change;
            c_mut.modifier += mod_sum_delta;
        }
    }

    /// interior mutation on 2 children
    fn distribute_extra(&self, i: usize, si: usize, dist: f64) {
        debug_assert!(si <= i);

        let n = i - si;
        if n > 1 {
            let v = dist / (n as f64);

            self.children[si + 1].borrow_mut().shift += v;
            self.children[i].borrow_mut().shift -= v;
            self.children[i].borrow_mut().change -= dist - v;
        }
    }

    fn next_left_contour(&self) -> Option<TreeRef> {
        if self.children.is_empty() {
            self.thread_left.clone()
        } else {
            Some(self.children[0].clone())
        }
    }

    fn next_right_contour(&self) -> Option<TreeRef> {
        if self.children.is_empty() {
            self.thread_right.clone()
        } else {
            Some(self.children[self.children.len() - 1].clone())
        }
    }

    /// only mutates self
    fn position_root(&mut self) {
        let first_ch = self.children[0].borrow();
        let last_ch = self.children[self.children.len() - 1].borrow();

        self.prelim = (first_ch.modifier + first_ch.prelim - first_ch.width / 2.0
            + last_ch.modifier
            + last_ch.prelim
            + last_ch.width / 2.0)
            / 2.0;
    }

    /// mutates  children
    fn move_subtree(&self, i: usize, si: usize, dist: f64) {
        let ith_child = self.children[i].clone();
        ith_child.borrow_mut().modifier += dist;
        ith_child.borrow_mut().msel += dist;
        ith_child.borrow_mut().mser += dist;
        self.distribute_extra(i, si, dist);
    }

    /// icky: mutates "through" extreme_left of a child
    fn set_left_thread(&self, i: usize, cl: TreeRef, mscl: f64) {
        let li = (self.children[0])
            .borrow()
            .extreme_left
            .clone()
            .expect("should be present")
            .upgrade()
            .unwrap();

        li.borrow_mut().thread_left = Some(cl.clone());

        let diff = (mscl - cl.borrow().modifier) - self.children[0].borrow().msel;
        li.borrow_mut().modifier += diff;
        li.borrow_mut().prelim -= diff;
        let ith_el = self.children[i].borrow().extreme_left.clone();
        self.children[0].borrow_mut().extreme_left = ith_el;
        let ith_msel = self.children[i].borrow().msel;
        self.children[0].borrow_mut().msel = ith_msel;
    }

    /// icky: mutates "through" extreme_right of a child
    fn set_right_thread(&self, i: usize, sr: TreeRef, mssr: f64) {
        let ri = (self.children[i])
            .borrow()
            .extreme_right
            .clone()
            .expect("should be present")
            .upgrade()
            .unwrap();
        ri.borrow_mut().thread_right = Some(sr.clone());

        let diff = (mssr - sr.borrow().modifier) - self.children[i].borrow().mser;
        ri.borrow_mut().modifier += diff;
        ri.borrow_mut().prelim -= diff;
        let i_minus_1_er = self.children[i - 1].borrow().extreme_right.clone();
        self.children[i].borrow_mut().extreme_right = i_minus_1_er;
        let i_minus_1_mser = self.children[i - 1].borrow().mser;
        self.children[i].borrow_mut().mser = i_minus_1_mser;
    }

    fn center_top(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    fn center_bottom(&self) -> (f64, f64) {
        (self.x, self.y + self.height)
    }

    fn top_left(&self) -> (f64, f64) {
        (self.x - self.width / 2., self.y)
    }

    fn bottom_right(&self) -> (f64, f64) {
        (self.x + self.width / 2., self.y + self.height)
    }

    fn bottom(&self) -> f64 {
        self.y + self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LeftSibling {
    lowest_y: f64, // y increases downwards, so "lowest y" is largest by value
    index: usize,
}

// original paper uses some unhinged linked list
// i wanted to keep the first implementation as structurally close to the paper version as possible,
// but i have to draw a line somewhere
#[derive(Default, Debug)]
struct LeftSiblingLookup(Vec<LeftSibling>);

impl LeftSiblingLookup {
    fn iter(&self) -> LeftSiblingLookupIterator<'_> {
        self.into_iter()
    }

    fn update(&mut self, low_y: f64, index: usize) {
        let l = self.0.partition_point(|p| low_y < p.lowest_y);

        if l < self.0.len() {
            self.0[l] = LeftSibling {
                lowest_y: low_y,
                index,
            };
            self.0.truncate(l + 1);
        } else {
            self.0.push(LeftSibling {
                lowest_y: low_y,
                index,
            });
        }
    }
}

struct LeftSiblingLookupIterator<'a> {
    lookup: &'a LeftSiblingLookup,
    pos: usize,
}

impl<'a> IntoIterator for &'a LeftSiblingLookup {
    type Item = LeftSibling;
    type IntoIter = LeftSiblingLookupIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        LeftSiblingLookupIterator {
            lookup: self,
            pos: self.0.len(),
        }
    }
}

impl LeftSiblingLookupIterator<'_> {
    fn peek(&self) -> Option<<LeftSiblingLookupIterator<'_> as Iterator>::Item> {
        if self.is_empty() {
            None
        } else {
            Some(self.lookup.0[self.pos - 1])
        }
    }
    fn is_empty(&self) -> bool {
        self.pos == 0
    }
}

impl<'a> Iterator for LeftSiblingLookupIterator<'a> {
    type Item = LeftSibling;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == 0 {
            None
        } else {
            let res = Some(self.lookup.0[self.pos - 1]);
            self.pos -= 1;
            res
        }
    }
}

impl<'a> ExactSizeIterator for LeftSiblingLookupIterator<'a> {
    fn len(&self) -> usize {
        self.pos
    }
}
