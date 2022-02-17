use super::{Schedule, queue::Queue};
use parking_lot::RwLock;
use std::sync::atomic::{Ordering::SeqCst, AtomicUsize};
use std::sync::Arc;
use std::ops::{Deref, DerefMut};

struct Inner {
    queue: RwLock<Queue>,
    dirty: AtomicUsize
}

#[derive(Clone)]
pub struct Controller(Arc<Inner>, usize);

impl Controller {

    pub fn new(schedule: Box<dyn Schedule>) -> Self {
        Self(Arc::new(Inner {
            queue: RwLock::new(Queue::new(schedule)),
            dirty: AtomicUsize::new(0)
        }), 1)
    }

    pub fn changed(&mut self) -> bool {
        let ver = self.0.dirty.load(SeqCst);
        let old = self.1;

        self.1 = ver;
        old != ver
    }

    pub fn write(&mut self) -> Write {
        Write(self.0.queue.write(), &self.0.dirty)
    }

    pub fn read(&self) -> Read {
        Read(self.0.queue.read())
    }
}

pub struct Read<'a>(parking_lot::RwLockReadGuard<'a, Queue>);

pub struct Write<'a>(parking_lot::RwLockWriteGuard<'a, Queue>, &'a AtomicUsize);

impl<'a> Deref for Read<'a> {
    type Target = Queue;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<'a> Deref for Write<'a> {
    type Target = Queue;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<'a> DerefMut for Write<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl<'a> Drop for Write<'a> {
    fn drop(&mut self) {
        self.1.fetch_add(1, SeqCst);
    }
}