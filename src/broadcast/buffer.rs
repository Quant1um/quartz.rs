use tokio::sync::Notify;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::Arc;
use std::time::Duration;
use super::codec::Page;
use std::collections::vec_deque::Iter;

pub struct Buffer(Arc<Shared>);
pub struct Receiver(Arc<Shared>, usize);

impl Buffer {

    pub fn new(length: Duration) -> (Self, Receiver) {
        let shared =  Arc::new(Shared::new(length));

        (Self(shared.clone()), Receiver::new(shared))
    }

    pub fn push(&self, page: Page) {
        {
            let mut queue = self.0.queue.write();
            queue.push(page);
        }

        self.0.version.fetch_add(1, SeqCst);
        self.0.notify.notify_waiters();
    }

    pub fn receivers(&self) -> usize {
        Arc::strong_count(&self.0) - 1
    }
}

impl Receiver {

    fn new(shared: Arc<Shared>) -> Self {
        let version = shared.version.load(SeqCst);
        Self(shared, version)
    }

    pub fn buffered(&mut self) -> Vec<Page> {
        let queue = self.0.queue.read();
        self.1 = self.0.version.load(SeqCst);
        queue.iter()
            .map(|f| f.clone())
            .collect()
    }

    pub async fn poll(&mut self) -> Page {
        loop {
            let notified = self.0.notify.notified();
            let version = self.0.version.load(SeqCst);
            if version != self.1 {
                self.1 = version;

                let queue = self.0.queue.read();
                return queue.latest()
                    .expect("broadcast buffer: no page after notify?")
                    .clone()
            }

            notified.await;
        }
    }

    pub fn receivers(&self) -> usize {
        Arc::strong_count(&self.0) - 1
    }
}

impl Clone for Receiver {
    fn clone(&self) -> Self {
        Self::new(self.0.clone())
    }
}

struct Queue {
    queue: VecDeque<Page>,
    length: Duration,
    max_length: Duration
}

impl Queue {

    fn new(max_length: Duration) -> Self {
        Self {
            queue: VecDeque::new(),
            length: Duration::from_nanos(0),
            max_length
        }
    }

    fn push(&mut self, page: Page) {
        while self.length > self.max_length {
            match self.queue.pop_front() {
                Some(page) => {
                    self.length -= page.duration;
                },

                None => panic!("broadcast buffer: queue invariant broken?")
            }
        }

        self.length += page.duration;
        self.queue.push_back(page);
    }

    fn iter(&self) -> Iter<Page> {
        self.queue.iter()
    }

    fn latest(&self) -> Option<&Page> {
        self.queue.back()
    }
}

struct Shared {
    queue: RwLock<Queue>,
    version: AtomicUsize,
    notify: Notify
}

impl Shared {

    fn new(length: Duration) -> Self {
        Self {
            queue: RwLock::new(Queue::new(length)),
            version: AtomicUsize::new(0),
            notify: Notify::new()
        }
    }
}