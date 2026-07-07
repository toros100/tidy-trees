use crate::svg_draw::Grid;
use crate::util::{get_bounds, translate, visit_postorder_fn};
use crate::{DerefMut, Tree, TreeRef, util::visit_fn};
use std::fmt::Write as _;
use std::fs::File;
use std::io::Write as _;

#[macro_export]
macro_rules! text_tree {
    ($s:expr) => {
        {
            let (s,w,h) = text_draw::node_size($s);
            debug_assert_ne!(w, 0.);
            debug_assert_ne!(h, 0.);
            let tr = TreeRef::new(w as f64, h as f64);
            tr.borrow_mut().content = s;
            tr
        }
    };
    ($s:expr; $($ch:expr),+) => {
        {
            let (s,w,h) = text_draw::node_size($s);
            debug_assert_ne!(w, 0.);
            debug_assert_ne!(h, 0.);
            let tr = TreeRef::new_with_children(w as f64, h as f64,vec![$($ch),+] );
            tr.borrow_mut().content = s;
            tr
        }
    };
}

pub fn node_size(val: impl Into<String>) -> (String, f64, f64) {
    let s: String = val.into();

    let text_width = s.lines().map(|s| s.len()).max().unwrap_or(1).max(1);
    let text_height = if s.is_empty() {
        // special case to get a cute empty node
        // ╭┴╮
        // ╰─╯
        0
    } else {
        // note that this is not the same as s.lines.count()
        // we want "a\n" to have two lines
        s.matches('\n').count() + 1
    };

    // for n chars, we need an interval of length n-1 in f64-space, e.g. [0, n-1]
    // first char at 0, second char at 1, n-th char at n-1.
    //
    // for the width, we add 3 (2 for the node border, 1 as padding to ensure that nodes are at
    // least 1 apart, ensuring that flooring f64-positions of adjacent nodes wont hit the same
    // integer), thus node_width = (text_width - 1) + 3 = text_width + 2
    //
    // for the height, we only need to add the 2 for the node border, the padding is handled
    // separately by the vertical layout
    let node_width = text_width + 2;
    let node_height = text_height + 1;

    (s, node_width as f64, node_height as f64)
}

pub fn ascii_draw_debug(tree: TreeRef, out: &'static str) {
    const COMPACT: bool = true;
    visit_fn(tree.borrow_mut().deref_mut(), |t| {
        if COMPACT && t.children.len() == 1 {
            t.children[0].borrow_mut().y = t.y + t.height + 1.;
        }
        if !COMPACT || t.children.len() > 1 {
            for c in &t.children {
                c.borrow_mut().y = t.y + t.height + 2.;
            }
        }
    });

    tree.layout();

    let bounds = get_bounds(tree.borrow_mut().deref_mut());

    assert!(bounds.min_y < bounds.max_y);
    assert!(bounds.min_x < bounds.max_x);
    translate(tree.borrow_mut().deref_mut(), -bounds.min_x, -bounds.min_y);

    let scale = 10.;
    visit_fn(tree.borrow_mut().deref_mut(), |t| {
        t.x = (t.x - t.width / 2.0) * scale + (t.width * scale * 0.5);
        t.width *= scale;
        // t.x *= scale;
        t.y *= scale;
        t.height *= scale;
    });

    let y_max = (bounds.max_y - bounds.min_y) * scale;
    let x_max = (bounds.max_x - bounds.min_x) * scale;

    let mut dst = String::new();

    writeln!(
        dst,
        "{}",
        svg_fmt::BeginSvg {
            w: x_max as f32,
            h: y_max as f32
        }
    )
    .unwrap();

    writeln!(
        &mut dst,
        "{}",
        svg_fmt::rectangle(0., 0., x_max as f32, y_max as f32).fill(svg_fmt::Fill::Color(
            svg_fmt::Color {
                r: 255,
                g: 255,
                b: 255,
            }
        ))
    )
    .unwrap();

    let g = Grid::new(0., 0., x_max as f32, y_max as f32, 10.);
    writeln!(dst, "{}", g).unwrap();

    visit_fn(tree.borrow_mut().deref_mut(), |t| {
        writeln!(
            dst,
            "{}",
            svg_fmt::rectangle(
                (t.x - t.width / 2.) as f32,
                t.y as f32,
                t.width as f32,
                t.height as f32
            )
            .fill(svg_fmt::Fill::None)
            .stroke(svg_fmt::Stroke::Color(svg_fmt::black(), 0.5))
        )
        .unwrap();
    });

    visit_fn(tree.borrow_mut().deref_mut(), |t| {
        // TODO: re-check this with new understanding of x as the center coordinate
        // t.x += 1.;
        // t.width -= 1.;
        t.width -= 1. * scale; // why only decreasing width by 1
        // t.y += 1. * scale;
        // t.height -= 1. * scale;
    });

    visit_fn(tree.borrow_mut().deref_mut(), |t| {
        writeln!(
            dst,
            "{}",
            svg_fmt::rectangle(
                (t.x - t.width / 2.) as f32,
                t.y as f32,
                t.width as f32,
                t.height as f32
            )
            .fill(svg_fmt::Fill::None)
            .stroke(svg_fmt::Stroke::Color(svg_fmt::green(), 0.5))
        )
        .unwrap();
    });

    writeln!(dst, "{}", svg_fmt::EndSvg).unwrap();

    let mut file = File::create(out).unwrap();
    writeln!(file, "{}", dst).unwrap();
}

pub fn layout_and_print(tree: TreeRef, opt: DrawOptions) {
    visit_fn(tree.borrow_mut().deref_mut(), |t| {
        if opt.compact && t.children.len() == 1 {
            t.children[0].borrow_mut().y = t.y + t.height + 1.;
        }
        if !opt.compact || t.children.len() > 1 {
            for c in &t.children {
                c.borrow_mut().y = t.y + t.height + 2.;
            }
        }
    });

    tree.layout();

    visit_fn(tree.borrow_mut().deref_mut(), |t| {
        t.width -= 1.;
    });

    let bounds = get_bounds(tree.borrow_mut().deref_mut());

    assert!(bounds.min_y < bounds.max_y);
    assert!(bounds.min_x < bounds.max_x);

    translate(tree.borrow_mut().deref_mut(), -bounds.min_x, -bounds.min_y);
    let bounds = get_bounds(tree.borrow_mut().deref_mut());

    let width = (bounds.max_x - bounds.min_x).floor() as usize + 1;
    let height = (bounds.max_y - bounds.min_y).floor() as usize + 1;

    let mut ascii_edges = AsciiDrawEdges::new(width, height, opt);

    visit_postorder_fn(tree.borrow_mut().deref_mut(), |t| {
        ascii_edges.draw_edges(t);
    });

    for l in ascii_edges.dst {
        let line: String = l.iter().collect();
        println!("{line}")
    }
}

#[derive(Clone, Copy)]
pub(crate) struct DrawOptions {
    compact: bool,
    char_set: CharSet,
}

impl Default for DrawOptions {
    fn default() -> Self {
        Self {
            compact: true,
            char_set: rounded(),
        }
    }
}

struct AsciiDrawEdges {
    // char_set: CharSet,
    dst: Vec<Vec<char>>,
    opt: DrawOptions,
}

impl AsciiDrawEdges {
    fn new(width: usize, height: usize, opt: DrawOptions) -> Self {
        let dst = vec![vec![' '; width]; height];
        Self { dst, opt }
    }

    fn draw_box(&mut self, t: &Tree) {
        let tl = t.top_left();
        let x_0 = tl.0.floor() as usize;
        let y_0 = tl.1.floor() as usize;

        let br = t.bottom_right();
        let x_1 = br.0.floor() as usize;
        let y_1 = br.1.floor() as usize;

        debug_assert!(x_1 >= x_0);
        debug_assert!(y_1 >= y_0);

        for i in (x_0 + 1)..x_1 {
            self.dst[y_0][i] = self.opt.char_set.horizontal;
            self.dst[y_1][i] = self.opt.char_set.horizontal;
        }

        for i in (y_0 + 1)..y_1 {
            self.dst[i][x_0] = self.opt.char_set.vertical;
            self.dst[i][x_1] = self.opt.char_set.vertical;
        }

        self.dst[y_0][x_0] = self.opt.char_set.top_left;
        self.dst[y_0][x_1] = self.opt.char_set.top_right;
        self.dst[y_1][x_1] = self.opt.char_set.bottom_right;
        self.dst[y_1][x_0] = self.opt.char_set.bottom_left;

        let mut x_off = 0;
        let mut y_off = 0;

        // HACK: fragile and not nice (skipping the node border)
        let x_start = x_0 + 1;
        let y_start = y_0 + 1;

        for c in t.content.chars() {
            if c == '\n' {
                y_off += 1;
                x_off = 0;
            } else {
                self.dst[y_start + y_off][x_start + x_off] = c;
                x_off += 1;
            }
        }
    }
    fn draw_edges(&mut self, t: &mut Tree) {
        self.draw_box(t);

        let parent_center_x = t.x.floor() as usize;
        let parent_bottom = t.bottom().floor() as usize;

        if !t.children.is_empty() {
            self.dst[parent_bottom][parent_center_x] = self.opt.char_set.join_top;
        }

        for (i, child) in t.children.iter().enumerate() {
            let child_center_x = child.borrow().x.floor() as usize;
            let child_top = child.borrow().y.floor() as usize;

            self.dst[child_top][child_center_x] = self.opt.char_set.join_bottom;

            if t.children.len() == 1 {
                if !self.opt.compact {
                    self.dst[child_top - 1][child_center_x] = self.opt.char_set.vertical
                }
                continue;
            }

            let last_i = t.children.len() - 1;
            self.dst[child_top - 1][child_center_x] = match child_center_x.cmp(&parent_center_x) {
                std::cmp::Ordering::Less if i == 0 => self.opt.char_set.top_left,
                std::cmp::Ordering::Greater if i == last_i => self.opt.char_set.top_right,
                // pretty sure the Equal and i == 0 or i == last_i cases can only occur with
                // unrealistically huge trees
                std::cmp::Ordering::Equal if i == 0 => self.opt.char_set.join_left,
                std::cmp::Ordering::Equal if i == last_i => self.opt.char_set.join_right,
                std::cmp::Ordering::Equal => self.opt.char_set.cross,
                _ => self.opt.char_set.join_top,
            };

            if i != t.children.len() - 1 {
                let next_center_x = t.children[i + 1].borrow().x.floor() as usize;
                for x in (child_center_x + 1)..next_center_x {
                    self.dst[child_top - 1][x] = if x == parent_center_x {
                        self.opt.char_set.join_bottom
                    } else {
                        self.opt.char_set.horizontal
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct CharSet {
    pub vertical: char,
    pub horizontal: char,
    pub join_bottom: char,
    pub join_top: char,
    pub top_left: char,
    pub top_right: char,
    pub bottom_right: char,
    pub bottom_left: char,
    pub join_left: char,
    pub join_right: char,
    pub cross: char,
}

pub fn square() -> CharSet {
    CharSet {
        vertical: '│',
        horizontal: '─',
        top_left: '┌',
        top_right: '┐',
        bottom_left: '└',
        bottom_right: '┘',
        join_bottom: '┴',
        join_top: '┬',
        join_left: '├',
        join_right: '┤',
        cross: '┼',
    }
}

pub fn rounded() -> CharSet {
    CharSet {
        vertical: '│',
        horizontal: '─',
        top_left: '╭',
        top_right: '╮',
        bottom_right: '╯',
        bottom_left: '╰',
        join_bottom: '┴',
        join_top: '┬',
        join_left: '├',
        join_right: '┤',
        cross: '┼',
    }
}
