use std::sync::{Arc, Mutex};

pub mod asset;
pub mod color;
pub mod console;
pub mod gui;
pub mod input;
pub mod lang;
pub mod render2d;
pub mod sprite;
pub mod text;

pub use emath as math;

pub type Handle = Arc<dyn std::any::Any>;

pub struct QueueBuilder<T> {
    items: Vec<T>,
    barriers: Vec<usize>,
    current_barrier: usize,
}

impl<T> QueueBuilder<T> {
    pub fn new() -> Self {
        QueueBuilder {
            items: Vec::new(),
            barriers: Vec::new(),
            current_barrier: 0,
        }
    }
    pub fn reset(&mut self) {
        self.items.clear();
        self.barriers.clear();
        self.barriers.push(0);
        self.current_barrier = 0;
    }
    pub fn queue(&mut self, item: T) {
        self.items.push(item);
    }
    pub fn barrier(&mut self) {
        self.barriers.push(self.items.len());
    }
    pub fn draw_next(&mut self) -> (usize, &[T]) {
        let previous_barrier = self.current_barrier;
        self.current_barrier += 1;
        let start = self.barriers[previous_barrier];
        let end = self.barriers[self.current_barrier];
        (previous_barrier, &self.items[start..end])
    }
}

pub trait Renderer {
    type DrawCall;
    fn draw(&mut self, draw_call: Self::DrawCall);
}

pub trait Drawable {
    type Renderer: Renderer;
    fn draw_next(
        &mut self,
        renderer: &mut Self::Renderer,
    ) -> <Self::Renderer as Renderer>::DrawCall;
}

pub struct RenderQueue {
    queue: Mutex<Vec<usize>>,
}

impl RenderQueue {
    pub fn new() -> Arc<Self> {
        Arc::new(RenderQueue {
            queue: Mutex::new(Vec::new()),
        })
    }
    pub fn get_dispatcher(self: &Arc<Self>, index: usize) -> Dispatcher {
        Dispatcher {
            index,
            dispatcher: self.clone(),
        }
    }

    fn queue(&self, index: usize) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(index);
    }
    pub fn draw<R: Renderer>(
        &self,
        renderer: &mut R,
        mut drawables: Vec<&mut dyn Drawable<Renderer = R>>,
    ) {
        let mut queue = self.queue.lock().unwrap();
        for index in queue.drain(..) {
            let draw_call = drawables[index].draw_next(renderer);
            renderer.draw(draw_call);
        }
    }
}

#[derive(Clone)]
pub struct Dispatcher {
    index: usize,
    dispatcher: Arc<RenderQueue>,
}

impl Dispatcher {
    pub fn dispatch(&self) {
        self.dispatcher.queue(self.index);
    }
}
