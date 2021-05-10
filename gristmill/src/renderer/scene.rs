use vulkano::command_buffer::SubpassContents;

use super::{LoadContext, RenderContext};
use crate::geometry2d::Size;

// -------------------------------------------------------------------------------------------------

pub trait SceneRendererType {}

pub struct Depth;
impl SceneRendererType for Depth {}

pub struct Geometry2D;
impl SceneRendererType for Geometry2D {}

pub struct Geometry3D;
impl SceneRendererType for Geometry3D {}

pub struct PostProcessing;
impl SceneRendererType for PostProcessing {}

pub trait SceneRenderer {
    type RenderType: SceneRendererType;
    type Scene;
    fn contents() -> SubpassContents;
    fn new(context: &mut LoadContext) -> Self;
    fn set_dimensions(&mut self, _dimensions: Size) {}
    fn pre_render(&mut self, context: &mut RenderContext, scene: &mut Self::Scene);
    fn render(&mut self, context: &mut RenderContext, scene: &mut Self::Scene);
}

// -------------------------------------------------------------------------------------------------

impl<T> SceneRenderer for Option<T> where T: SceneRenderer {
    type RenderType = T::RenderType;
    type Scene = Option<T::Scene>;
    fn contents() -> SubpassContents { T::contents() }
    fn new(_context: &mut LoadContext) -> Self { None }
    fn set_dimensions(&mut self, dimensions: Size) {
        if let Some(inner) = self.as_mut() {
            inner.set_dimensions(dimensions);
        }
    }
    fn pre_render(&mut self, context: &mut RenderContext, scene: &mut Self::Scene) {
        if let Some(inner) = self.as_mut() {
            inner.pre_render(context, scene.as_mut().expect("Scene must be Some if optional renderer is Some"));
        }
    }
    fn render(&mut self, context: &mut RenderContext, scene: &mut Self::Scene) {
        if let Some(inner) = self.as_mut() {
            inner.render(context, scene.as_mut().expect("Scene must be Some if optional renderer is Some"));
        }
    }
}

pub trait SceneRendererOptionExt {
    fn create_inner(&mut self, context: &mut LoadContext);
    fn destroy_inner(&mut self);
}

impl<T> SceneRendererOptionExt for Option<T> where T: SceneRenderer {
    fn create_inner(&mut self, context: &mut LoadContext) {
        if self.is_some() {
            panic!("inner renderer already exists");
        }
        *self = Some(T::new(context));
    }
    fn destroy_inner(&mut self) {
        if self.is_none() {
            panic!("inner renderer does not exist");
        }
        *self = None;
    }
}
