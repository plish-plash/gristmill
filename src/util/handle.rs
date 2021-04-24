use std::sync::Arc;

use slotmap::{DenseSlotMap, Key};

pub trait Handle {
    type Key: Key;
    fn from(key: Self::Key) -> Self;
    fn key(&self) -> Self::Key;
}

#[macro_export]
macro_rules! new_handle_type {
    ($name:ident) => {
        #[derive(Copy, Clone, Eq, PartialEq, Debug)]
        pub struct $name(slotmap::DefaultKey);
        impl $crate::util::handle::Handle for $name {
            type Key = slotmap::DefaultKey;
            fn from(key: Self::Key) -> Self { $name(key) }
            fn key(&self) -> Self::Key { self.0 }
        }
    };
}

pub struct HandleOwner<H, V> where H: Handle {
    handles: Vec<Arc<H>>,
    data: DenseSlotMap<H::Key, V>,
}

// This is the right API, but I feel like the implementation could be cleaner
impl<H, V> HandleOwner<H, V> where H: Handle {
    pub fn new() -> HandleOwner<H, V> {
        HandleOwner { handles: Vec::new(), data: DenseSlotMap::with_key() }
    }
    pub fn get(&self, handle: &Arc<H>) -> &V {
        self.data.get(handle.key()).unwrap()
    }
    pub fn insert(&mut self, value: V) -> Arc<H> {
        let key = self.data.insert(value);
        let handle = Arc::new(H::from(key));
        self.handles.push(handle.clone());
        handle
    }
    pub fn cleanup(&mut self) {
        let data = &mut self.data;
        self.handles = self.handles.drain(..).filter(move |handle| {
            if Arc::strong_count(handle) > 1 {
                true
            }
            else {
                data.remove(handle.key());
                false
            }
        }).collect();
    }

    pub fn iter(&self) -> impl Iterator<Item = &V> {
        self.data.values()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.values_mut()
    }
}
