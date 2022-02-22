mod join;

use tokio::sync::watch::*;
use std::ops::Deref;
use std::sync::Arc;

pub use join::*;

pub struct EventStream<T>(Receiver<Option<Arc<T>>>);
pub struct EventHandle<T>(Sender<Option<Arc<T>>>);

impl<T> Clone for EventStream<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> EventStream<T> {

    pub fn new() -> (Self, EventHandle<T>) {
        let (sender, receiver) = channel(None);
        (Self(receiver), EventHandle(sender))
    }

    pub fn current(&mut self) -> Option<Arc<T>> {
        self.0.borrow_and_update().clone()
    }

    pub async fn poll(&mut self) -> Option<Arc<T>> {
        loop {
            self.0.changed().await.ok()?;

            match self.0.borrow_and_update().deref() {
                Some(data) => return Some(data.clone()),
                None => continue
            }
        }
    }

    pub fn join<U>(self, with: EventStream<U>) -> join::Join<T, U> {
        join::Join::new(self, with)
    }
}

impl<T> EventHandle<T> {

    pub fn current(&self) -> Option<Arc<T>> {
        self.0.borrow().clone()
    }

    pub fn send(&mut self, data: T) {
        let _ = self.0.send(Some(Arc::new(data)));
    }
}

use rocket::response;
use rocket::Request;

impl<'r, T: 'static + Send + Sync + serde::Serialize> response::Responder<'r, 'r> for EventStream<T> {
    fn respond_to(mut self, req: &'r Request<'_>) -> response::Result<'r> {
        use rocket::response::stream::{Event as SSEEvent, EventStream as SSEStream};

        let stream = async_stream::stream! {
            if let Some(data) = self.current() {
                yield SSEEvent::json(data.deref());
            }

            while let Some(data) = self.poll().await {
                yield SSEEvent::json(data.deref());
            }
        };

        SSEStream::from(stream).respond_to(req)
    }
}