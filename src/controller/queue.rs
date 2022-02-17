use super::{Schedule, Track};
use std::collections::VecDeque;

const MINIMUM_IN_QUEUE: usize = 2; // including preload

pub struct Queue {
    schedule: Box<dyn Schedule>,
    queue: VecDeque<Track>
}

impl Queue {

    pub fn new(schedule: Box<dyn Schedule>) -> Self {
        Self {
            schedule,
            queue: VecDeque::new()
        }
    }

    pub fn skip(&mut self) -> Track {
        while self.queue.len() < MINIMUM_IN_QUEUE + 1 {
            self.queue.push_back(self.schedule.next());
        }

        self.queue.pop_front().unwrap()
    }

    pub fn remove(&mut self, id: usize) -> Option<Track> {
        self.queue.remove(id)
    }

    pub fn current(&self) -> Option<&Track> {
        self.queue.front()
    }
}