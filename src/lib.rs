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

use std::collections::BTreeMap;

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

pub struct Batcher<Params, Instance>(Vec<(Params, Vec<Instance>)>);

impl<Params, Instance> Batcher<Params, Instance>
where
    Params: Eq + PartialOrd,
{
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clear(&mut self) {
        for (_, batch) in self.0.iter_mut() {
            batch.clear();
        }
    }
    fn get_batch(&mut self, params: Params) -> &mut Vec<Instance> {
        for i in 0..self.0.len() {
            let batch = &mut self.0[i];
            if batch.0 == params {
                return &mut self.0[i].1;
            }
            if batch.0 > params {
                self.0.insert(i, (params, Vec::new()));
                return &mut self.0[i].1;
            }
        }
        self.0.push((params, Vec::new()));
        let (_, batch) = self.0.last_mut().unwrap();
        batch
    }
    pub fn add(&mut self, params: Params, instance: Instance) {
        self.get_batch(params).push(instance);
    }
    pub fn batches(&self) -> impl Iterator<Item = &(Params, Vec<Instance>)> {
        self.0.iter().filter(|(_, batch)| !batch.is_empty())
    }
}
impl<Params, Instance> Default for Batcher<Params, Instance> {
    fn default() -> Self {
        Batcher(Vec::new())
    }
}

pub struct Stage<Layer, Camera, Params, Instance> {
    layers: BTreeMap<Layer, Batcher<Params, Instance>>,
    cameras: BTreeMap<Layer, Camera>,
}

impl<Layer, Camera, Params, Instance> Stage<Layer, Camera, Params, Instance>
where
    Layer: Ord,
    Params: Eq + PartialOrd,
{
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clear(&mut self) {
        for batcher in self.layers.values_mut() {
            batcher.clear();
        }
    }
    pub fn get_camera(&self, layer: &Layer) -> Option<&Camera> {
        self.cameras.get(layer)
    }
    pub fn set_camera(&mut self, layer: Layer, camera: Camera) {
        self.cameras.insert(layer, camera);
    }
    pub fn get_layer(&mut self, layer: Layer) -> &mut Batcher<Params, Instance> {
        self.layers.entry(layer).or_default()
    }
    pub fn add(&mut self, layer: Layer, params: Params, instance: Instance) {
        self.get_layer(layer).add(params, instance);
    }
    pub fn draw<R>(&mut self, renderer: &mut R, context: &mut R::Context)
    where
        R: Renderer<Camera = Camera, Params = Params, Instance = Instance>,
    {
        for (layer, batcher) in self.layers.iter() {
            if let Some(camera) = self.cameras.get(layer) {
                renderer.set_camera(context, camera);
            }
            for (params, batch) in batcher.batches() {
                renderer.draw(context, params, batch.as_slice());
            }
        }
        self.clear();
    }
}
impl<Layer, Camera, Params, Instance> Default for Stage<Layer, Camera, Params, Instance> {
    fn default() -> Self {
        Stage {
            layers: BTreeMap::new(),
            cameras: BTreeMap::new(),
        }
    }
}

pub trait Renderer {
    type Context;
    type Camera;
    type Params: Eq + PartialOrd;
    type Instance;
    fn set_camera(&mut self, context: &mut Self::Context, camera: &Self::Camera);
    fn draw(
        &mut self,
        context: &mut Self::Context,
        params: &Self::Params,
        instances: &[Self::Instance],
    );
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
