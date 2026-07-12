#[derive(Debug, Clone, serde::Deserialize)]
#[serde(from = "JsonTree<T>")]
pub struct Tree<T> {
    val: T,
    children: Vec<Tree<T>>,
    num_nodes: usize,
}

#[derive(serde::Deserialize)]
pub struct JsonTree<T> {
    content: T,
    children: Option<Vec<Tree<T>>>,
}

impl<T> From<JsonTree<T>> for Tree<T> {
    fn from(value: JsonTree<T>) -> Self {
        let children = value.children.unwrap_or(vec![]);
        let num_nodes = children.iter().map(|c| c.num_nodes).sum::<usize>() + 1;
        Self {
            val: value.content,
            num_nodes,
            children,
        }
    }
}

impl<T> Tree<T> {
    pub fn new(val: T, children: Vec<Tree<T>>) -> Self {
        let num_nodes = children.iter().map(|c| c.num_nodes).sum::<usize>() + 1;
        Self {
            val,
            children,
            num_nodes,
        }
    }

    pub fn len(&self) -> usize {
        self.num_nodes
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn mirror(&mut self) {
        self.children.reverse();
        for c in self.children.iter_mut() {
            c.mirror();
        }
    }
}

#[derive(Clone)]
pub struct Node {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

impl Node {
    pub fn iter_children<'s>(&'s self) -> impl ExactSizeIterator<Item = NodeId> + use<'s> {
        self.children.iter().copied()
    }
}

// immutable tree structure
#[derive(Default)]
pub struct TreeStructure {
    nodes: Vec<Node>,
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct NodeId(usize);

impl TreeStructure {
    pub fn new() -> Self {
        TreeStructure { nodes: vec![] }
    }

    pub fn load_data<'t, V>(&'t mut self, t: Tree<V>) -> TreeData<'t, V> {
        self.nodes.clear();
        self.nodes.reserve_exact(t.len());
        let mut data = Vec::with_capacity(t.len());

        self.load_data_inner(t, &mut data, None);
        TreeData { tree: self, data }
    }

    fn load_data_inner<V>(&mut self, t: Tree<V>, data: &mut Vec<V>, parent: Option<NodeId>) {
        let root_idx = self.nodes.len();
        let root_id = NodeId(root_idx);
        let mut children = Vec::<NodeId>::with_capacity(t.children.len());

        let mut child_idx = root_idx + 1;
        for c in t.children.iter() {
            children.push(NodeId(child_idx));
            child_idx += c.num_nodes;
        }

        data.push(t.val);
        let root_node = Node {
            id: root_id,
            parent,
            children,
        };
        self.nodes.push(root_node);

        for c in t.children {
            self.load_data_inner(c, data, Some(root_id));
        }
    }

    pub fn load_data_fn<'t, V, D>(
        &'t mut self,
        t: Tree<V>,
        mut data_fn: impl FnMut(V) -> D,
    ) -> TreeData<'t, D> {
        self.nodes.clear();
        self.nodes.reserve_exact(t.num_nodes);

        let mut data = Vec::<D>::with_capacity(t.num_nodes);
        self.load_data_fn_inner(t, None, &mut data_fn, &mut data);

        TreeData { tree: self, data }
    }

    fn load_data_fn_inner<V, D>(
        &mut self,
        t: Tree<V>,
        parent: Option<NodeId>,
        data_fn: &mut impl FnMut(V) -> D,
        data: &mut Vec<D>,
    ) {
        let root_idx = self.nodes.len();
        let root_id = NodeId(root_idx);
        let mut children = Vec::<NodeId>::with_capacity(t.children.len());

        let mut child_idx = root_idx + 1;
        for c in t.children.iter() {
            children.push(NodeId(child_idx));
            child_idx += c.num_nodes;
        }

        let val = data_fn(t.val);
        data.push(val);
        let root_node = Node {
            id: root_id,
            parent,
            children,
        };
        self.nodes.push(root_node);

        for c in t.children {
            self.load_data_fn_inner(c, Some(root_id), data_fn, data);
        }
    }

    pub fn root(&self) -> Option<NodeId> {
        if self.nodes.is_empty() {
            None
        } else {
            Some(self.nodes[0].id)
        }
    }

    pub fn iter_children(&self, node_id: NodeId) -> impl ExactSizeIterator<Item = NodeId> {
        self.nodes[node_id.0].children.iter().copied()
    }

    pub fn num_children(&self, node_id: NodeId) -> usize {
        self.nodes[node_id.0].children.len()
    }

    pub fn get_child(&self, node_id: NodeId, i: usize) -> NodeId {
        self.nodes[node_id.0].children[i]
    }

    pub fn has_children(&self, node_id: NodeId) -> bool {
        !self.nodes[node_id.0].children.is_empty()
    }

    pub fn first_child(&self, node_id: NodeId) -> NodeId {
        self.get_child(node_id, 0)
    }

    pub fn last_child(&self, node_id: NodeId) -> NodeId {
        self.get_child(node_id, self.num_children(node_id) - 1)
    }

    pub fn iter_preorder(&self) -> impl ExactSizeIterator<Item = &Node> {
        self.nodes.iter()
    }
}

// data container relative to a TreeStructure
// nodes are identified by NodeId, which are just wrapped indices, thus we can use them to implement
// "references" to other nodes without having to wrap everything with RefCell or using unsafe
//
// because self.tree is just &'t TreeStructure (which is Copy), it can be used meaningfully while self is
// already mutably borrowed, pretty cool? (e.g.: mutably borrow the data via an iterator, but then
// at the same time use self.tree to iterate over the children of some node, which would not be
// possible if TreeData owned the TreeStructure)
pub struct TreeData<'t, D> {
    tree: &'t TreeStructure,
    data: Vec<D>,
}

impl<'t, D> TreeData<'t, D> {
    pub fn tree(&self) -> &'t TreeStructure {
        self.tree
    }

    pub fn new_default<V>(&self) -> TreeData<'t, V>
    where
        V: Default,
    {
        TreeData {
            tree: self.tree,
            data: self.tree.nodes.iter().map(|_| V::default()).collect(),
        }
    }

    pub fn derive_from_nodes<V>(&self, derive_fn: impl Fn(&Node) -> V) -> TreeData<'t, V> {
        TreeData {
            tree: self.tree,
            data: self.tree.nodes.iter().map(derive_fn).collect(),
        }
    }

    pub fn derive<V>(&self, derive_fn: impl Fn(&TreeStructure, &Node, &D) -> V) -> TreeData<'t, V> {
        TreeData {
            tree: self.tree,
            data: self
                .tree
                .nodes
                .iter()
                .zip(self.data.iter())
                .map(|(n, d)| derive_fn(self.tree, n, d))
                .collect(),
        }
    }

    pub fn transform<V>(self, transform_fn: impl Fn(&Node, D) -> V) -> TreeData<'t, V> {
        TreeData {
            tree: self.tree,
            data: self
                .tree
                .nodes
                .iter()
                .zip(self.data)
                .map(|(n, d)| transform_fn(n, d))
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    // could implement Index/IndexMut to access?
    pub fn get(&self, id: NodeId) -> &D {
        &self.data[id.0]
    }

    pub fn get_mut(&mut self, id: NodeId) -> &mut D {
        &mut self.data[id.0]
    }

    // somewhat annoying code duplication
    // deref is not an option, because we need iter_children (and iter_nodes_preorder) to use the
    // exact lifetime (we don't want it to "actually" borrow self)
    // could make tree field public and just access that directly? but that's annoying too

    pub fn iter_children(
        &self,
        node_id: NodeId,
    ) -> impl ExactSizeIterator<Item = NodeId> + use<'t, D> {
        self.tree.iter_children(node_id)
    }

    pub fn first_child(&self, node_id: NodeId) -> NodeId {
        self.tree.first_child(node_id)
    }

    pub fn last_child(&self, node_id: NodeId) -> NodeId {
        self.tree.last_child(node_id)
    }

    pub fn get_child(&self, node_id: NodeId, i: usize) -> NodeId {
        self.tree.get_child(node_id, i)
    }

    pub fn num_children(&self, node_id: NodeId) -> usize {
        self.tree.num_children(node_id)
    }

    pub fn has_children(&self, node_id: NodeId) -> bool {
        self.tree.has_children(node_id)
    }

    pub fn root(&self) -> Option<NodeId> {
        self.tree.root()
    }

    pub fn iter_preorder(&self) -> impl ExactSizeIterator<Item = (&Node, &D)> {
        self.tree.nodes.iter().zip(self.data.iter())
    }

    pub fn iter_mut_preorder(&mut self) -> impl ExactSizeIterator<Item = (&Node, &mut D)> {
        self.tree.nodes.iter().zip(self.data.iter_mut())
    }

    pub fn iter_nodes_preorder(&self) -> impl ExactSizeIterator<Item = &'t Node> + use<'t, D> {
        self.tree.iter_preorder()
    }
}
