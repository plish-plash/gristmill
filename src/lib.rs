pub mod asset;
pub mod color;
pub mod console;
pub mod gui;
pub mod input;
pub mod lang;
pub mod particles;
pub mod scene2d;
pub mod style;
pub mod text;

use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

pub use emath as math;
use emath::Vec2;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Size { width, height }
    }
    pub fn to_vec2(self) -> Vec2 {
        Vec2 {
            x: self.width as f32,
            y: self.height as f32,
        }
    }
}

pub trait Renderer {
    type Context;
    type Params;
    type Instance;
    fn draw(
        &mut self,
        context: &mut Self::Context,
        params: &Self::Params,
        instances: &[Self::Instance],
    );
}

struct Batch<Instance>(Vec<Instance>);

impl<Instance> Default for Batch<Instance> {
    fn default() -> Self {
        Batch(Vec::new())
    }
}

struct Layer<Params, Instance>(HashMap<Params, Batch<Instance>>);

impl<Params, Instance> Default for Layer<Params, Instance> {
    fn default() -> Self {
        Layer(HashMap::new())
    }
}

pub struct Scene<L, P, I>(BTreeMap<L, Layer<P, I>>);

impl<L: Ord, P: Eq + Hash, I> Scene<L, P, I> {
    pub fn new() -> Self {
        Scene(BTreeMap::new())
    }
    fn get_batch(&mut self, layer: L, params: P) -> &mut Batch<I> {
        let layer = self.0.entry(layer).or_default();
        layer.0.entry(params).or_default()
    }
    pub fn queue(&mut self, layer: L, params: P, instance: I) {
        self.get_batch(layer, params).0.push(instance);
    }
    pub fn queue_all<Iter>(&mut self, layer: L, params: P, instances: Iter)
    where
        Iter: Iterator<Item = I>,
    {
        self.get_batch(layer, params).0.extend(instances);
    }
    pub fn draw<R>(&mut self, renderer: &mut R, context: &mut R::Context)
    where
        R: Renderer<Params = P, Instance = I>,
    {
        for layer in self.0.values_mut() {
            for (params, batch) in layer.0.iter_mut() {
                if !batch.0.is_empty() {
                    renderer.draw(context, params, &batch.0);
                    batch.0.clear();
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct DrawMetrics(u32);

impl DrawMetrics {
    pub fn new() -> Self {
        DrawMetrics(0)
    }
    pub fn draw_call(&mut self) {
        self.0 += 1;
    }
    pub fn end_render(&mut self) -> DrawMetrics {
        let frame = self.clone();
        self.0 = 0;
        frame
    }
}
impl std::ops::Add for DrawMetrics {
    type Output = DrawMetrics;
    fn add(self, rhs: Self) -> Self::Output {
        DrawMetrics(self.0 + rhs.0)
    }
}
impl std::fmt::Display for DrawMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Draw calls: {}", self.0)
    }
}
