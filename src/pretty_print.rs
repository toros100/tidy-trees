use crate::{
    tidy_layout::{self, LayoutData, NodeOptions},
    tree::{self, Node},
};

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

impl CharSet {
    pub fn square() -> Self {
        Self {
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
    pub fn rounded() -> Self {
        Self {
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
}

pub fn node_size(val: impl AsRef<str>) -> (f64, f64) {
    let s = val.as_ref();

    // NOTE: should use something (e.g. https://crates.io/crates/unicode-width) to detect the actual
    // width in columns (the entire text handling thing needs to be reworked, would be cool to
    // support colors too)

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
    // we add 2 for the frame, thus node_width = (text_width - 1) + 2
    let node_width = (text_width + 1) as f64;
    let node_height = (text_height + 1) as f64;

    (node_width, node_height)
}

pub fn print_tree(td: tree::TreeData<String>) {
    let nd = td.derive(|a, b, c| {
        // placing single children closer to their parent, because no space for connecting edges is
        // required, e.g.:
        //
        //   ╭───╮              ╭───╮
        //   │ a │              │ a │
        //   ╰─┬─╯      vs      ╰─┬─╯
        //   ╭─┴──╮             ╭─┴─╮
        // ╭─┴─╮╭─┴─╮           │ x │
        // │ x ││ y │           ╰───╯
        // ╰───╯╰───╯
        let parent_gap = b
            .parent
            .map(|p| if a.num_children(p) == 1 { 1. } else { 2. })
            .unwrap_or(0.);

        let (width, height) = node_size(c);

        NodeOptions {
            width,
            height,
            parent_gap,
            x_pad: 0.5, // ensures that nodes are placed at least 1.0 apart in f64 space
            // s.t. their endpoints will not floor to the same integer
            ..Default::default()
        }
    });

    let mut tl = tidy_layout::layout(&nd);

    tl.normalize();

    let b = tl.bounds().unwrap();

    let width = (b.max_x - b.min_x).floor() as usize + 1;
    let height = (b.max_y - b.min_y).floor() as usize + 1;

    let mut to = TextOut::new(width, height, CharSet::rounded());

    for (_, d) in tl.iter_preorder() {
        // just writing the node frames and then overwriting portions of it again
        // (where edges connect), could do better
        to.write_frame(d);
    }

    for (n, d) in tl.iter_preorder() {
        let x = (d.x - d.width / 2.0).floor() as usize;
        let y = d.y.floor() as usize;

        let text_base_x = x + 1;
        let text_base_y = y + 1;
        let mut text_off_x = 0;
        let mut text_off_y = 0;
        let s = td.get(n.id);

        // TODO: make less bad
        for c in s.chars() {
            match c {
                '\r' => continue,
                '\n' => {
                    text_off_x = 0;
                    text_off_y += 1;
                }
                c => {
                    to.dst[text_base_y + text_off_y][text_base_x + text_off_x] = c;
                    text_off_x += 1;
                }
            }
        }

        to.write_edges(&tl, n);
    }

    for line in to.dst {
        let s = line.iter().collect::<String>();
        println!("{s}")
    }
}

struct TextOut {
    dst: Vec<Vec<char>>,
    char_set: CharSet,
}

impl TextOut {
    fn new(width: usize, height: usize, char_set: CharSet) -> Self {
        Self {
            dst: vec![vec![' '; width]; height],
            char_set,
        }
    }

    fn write_frame(&mut self, d: &tidy_layout::LayoutData) {
        let x = (d.x - d.width / 2.0).floor() as usize;
        let y = d.y.floor() as usize;
        let h = d.height.floor() as usize;
        let w = d.width.floor() as usize;

        self.dst[y][x] = self.char_set.top_left;
        self.dst[y][x + w] = self.char_set.top_right;
        self.dst[y + h][x] = self.char_set.bottom_left;
        self.dst[y + h][x + w] = self.char_set.bottom_right;

        for xi in (x + 1)..(x + w) {
            self.dst[y][xi] = self.char_set.horizontal;
            self.dst[y + h][xi] = self.char_set.horizontal;
        }
        for yi in (y + 1)..(y + h) {
            self.dst[yi][x] = self.char_set.vertical;
            self.dst[yi][x + w] = self.char_set.vertical;
        }
    }

    fn write_edges(&mut self, tl: &tree::TreeData<LayoutData>, n: &Node) {
        let d = tl.get(n.id);

        let parent_center_x = d.x.floor() as usize;
        let parent_bottom = d.bottom().floor() as usize;

        if !n.children.is_empty() {
            self.dst[parent_bottom][parent_center_x] = self.char_set.join_top;
        }

        for (i, child) in n.children.iter().enumerate() {
            let cd = tl.get(*child);

            let child_center_x = cd.x.floor() as usize;
            let child_top = cd.y.floor() as usize;

            self.dst[child_top][child_center_x] = self.char_set.join_bottom;

            if n.children.len() == 1 {
                // if !self.opt.compact {
                //     self.dst[child_top - 1][child_center_x] = self.opt.char_set.vertical
                // }
                continue;
            }

            let last_i = n.children.len() - 1;
            self.dst[child_top - 1][child_center_x] = match child_center_x.cmp(&parent_center_x) {
                std::cmp::Ordering::Less if i == 0 => self.char_set.top_left,
                std::cmp::Ordering::Greater if i == last_i => self.char_set.top_right,
                // pretty sure the Equal and i == 0 or i == last_i cases can only occur with
                // unrealistically huge trees
                std::cmp::Ordering::Equal if i == 0 => self.char_set.join_left,
                std::cmp::Ordering::Equal if i == last_i => self.char_set.join_right,
                std::cmp::Ordering::Equal => self.char_set.cross,
                _ => self.char_set.join_top,
            };

            if i != n.children.len() - 1 {
                let next_center_x = tl.get(tl.get_child(n.id, i + 1)).x.floor() as usize;
                // d.children[i + 1].borrow().x.floor() as usize;
                for x in (child_center_x + 1)..next_center_x {
                    self.dst[child_top - 1][x] = if x == parent_center_x {
                        self.char_set.join_bottom
                    } else {
                        self.char_set.horizontal
                    }
                }
            }
        }
    }
}
