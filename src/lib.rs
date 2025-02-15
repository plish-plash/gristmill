pub mod asset;
pub mod color;
pub mod gui;
pub mod input;
pub mod lang;
pub mod logger;
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

pub struct Batch<Instance>(Vec<Instance>);

impl<Instance> Batch<Instance> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clear(&mut self) {
        self.0.clear();
    }
    pub fn add(&mut self, instance: Instance) {
        self.0.push(instance);
    }
    pub fn append(&mut self, mut batch: Batch<Instance>) {
        self.0.append(&mut batch.0);
    }
    pub fn as_slice(&self) -> &[Instance] {
        self.0.as_slice()
    }
}
impl<Instance> Default for Batch<Instance> {
    fn default() -> Self {
        Batch(Vec::new())
    }
}
impl<Instance> Extend<Instance> for Batch<Instance> {
    fn extend<T: IntoIterator<Item = Instance>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

pub struct Batcher<Params, Instance>(HashMap<Params, Batch<Instance>>);

impl<Params, Instance> Batcher<Params, Instance>
where
    Params: Eq + Hash,
{
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clear(&mut self) {
        for batch in self.0.values_mut() {
            batch.clear();
        }
    }
    pub fn get_mut(&mut self, params: Params) -> &mut Batch<Instance> {
        self.0.entry(params).or_default()
    }
    pub fn add(&mut self, params: Params, instance: Instance) {
        self.get_mut(params).add(instance);
    }
    pub fn batches(&self) -> impl Iterator<Item = (&Params, &Batch<Instance>)> {
        self.0.iter()
    }
}
impl<Params, Instance> Default for Batcher<Params, Instance> {
    fn default() -> Self {
        Batcher(HashMap::new())
    }
}

pub struct LayerBatcher<Layer, Params, Instance>(BTreeMap<Layer, Batcher<Params, Instance>>);

impl<Layer, Params, Instance> LayerBatcher<Layer, Params, Instance>
where
    Layer: Ord,
    Params: Eq + Hash,
{
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clear(&mut self) {
        for batcher in self.0.values_mut() {
            batcher.clear();
        }
    }
    pub fn get_mut(&mut self, layer: Layer) -> &mut Batcher<Params, Instance> {
        self.0.entry(layer).or_default()
    }
    pub fn add(&mut self, layer: Layer, params: Params, instance: Instance) {
        self.get_mut(layer).add(params, instance);
    }
    pub fn batches(&self) -> impl Iterator<Item = (&Params, &Batch<Instance>)> {
        self.0.values().flat_map(|batcher| batcher.batches())
    }
}
impl<Layer, Params, Instance> Default for LayerBatcher<Layer, Params, Instance> {
    fn default() -> Self {
        LayerBatcher(BTreeMap::new())
    }
}

pub trait Renderer {
    type Context;
    type Params: Eq + Hash + 'static;
    type Instance: 'static;
    fn draw(
        &mut self,
        context: &mut Self::Context,
        params: &Self::Params,
        instances: &[Self::Instance],
    );
    fn draw_batch(
        &mut self,
        context: &mut Self::Context,
        params: &Self::Params,
        batch: &Batch<Self::Instance>,
    ) {
        self.draw(context, params, batch.as_slice())
    }
    fn draw_batches<'a, I>(&mut self, context: &mut Self::Context, batches: I)
    where
        I: IntoIterator<Item = (&'a Self::Params, &'a Batch<Self::Instance>)>,
    {
        for (params, batch) in batches {
            self.draw_batch(context, params, batch);
        }
    }
}

#[derive(Default, Clone)]
pub struct DrawMetrics(u32);

impl DrawMetrics {
    pub fn new() -> Self {
        DrawMetrics::default()
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
