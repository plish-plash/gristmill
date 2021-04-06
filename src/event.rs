
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
    pub fn dispatch_queue<H>(&mut self, handler: &mut H, context: &mut H::Context) where H: EventHandler<E> {
        while !self.queue.is_empty() {
            for event in self.queue.split_off(0) {
                handler.handle_event(self, context, event);
            }
        }
    }
    pub fn discard_queue(&mut self) {
        self.queue.clear();
    }
}

pub trait EventHandler<E: Event> {
    type Context;
    fn handle_event(&mut self, system: &mut EventSystem<E>, context: &mut Self::Context, event: E);
}