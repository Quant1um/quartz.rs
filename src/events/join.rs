use super::EventStream;

pub struct Join<T, U>(EventStream<T>, EventStream<U>);

impl<T, U> Clone for Join<T, U> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<T, U> Join<T, U> {

    pub fn new(first: EventStream<T>, second: EventStream<U>) -> Self {
        Self(first, second)
    }
}

use rocket::response;
use rocket::Request;

impl<'r, T: 'static + Send + Sync + serde::Serialize, U: 'static + Send + Sync + serde::Serialize> response::Responder<'r, 'r> for Join<T, U> {
    fn respond_to(mut self, req: &'r Request<'_>) -> response::Result<'r> {
        use rocket::response::stream::{Event as SSEEvent, EventStream as SSEStream};
        use std::ops::Deref;
        use either::Either;

        let stream = async_stream::stream! {
            if let Some(data) = self.0.current() {
                yield SSEEvent::json(data.deref());
            }

            if let Some(data) = self.1.current() {
                yield SSEEvent::json(data.deref());
            }

            loop {
                let result = tokio::select! {
                    Some(data) = self.0.poll() => Either::Left(data), //TODO exit condition
                    Some(data) = self.1.poll() => Either::Right(data)
                };

                match result {
                    Either::Left(data) => yield SSEEvent::json(data.deref()),
                    Either::Right(data) => yield SSEEvent::json(data.deref())
                }
            }
        };

        SSEStream::from(stream).respond_to(req)
    }
}
