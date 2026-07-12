use crate::tree::{NodeId, TreeData};

impl From<NodeOptions> for LayoutData {
    fn from(value: NodeOptions) -> Self {
        Self {
            width: value.width,
            height: value.height,
            ..Default::default()
        }
    }
}

// TODO: should all be non-negative and normal, could error or just set to 0.0 otherwise?
#[derive(Default, Clone, Copy)]
pub struct NodeOptions {
    pub width: f64,
    pub height: f64,
    pub x_pad: f64,
    pub y_pad: f64,
    pub parent_gap: f64,
}

#[derive(Default)]
pub struct LayoutData {
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
    prelim: f64,
    modifier: f64,
    shift: f64,
    change: f64,
    thread_left: Option<NodeId>,
    thread_right: Option<NodeId>,
    extreme_left: Option<NodeId>,
    extreme_right: Option<NodeId>,
    msel: f64,
    mser: f64,
}

impl LayoutData {
    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LeftSibling {
    lowest_y: f64, // y increases downwards, so "lowest y" is largest by value
    index: usize,
}

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

struct TidyLayout<'b, 'd, LayoutData> {
    data: &'b mut TreeData<'d, LayoutData>,
}

pub struct Bounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

impl<'b, 'd> TidyLayout<'b, 'd, LayoutData> {
    fn new(data: &'b mut TreeData<'d, LayoutData>) -> Self {
        Self { data }
    }

    pub fn bounds(&self) -> Option<Bounds> {
        if self.data.is_empty() {
            None
        } else {
            let mut min_x = f64::MAX;
            let mut max_x = f64::MIN;
            let mut min_y = f64::MAX;
            let mut max_y = f64::MIN;
            for (_, d) in self.data.iter_preorder() {
                min_x = min_x.min(d.x - d.width / 2.);
                max_x = max_x.max(d.x + d.width / 2.);
                min_y = min_y.min(d.y);
                max_y = max_y.max(d.y + d.height);
            }

            Some(Bounds {
                min_x,
                max_x,
                min_y,
                max_y,
            })
        }
    }

    fn data(&self, id: NodeId) -> &LayoutData {
        self.data.get(id)
    }
    fn data_mut(&mut self, id: NodeId) -> &mut LayoutData {
        self.data.get_mut(id)
    }

    fn layout(&mut self) {
        if let Some(root_id) = self.data.root() {
            self.first_walk(root_id);
            self.second_walk();
        }
    }

    // fn second_walk(&mut self, id: NodeId, mut mod_sum: f64) {
    //     let data = self.data_mut(id);
    //     mod_sum += data.modifier;
    //     data.x = data.prelim + mod_sum;
    //     self.add_child_spacing(id);
    //
    //     for child_id in self.data.iter_children(id) {
    //         self.second_walk(child_id, mod_sum);
    //     }
    // }

    fn second_walk(&mut self) {
        for n in self.data.iter_nodes_preorder() {
            let inherited_modifier = n.parent.map(|p| self.data.get(p).modifier).unwrap_or(0.);
            let n_data = self.data_mut(n.id);
            n_data.modifier += inherited_modifier;
            n_data.x = n_data.prelim + n_data.modifier;
            for c in n.iter_children() {
                self.add_child_spacing(c);
            }
        }
    }

    fn add_child_spacing(&mut self, id: NodeId) {
        let mut shift_sum = 0f64;
        let mut mod_sum_delta = 0f64;
        for child_id in self.data.iter_children(id) {
            let child_data = self.data_mut(child_id);
            shift_sum += child_data.shift;
            mod_sum_delta += shift_sum + child_data.change;
            child_data.modifier += mod_sum_delta;
        }
    }

    fn set_right_thread(&mut self, id: NodeId, i: usize, sr: NodeId, mssr: f64) {
        let ri = self.data(self.data.get_child(id, i)).extreme_right.unwrap();

        self.data_mut(ri).thread_right = Some(sr);

        let diff = { mssr - self.data(sr).modifier - self.data(self.data.get_child(id, i)).mser };

        self.data_mut(ri).modifier += diff;
        self.data_mut(ri).prelim -= diff;

        let prev_subtree_er = self.data(self.data.get_child(id, i - 1)).extreme_right;
        self.data_mut(self.data.get_child(id, i)).extreme_right = prev_subtree_er;
        let prev_subtree_mser = self.data(self.data.get_child(id, i - 1)).mser;
        self.data_mut(self.data.get_child(id, i)).mser = prev_subtree_mser;
    }

    fn set_left_thread(&mut self, id: NodeId, i: usize, cl: NodeId, mscl: f64) {
        let li = self.data(self.data.first_child(id)).extreme_left.unwrap();

        self.data_mut(li).thread_left = Some(cl);

        let diff = { mscl - self.data(cl).modifier - self.data(self.data.first_child(id)).msel };

        self.data_mut(li).modifier += diff;
        self.data_mut(li).prelim -= diff;

        let curr_subtree_el = self.data(self.data.get_child(id, i)).extreme_left;
        self.data_mut(self.data.first_child(id)).extreme_left = curr_subtree_el;
        let curr_subtree_msel = self.data(self.data.get_child(id, i)).msel;
        self.data_mut(self.data.first_child(id)).msel = curr_subtree_msel;
    }

    fn first_walk(&mut self, id: NodeId) {
        if !self.data.has_children(id) {
            self.set_extremes(id);
        } else {
            let first_child_id = self.data.first_child(id);

            self.first_walk(first_child_id);

            let mut lookup = LeftSiblingLookup::default();

            let min_y = self
                .data(self.data(first_child_id).extreme_left.unwrap())
                .bottom();
            lookup.update(min_y, 0);

            for (i, child_id) in self.data.iter_children(id).enumerate().skip(1) {
                self.first_walk(child_id);
                let min_y = self
                    .data(self.data(child_id).extreme_right.unwrap())
                    .bottom();

                self.separate(id, i, lookup.iter());
                lookup.update(min_y, i);
            }

            self.position_root(id);
            self.set_extremes(id);
        }
    }

    fn set_extremes(&mut self, id: NodeId) {
        if !self.data.has_children(id) {
            let d = self.data_mut(id);
            d.extreme_left = Some(id);
            d.extreme_right = Some(id);
            d.msel = 0f64;
            d.mser = 0f64;
        } else {
            let (el, msel) = {
                let first_child_data = self.data(self.data.first_child(id));
                (first_child_data.extreme_left, first_child_data.msel)
            };

            let (er, mser) = {
                let last_child_data = self.data(self.data.last_child(id));
                (last_child_data.extreme_right, last_child_data.mser)
            };

            let d = &mut self.data_mut(id);
            debug_assert!(el.is_some());
            debug_assert!(er.is_some());

            d.extreme_left = el;
            d.extreme_right = er;
            d.msel = msel;
            d.mser = mser;
        }
    }

    fn separate(&mut self, id: NodeId, i: usize, mut it: LeftSiblingLookupIterator) {
        debug_assert!(i > 0);

        let l_sib = self.data.get_child(id, i - 1);
        let mut r_mod_sum = self.data(l_sib).modifier;

        let current_subtree = self.data.get_child(id, i);
        let mut l_mod_sum = self.data(current_subtree).modifier;

        let mut r_contour = Some(l_sib);
        let mut l_contour = Some(current_subtree);

        let mut first = true;

        while let Some(r_contour_id) = r_contour
            && let Some(l_contour_id) = l_contour
        {
            debug_assert!(!it.is_empty());

            let sib = it.peek().unwrap();

            if self.data(r_contour_id).bottom() > sib.lowest_y {
                _ = it.next();
                debug_assert!(!it.is_empty());
            }

            let dist = {
                (r_mod_sum + self.data(r_contour_id).prelim)
                    - (l_mod_sum + self.data(l_contour_id).prelim)
                    + self.data(r_contour_id).width / 2.0
                    + self.data(l_contour_id).width / 2.0
            };

            if first && dist <= 0. || dist > 0f64 {
                l_mod_sum += dist;
                self.move_subtree(id, i, it.peek().unwrap().index, dist);
            }
            first = false;

            let r_bottom = self.data(r_contour_id).bottom();
            let l_bottom = self.data(l_contour_id).bottom();

            if r_bottom <= l_bottom {
                r_contour = self.next_right_contour(r_contour_id);
                if let Some(r_contour_id) = r_contour {
                    r_mod_sum += self.data(r_contour_id).modifier;
                }
            }
            if r_bottom >= l_bottom {
                l_contour = self.next_left_contour(l_contour_id);
                if let Some(l_contour_id) = l_contour {
                    l_mod_sum += self.data(l_contour_id).modifier;
                }
            }

            if r_contour.is_none()
                && let Some(l_contour_id) = l_contour
            {
                self.set_left_thread(id, i, l_contour_id, l_mod_sum);
            } else if let Some(r_contour_id) = r_contour
                && l_contour.is_none()
            {
                self.set_right_thread(id, i, r_contour_id, r_mod_sum);
            }
        }
    }

    fn position_root(&mut self, id: NodeId) {
        self.data_mut(id).prelim = {
            let first_ch_data = self.data(self.data.first_child(id));
            let last_ch_data = self.data(self.data.last_child(id));

            (first_ch_data.modifier + first_ch_data.prelim - first_ch_data.width / 2.0
                + last_ch_data.modifier
                + last_ch_data.prelim
                + last_ch_data.width / 2.0)
                / 2.0
        };
    }

    fn move_subtree(&mut self, id: NodeId, i: usize, si: usize, dist: f64) {
        let ith_child_data = self.data_mut(self.data.get_child(id, i));
        ith_child_data.modifier += dist;
        ith_child_data.msel += dist;
        ith_child_data.mser += dist;
        self.distribute_extra(id, i, si, dist);
    }

    fn next_left_contour(&self, id: NodeId) -> Option<NodeId> {
        if !self.data.has_children(id) {
            self.data(id).thread_left
        } else {
            Some(self.data.first_child(id))
        }
    }

    fn next_right_contour(&self, id: NodeId) -> Option<NodeId> {
        if !self.data.has_children(id) {
            self.data(id).thread_right
        } else {
            Some(self.data.last_child(id))
        }
    }

    fn distribute_extra(&mut self, id: NodeId, i: usize, si: usize, dist: f64) {
        debug_assert!(si <= i);

        let n = i - si;
        if n > 1 {
            let val = dist / (n as f64);
            self.data_mut(self.data.get_child(id, si + 1)).shift += val;
            let d = self.data_mut(self.data.get_child(id, i));
            d.shift -= val;
            d.change -= dist - val;
        }
    }
}

impl<'t> TreeData<'t, LayoutData> {
    pub fn bounds(&self) -> Option<Bounds> {
        if self.is_empty() {
            None
        } else {
            let mut min_x = f64::MAX;
            let mut max_x = f64::MIN;
            let mut min_y = f64::MAX;
            let mut max_y = f64::MIN;
            for (_, d) in self.iter_preorder() {
                min_x = min_x.min(d.x - d.width / 2.);
                max_x = max_x.max(d.x + d.width / 2.);
                min_y = min_y.min(d.y);
                max_y = max_y.max(d.y + d.height);
            }

            Some(Bounds {
                min_x,
                max_x,
                min_y,
                max_y,
            })
        }
    }
    pub fn normalize(&mut self) {
        if let Some(b) = self.bounds() {
            for (_, d) in self.iter_mut_preorder() {
                d.x -= b.min_x;
                d.y -= b.min_y;
            }
        }
    }
}

pub fn layout<'t>(td: &TreeData<'t, NodeOptions>) -> TreeData<'t, LayoutData> {
    let mut ld = td.derive(|_, _, c| LayoutData {
        width: c.width + 2. * c.x_pad,
        height: c.height + 2. * c.y_pad,
        ..Default::default()
    });

    for n in ld.iter_nodes_preorder() {
        ld.get_mut(n.id).y =
            n.parent.map_or(0., |p| ld.get(p).y + ld.get(p).height) + td.get(n.id).parent_gap;
    }

    let mut tl = TidyLayout::new(&mut ld);
    tl.layout();

    for (n, d) in ld.iter_mut_preorder() {
        let opt = td.get(n.id);
        d.height -= 2. * opt.y_pad;
        d.width -= 2. * opt.x_pad;
        d.y += opt.y_pad;
    }

    // made the relevant fields in LayoutData public so it can be used to provide the final layout
    // without allocating for a new TreeData (with a struct that would just contain x, y, width,
    // height)
    ld
}
