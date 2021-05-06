use slotmap::{Key, SlotMap};

// -------------------------------------------------------------------------------------------------

// TODO lots of unwraps here, should at least give useful error messages

pub struct Forest<K, I> where K: Key {
    nodes: SlotMap<K, ForestNode<K, I>>,
}

impl<K, I> Forest<K, I> where K: Key {
    pub fn new() -> Forest<K, I> {
        Forest { nodes: SlotMap::with_key() }
    }

    pub fn get(&self, node: K) -> &I {
        &self.nodes.get(node).unwrap().item
    }
    pub fn get_mut(&mut self, node: K) -> &mut I {
        &mut self.nodes.get_mut(node).unwrap().item
    }

    pub fn get_parent(&self, node: K) -> K {
        self.nodes.get(node).unwrap().parent
    }
    pub fn set_parent(&mut self, node: K, parent: K) {
        let old_parent = self.nodes.get(node).unwrap().parent;
        if old_parent == parent {
            return;
        }
        if !old_parent.is_null() {
            self.nodes.get_mut(old_parent).unwrap().remove_child(node);
        }
        self.nodes.get_mut(node).unwrap().parent = parent;
        if !parent.is_null() {
            self.nodes.get_mut(parent).unwrap().children.push(node);
        }
    }

    pub fn contains(&self, node: K) -> bool {
        self.nodes.contains_key(node)
    }
    pub fn add(&mut self, item: I) -> K {
        self.nodes.insert(ForestNode::new(item, K::null()))
    }
    pub fn remove(&mut self, node: K) -> I {
        self.set_parent(node, K::null());
        self.remove_node_and_children(node)
    }
    fn remove_node_and_children(&mut self, node: K) -> I {
        let mut node_data = self.nodes.remove(node).unwrap();
        for child in node_data.children.drain(..) {
            self.remove_node_and_children(child);
        }
        node_data.item
    }

    pub fn add_child(&mut self, parent: K, item: I) -> K {
        let node = self.nodes.insert(ForestNode::new(item, parent));
        if !parent.is_null() {
            self.nodes.get_mut(parent).unwrap().children.push(node);
        }
        node
    }
    pub fn get_child_count(&self, node: K) -> usize {
        self.nodes.get(node).unwrap().children.len()
    }
    pub fn get_children(&self, node: K) -> Vec<K> {
        self.nodes.get(node).unwrap().children.clone()
    }
    pub fn iter_children(&self, node: K) -> std::slice::Iter<'_, K> {
        self.nodes.get(node).unwrap().children.iter()
    }
}

pub struct ForestNode<K, I> where K: Key {
    parent: K,
    children: Vec<K>,
    item: I,
}

impl<K, I> ForestNode<K, I> where K: Key {
    fn new(item: I, parent: K) -> ForestNode<K, I> {
        ForestNode {
            parent,
            children: Vec::new(),
            item,
        }
    }
    fn remove_child(&mut self, child: K) {
        self.children.remove(self.children.iter().position(|x| *x == child).unwrap());
    }
}
