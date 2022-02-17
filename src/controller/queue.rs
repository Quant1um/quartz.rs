use super::{Schedule, Track};
use std::collections::VecDeque;

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

    pub fn next(&mut self) -> &Track {
        if self.queue.is_empty() {
            self.queue.push_back(self.schedule.next());
        }

        self.queue.front().unwrap()
    }

    pub fn remove(&mut self, id: usize) -> Option<Track> {
        self.queue.remove(id)
    }

    pub fn current(&self) -> Option<&Track> {
        self.queue.front()
    }
}