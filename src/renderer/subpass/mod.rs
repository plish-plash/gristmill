pub mod example;
pub mod gui;

// -------------------------------------------------------------------------------------------------

use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState, SubpassContents};
use vulkano::instance::QueueFamily;

use super::SubpassSetup;
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
    fn pre_render(&mut self, scene: &mut Self::Scene, builder: &mut AutoCommandBufferBuilder, queue_family: QueueFamily);
    fn render(&mut self, scene: &Self::Scene, builder: &mut AutoCommandBufferBuilder, dynamic_state: &DynamicState);
}

// -------------------------------------------------------------------------------------------------

impl<T> RenderSubpass for Option<T> where T: RenderSubpass {
    type SubpassCategory = T::SubpassCategory;
    type Scene = Option<T::Scene>;
    fn contents() -> SubpassContents { SubpassContents::Inline }
    fn new(_subpass_setup: &mut SubpassSetup) -> Self { None }
    fn set_dimensions(&mut self, dimensions: Size) {
        if let Some(inner) = self.as_mut() {
            inner.set_dimensions(dimensions);
        }
    }
    fn pre_render(&mut self, scene: &mut Self::Scene, builder: &mut AutoCommandBufferBuilder, queue_family: QueueFamily) {
        if let Some(inner) = self.as_mut() {
            inner.pre_render(scene.as_mut().expect("Scene must be Some if optional subpass is Some"), builder, queue_family);
        }
    }
    fn render(&mut self, scene: &Self::Scene, builder: &mut AutoCommandBufferBuilder, dynamic_state: &DynamicState) {
        if let Some(inner) = self.as_mut() {
            inner.render(scene.as_ref().expect("Scene must be Some if optional subpass is Some"), builder, dynamic_state);
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
