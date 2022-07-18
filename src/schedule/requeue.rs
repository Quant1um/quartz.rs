use std::collections::VecDeque;
use async_trait::async_trait;
use rand::{Rng, thread_rng};
use crate::schedule::Schedule;
use crate::Track;

pub struct Requeue {
    queue: VecDeque<Track>
}

impl Requeue {

    pub fn new(tracks: impl IntoIterator<Item = Track>) -> Self {
        Self {
            queue: tracks.into_iter().collect()
        }
    }

    pub fn shuffle(&mut self) {
        for _ in 0..100 {
            self.shift();
        }
    }

    pub fn shift(&mut self) {
        let next = self.queue.pop_front().expect("queue should not be empty");
        let position = thread_rng().gen_range::<f32, _>(0.0..0.6);
        let index = (self.queue.len() as f32 * (1.0 - position)) as usize;
        self.queue.insert(index, next);
    }
}

#[async_trait]
impl Schedule for Requeue {

    async fn next(&mut self) -> Track {
        self.shift();
        self.queue.front().expect("empty queue?").clone()
    }
}