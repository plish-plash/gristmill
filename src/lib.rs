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

pub use emath as math;
use emath::{Rect, Vec2};

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

pub trait Buffer<T>: Extend<T> {
    fn new() -> Self;
    fn clear(&mut self);
    fn push(&mut self, value: T);
}

pub trait Pipeline {
    type Context;
    type Material: Clone + PartialEq;
    type Instance;
    type InstanceBuffer: Buffer<Self::Instance>;
    type Camera: Default;
    fn transform(camera: &Self::Camera, instance: Self::Instance) -> Self::Instance;
    fn draw(
        &mut self,
        context: &mut Self::Context,
        camera: &Self::Camera,
        material: &Self::Material,
        instances: &mut Self::InstanceBuffer,
    );
}

pub struct Batcher<'a, P: Pipeline> {
    pipeline: &'a mut P,
    context: &'a mut P::Context,
    batch: &'a mut P::InstanceBuffer,
    material: Option<P::Material>,
    camera: P::Camera,
}

impl<'a, P: Pipeline> Batcher<'a, P> {
    pub fn new(
        pipeline: &'a mut P,
        context: &'a mut P::Context,
        instances: &'a mut P::InstanceBuffer,
    ) -> Self {
        Batcher {
            pipeline,
            context,
            batch: instances,
            material: None,
            camera: P::Camera::default(),
        }
    }
    pub fn set_camera(&mut self, camera: P::Camera) {
        self.camera = camera;
    }
    pub fn flush(&mut self) {
        if let Some(material) = self.material.as_ref() {
            self.pipeline
                .draw(self.context, &P::Camera::default(), material, self.batch);
        }
        self.batch.clear();
    }
    pub fn draw(&mut self, material: &P::Material, instance: P::Instance) {
        if self.material.as_ref() != Some(material) {
            self.flush();
            self.material = Some(material.clone());
        }
        self.batch.push(P::transform(&self.camera, instance));
    }
    pub fn draw_all<I>(&mut self, material: &P::Material, instances: I)
    where
        I: IntoIterator<Item = P::Instance>,
    {
        if self.material.as_ref() != Some(material) {
            self.flush();
            self.material = Some(material.clone());
        }
        self.batch.extend(
            instances
                .into_iter()
                .map(|instance| P::transform(&self.camera, instance)),
        );
    }
}
impl<'a, P: text::TextPipeline> Batcher<'a, P> {
    pub fn set_camera_and_clip(&mut self, camera: P::Camera, clip: Option<Rect>) {
        self.flush();
        self.pipeline.set_clip(&mut self.context, clip);
        self.set_camera(camera);
    }
}
impl<'a, P: Pipeline> Drop for Batcher<'a, P> {
    fn drop(&mut self) {
        self.flush();
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
