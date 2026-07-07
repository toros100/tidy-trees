use std::io::Write as _;
use svg_fmt::{BeginSvg, EndSvg, Fill, Style, line_segment};

use crate::util::{Visit, get_bounds, translate};
use crate::{Tree, TreeRef, visit, visit_fn};
use std::fmt::{Display, Write as _};
use std::fs::File;
use std::ops::DerefMut;
use std::path::Path;

#[derive(Clone, Copy)]
pub struct DrawOptions {
    pub node_x_pad: f64,
    pub node_y_pad: f64,
    pub img_pad: f64,
    pub show_bounding_boxes: bool,
}

impl Default for DrawOptions {
    fn default() -> Self {
        Self {
            node_y_pad: 10.,
            node_x_pad: 10.,
            img_pad: 20.,
            show_bounding_boxes: false,
        }
    }
}

pub struct Grid {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    gap: f32,
}

impl Grid {
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32, gap: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
            gap,
        }
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert!(self.gap.is_normal());

        let mut i = 0usize;

        let regular_style = svg_fmt::Style {
            stroke: svg_fmt::Stroke::Color(svg_fmt::red(), 0.5),
            stroke_opacity: 0.5,
            ..Default::default()
        };

        let thick_style = svg_fmt::Style {
            stroke: svg_fmt::Stroke::Color(svg_fmt::red(), 1.),
            ..regular_style
        };

        loop {
            let curr = (i as f32) * self.gap;

            if curr > self.max_x && curr > self.max_y {
                break;
            }

            let st = if i.is_multiple_of(5) {
                thick_style
            } else {
                regular_style
            };

            if curr <= self.max_x {
                writeln!(
                    f,
                    "{}",
                    svg_fmt::path()
                        .style(st)
                        .move_to(curr, 0.)
                        .line_to(curr, self.max_y)
                )
                .unwrap();
            }

            if curr <= self.max_y {
                writeln!(
                    f,
                    "{}",
                    svg_fmt::path()
                        .style(st)
                        .move_to(0., curr)
                        .line_to(self.max_x, curr)
                )
                .unwrap();
            }
            i += 1;
        }
        Ok(())
    }
}
struct DrawSvg<'d> {
    dst: &'d mut String,
    opt: DrawOptions,
}

impl<'d> DrawSvg<'d> {
    fn new(dst: &'d mut String, opt: DrawOptions) -> Self {
        Self { dst, opt }
    }
}

impl<'d> Visit for DrawSvg<'d> {
    fn visit(&mut self, t: &mut Tree) {
        let top_left = (t.top_left().0 as f32, t.top_left().1 as f32);

        let rect = svg_fmt::rectangle(top_left.0, top_left.1, t.width as f32, t.height as f32)
            .stroke(svg_fmt::Stroke::Color(svg_fmt::black(), 1.))
            .fill(svg_fmt::Fill::Color(svg_fmt::Color {
                r: 200,
                g: 230,
                b: 230,
            }));
        writeln!(self.dst, "{}", rect).unwrap();

        let (t_cb_x, t_cb_y) = t.center_bottom();
        // NOTE: would be nicer to handle node padding per node rather than globally
        let edge_horizontal_y = (t_cb_y + self.opt.node_y_pad) as f32;

        if !t.children.is_empty() {
            let (x_0, _) = t.children[0].borrow().center_top();
            let (x_1, _) = t.children[t.children.len() - 1].borrow().center_top();
            let y = t.y + t.height + self.opt.node_y_pad;

            writeln!(self.dst, r#"<g style="stroke-linecap:round">"#).unwrap();

            writeln!(
                self.dst,
                "{} {}",
                line_segment(x_0 as f32, y as f32, x_1 as f32, y as f32),
                line_segment(
                    t_cb_x as f32,
                    t_cb_y as f32,
                    t_cb_x as f32,
                    edge_horizontal_y,
                )
            )
            .unwrap();
        }

        for c in &t.children {
            let (c_ct_x, c_ct_y) = c.borrow().center_top();
            writeln!(
                self.dst,
                "{}",
                line_segment(
                    c_ct_x as f32,
                    edge_horizontal_y,
                    c_ct_x as f32,
                    c_ct_y as f32
                )
            )
            .unwrap()
        }

        if !t.children.is_empty() {
            writeln!(self.dst, "</g>").unwrap();
        }
    }
}

pub(crate) fn draw_to_svg(
    tree: TreeRef,
    out: impl AsRef<Path>,
    opt: DrawOptions,
) -> anyhow::Result<()> {
    visit_fn(tree.borrow_mut().deref_mut(), |t| {
        // adding padding to node dimensions and computing vertical layout
        t.width += 2. * opt.node_x_pad;
        t.height += 2. * opt.node_y_pad;
        for c in t.children.iter() {
            c.borrow_mut().y = t.y + t.height;
        }
    });

    tree.layout();
    let bounds = get_bounds(tree.borrow_mut().deref_mut());

    let width = (bounds.max_x - bounds.min_x) as f32 + (2. * opt.img_pad as f32);
    let height = (bounds.max_y - bounds.min_y) as f32 + (2. * opt.img_pad as f32);

    translate(
        tree.borrow_mut().deref_mut(),
        -bounds.min_x + opt.img_pad,
        -bounds.min_y + opt.img_pad,
    );

    let mut dst = String::new();

    writeln!(
        dst,
        "{}",
        BeginSvg {
            w: width,
            h: height
        }
    )?;

    writeln!(
        dst,
        "{}",
        svg_fmt::rectangle(0., 0., width, height).fill(Fill::Color(svg_fmt::Color {
            r: 255,
            g: 255,
            b: 255,
        }))
    )?;

    if opt.show_bounding_boxes {
        let s = Style {
            opacity: 0.3,
            fill: svg_fmt::red().into(),
            ..Default::default()
        };

        visit_fn(tree.borrow_mut().deref_mut(), |t| {
            let (x, y) = {
                let tl = t.top_left();
                (tl.0 as f32, tl.1 as f32)
            };
            writeln!(
                &mut dst,
                "{}",
                svg_fmt::rectangle(x, y, t.width as f32, t.height as f32).style(s)
            )
            .unwrap()
        });
    }

    visit_fn(tree.borrow_mut().deref_mut(), |t| {
        // removing padding again, s.t. node area is the actual content area
        t.width -= 2. * opt.node_x_pad;
        t.height -= 2. * opt.node_y_pad;
        // only have to shift the node on the y-axis, because t.y is the top of the node, but t.x is
        // at the center
        t.y += opt.node_y_pad;
    });

    let mut ds = DrawSvg::new(&mut dst, opt);
    visit(&mut ds, tree.borrow_mut().deref_mut());

    writeln!(&mut dst, "{}", EndSvg)?;

    let mut file = File::create(out)?;
    writeln!(file, "{}", dst)?;

    Ok(())
}
