use downcast_rs::Downcast;
use slab::Slab;
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub type ObjectsReadGuard<'a, T> = RwLockReadGuard<'a, Slab<T>>;
pub type ObjectsWriteGuard<'a, T> = RwLockWriteGuard<'a, Slab<T>>;

struct ObjectsInner<T> {
    data: RwLock<Slab<T>>,
    delete_queue: Mutex<Vec<usize>>,
}

pub struct Objects<T>(Arc<ObjectsInner<T>>);

impl<T> Default for Objects<T> {
    fn default() -> Self {
        let inner = ObjectsInner {
            data: RwLock::new(Slab::new()),
            delete_queue: Mutex::new(Vec::new()),
        };
        Objects(Arc::new(inner))
    }
}

impl<T> Clone for Objects<T> {
    fn clone(&self) -> Self {
        Objects(self.0.clone())
    }
}

impl<T> Objects<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn cleanup(&self) {
        let mut write_guard = self
            .0
            .data
            .try_write()
            .expect("Objects::cleanup() not allowed here");
        let mut delete = self.0.delete_queue.lock().unwrap();
        for item in delete.drain(..) {
            write_guard.remove(item);
        }
    }
    pub fn insert(&self, object: T) -> Obj<T> {
        let mut write_guard = self
            .0
            .data
            .try_write()
            .expect("Objects::insert() not allowed here");
        let key = write_guard.insert(object);
        Obj(Arc::new(ObjInner {
            objects: self.clone(),
            key,
        }))
    }

    pub fn read(&self) -> ObjectsReadGuard<T> {
        self.0
            .data
            .try_read()
            .expect("Objects::read() not allowed here")
    }
    pub fn write(&self) -> ObjectsWriteGuard<T> {
        self.0
            .data
            .try_write()
            .expect("Objects::write() not allowed here")
    }
}

struct ObjInner<T> {
    objects: Objects<T>,
    key: usize,
}

impl<T> Drop for ObjInner<T> {
    fn drop(&mut self) {
        let mut delete_queue = self.objects.0.delete_queue.lock().unwrap();
        delete_queue.push(self.key);
    }
}

pub struct ObjReadGuard<'a, T> {
    read_guard: ObjectsReadGuard<'a, T>,
    key: usize,
}

impl<'a, T> Deref for ObjReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.read_guard.get(self.key).expect("missing object")
    }
}

pub struct ObjWriteGuard<'a, T> {
    write_guard: ObjectsWriteGuard<'a, T>,
    key: usize,
}

impl<'a, T> Deref for ObjWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.write_guard.get(self.key).expect("missing object")
    }
}
impl<'a, T> DerefMut for ObjWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.write_guard.get_mut(self.key).expect("missing object")
    }
}

pub struct Obj<T>(Arc<ObjInner<T>>);

impl<T> Clone for Obj<T> {
    fn clone(&self) -> Self {
        Obj(self.0.clone())
    }
}

impl<T> PartialEq for Obj<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl<T> Eq for Obj<T> {}

impl<T> Obj<T> {
    pub fn objects(&self) -> Objects<T> {
        self.0.objects.clone()
    }
    pub fn read(&self) -> ObjReadGuard<T> {
        let read_guard = self
            .0
            .objects
            .0
            .data
            .try_read()
            .expect("Obj::read() not allowed here");
        ObjReadGuard {
            read_guard,
            key: self.0.key,
        }
    }
    pub fn write(&self) -> ObjWriteGuard<T> {
        let write_guard = self
            .0
            .objects
            .0
            .data
            .try_write()
            .expect("Obj::write() not allowed here");
        ObjWriteGuard {
            write_guard,
            key: self.0.key,
        }
    }
}

pub struct CastObjReadGuard<'a, T: ?Sized, D> {
    _marker: PhantomData<D>,
    read_guard: ObjReadGuard<'a, Box<T>>,
}

impl<'a, T: Downcast + ?Sized, D: 'static> Deref for CastObjReadGuard<'a, T, D> {
    type Target = D;
    fn deref(&self) -> &D {
        let base: &T = self.read_guard.deref();
        base.as_any().downcast_ref().expect("wrong type")
    }
}

pub struct CastObjWriteGuard<'a, T: ?Sized, D> {
    _marker: PhantomData<D>,
    write_guard: ObjWriteGuard<'a, Box<T>>,
}

impl<'a, T: Downcast + ?Sized, D: 'static> Deref for CastObjWriteGuard<'a, T, D> {
    type Target = D;
    fn deref(&self) -> &D {
        let base: &T = self.write_guard.deref();
        base.as_any().downcast_ref().expect("wrong type")
    }
}
impl<'a, T: Downcast + ?Sized, D: 'static> DerefMut for CastObjWriteGuard<'a, T, D> {
    fn deref_mut(&mut self) -> &mut D {
        let base: &mut T = self.write_guard.deref_mut();
        base.as_any_mut().downcast_mut().expect("wrong type")
    }
}

pub struct CastObj<T: ?Sized, D>(Obj<Box<T>>, PhantomData<D>);

impl<T: Downcast + ?Sized, D> CastObj<T, D> {
    pub fn new(inner: Obj<Box<T>>) -> Self {
        CastObj(inner, PhantomData)
    }
    pub fn objects(&self) -> Objects<Box<T>> {
        self.0.objects()
    }
    pub fn read(&self) -> CastObjReadGuard<T, D> {
        CastObjReadGuard {
            _marker: PhantomData,
            read_guard: self.0.read(),
        }
    }
    pub fn write(&self) -> CastObjWriteGuard<T, D> {
        CastObjWriteGuard {
            _marker: PhantomData,
            write_guard: self.0.write(),
        }
    }
}
