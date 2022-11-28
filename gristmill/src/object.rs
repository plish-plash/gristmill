pub use slotmap::*;
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

#[macro_export]
macro_rules! new_object_type {
    ($data_ty:ty, $key_ty:ident, $object_ty:ident, $collection_ty:ident) => {
        $crate::object::new_key_type! { pub struct $key_ty; }
        pub type $object_ty = $crate::object::Object<$key_ty, $data_ty>;
        pub type $collection_ty =
            std::sync::Arc<std::sync::RwLock<$crate::object::DenseSlotMap<$key_ty, $data_ty>>>;
    };
}

pub trait ObjectCollection {
    type Key: Key;
    type Value;
    fn insert(&self, value: Self::Value) -> Object<Self::Key, Self::Value>;
    fn remove(&self, object: Object<Self::Key, Self::Value>) -> Option<Self::Value>;
}

impl<K: Key, V> ObjectCollection for Arc<RwLock<DenseSlotMap<K, V>>> {
    type Key = K;
    type Value = V;
    fn insert(&self, value: Self::Value) -> Object<Self::Key, Self::Value> {
        let key = self
            .try_write()
            .expect("ObjectCollection::insert() not allowed here")
            .insert(value);
        Object::from_key(self.clone(), key)
    }
    fn remove(&self, object: Object<Self::Key, Self::Value>) -> Option<Self::Value> {
        self.try_write()
            .expect("ObjectCollection::remove() not allowed here")
            .remove(object.key())
    }
}

pub struct ObjectReadGuard<'a, K: Key, V> {
    read_guard: RwLockReadGuard<'a, DenseSlotMap<K, V>>,
    key: K,
}

impl<'a, K: Key, V> Deref for ObjectReadGuard<'a, K, V> {
    type Target = V;
    fn deref(&self) -> &V {
        self.read_guard.get(self.key).expect("object removed")
    }
}

pub struct ObjectWriteGuard<'a, K: Key, V> {
    write_guard: RwLockWriteGuard<'a, DenseSlotMap<K, V>>,
    key: K,
}

impl<'a, K: Key, V> Deref for ObjectWriteGuard<'a, K, V> {
    type Target = V;
    fn deref(&self) -> &V {
        self.write_guard.get(self.key).expect("object removed")
    }
}
impl<'a, K: Key, V> DerefMut for ObjectWriteGuard<'a, K, V> {
    fn deref_mut(&mut self) -> &mut V {
        self.write_guard.get_mut(self.key).expect("object removed")
    }
}

pub struct Object<K: Key, V> {
    objects: Arc<RwLock<DenseSlotMap<K, V>>>,
    key: K,
}

impl<K: Key, V> Clone for Object<K, V> {
    fn clone(&self) -> Self {
        Object {
            objects: self.objects.clone(),
            key: self.key,
        }
    }
}

impl<K: Key, V> PartialEq for Object<K, V> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.objects, &other.objects) && self.key == other.key
    }
}
impl<K: Key, V> Eq for Object<K, V> {}

impl<K: Key, V> Object<K, V> {
    pub fn from_key(objects: Arc<RwLock<DenseSlotMap<K, V>>>, key: K) -> Self {
        Object { objects, key }
    }
    pub fn key(&self) -> K {
        self.key
    }

    pub fn objects(&self) -> &Arc<RwLock<DenseSlotMap<K, V>>> {
        &self.objects
    }
    pub fn exists(&self) -> bool {
        let read_guard = self
            .objects
            .try_read()
            .expect("Object::exists() not allowed here");
        read_guard.contains_key(self.key)
    }
    pub fn read(&self) -> ObjectReadGuard<K, V> {
        let read_guard = self
            .objects
            .try_read()
            .expect("Object::read() not allowed here");
        ObjectReadGuard {
            read_guard,
            key: self.key,
        }
    }
    pub fn write(&self) -> ObjectWriteGuard<K, V> {
        let write_guard = self
            .objects
            .try_write()
            .expect("Object::write() not allowed here");
        ObjectWriteGuard {
            write_guard,
            key: self.key,
        }
    }
}
