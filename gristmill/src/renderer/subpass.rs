use vulkano::command_buffer::SubpassContents;

use super::{SubpassSetup, RenderContext};
use crate::geometry2d::Size;

// -------------------------------------------------------------------------------------------------

pub trait RenderSubpassCategory {}

pub struct Depth;
impl RenderSubpassCategory for Depth {}

pub struct Geometry;
impl RenderSubpassCategory for Geometry {}

pub struct Gui;
impl RenderSubpassCategory for Gui {}

pub struct PostProcessing;
impl RenderSubpassCategory for PostProcessing {}

pub trait RenderSubpass {
    type SubpassCategory: RenderSubpassCategory;
    type Scene;
    fn contents() -> SubpassContents;
    fn new(subpass_setup: &mut SubpassSetup) -> Self;
    fn set_dimensions(&mut self, _dimensions: Size) {}
    fn pre_render(&mut self, context: &mut RenderContext, scene: &mut Self::Scene);
    fn render(&mut self, context: &mut RenderContext, scene: &mut Self::Scene);
}

// -------------------------------------------------------------------------------------------------

impl<T> RenderSubpass for Option<T> where T: RenderSubpass {
    type SubpassCategory = T::SubpassCategory;
    type Scene = Option<T::Scene>;
    fn contents() -> SubpassContents { T::contents() }
    fn new(_subpass_setup: &mut SubpassSetup) -> Self { None }
    fn set_dimensions(&mut self, dimensions: Size) {
        if let Some(inner) = self.as_mut() {
            inner.set_dimensions(dimensions);
        }
    }
    fn pre_render(&mut self, context: &mut RenderContext, scene: &mut Self::Scene) {
        if let Some(inner) = self.as_mut() {
            inner.pre_render(context, scene.as_mut().expect("Scene must be Some if optional subpass is Some"));
        }
    }
    fn render(&mut self, context: &mut RenderContext, scene: &mut Self::Scene) {
        if let Some(inner) = self.as_mut() {
            inner.render(context, scene.as_mut().expect("Scene must be Some if optional subpass is Some"));
        }
    }
}

pub trait RenderSubpassOptionExt {
    fn create_inner(&mut self, subpass_setup: &mut SubpassSetup);
    fn destroy_inner(&mut self);
}

impl<T> RenderSubpassOptionExt for Option<T> where T: RenderSubpass {
    fn create_inner(&mut self, subpass_setup: &mut SubpassSetup) {
        if self.is_some() {
            panic!("inner subpass already exists");
        }
        *self = Some(T::new(subpass_setup));
    }
    fn destroy_inner(&mut self) {
        if self.is_none() {
            panic!("inner subpass does not exist");
        }
        *self = None;
    }
}
