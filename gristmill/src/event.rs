
pub trait Event {

}

pub struct EventSystem<E> where E: Event {
    queue: Vec<E>,
}

impl<E> EventSystem<E> where E: Event {
    pub fn new() -> EventSystem<E> {
        EventSystem {
            queue: Vec::new()
        }
    }
    pub fn fire_event(&mut self, event: E) {
        self.queue.push(event);
    }
    pub fn dispatch_queue<F>(&mut self, mut handler: F) where F: FnMut(E) {
        for event in self.queue.drain(..) {
            handler(event);
        }
    }
    pub fn discard_queue(&mut self) {
        self.queue.clear();
    }
}
