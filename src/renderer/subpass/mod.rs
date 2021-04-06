pub mod optional;
pub mod example;
pub mod gui;

// ------------------------------------------------------------------------------------------------

use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState, SubpassContents};
use vulkano::instance::QueueFamily;

use super::{PipelineArc, SubpassSetup};
use crate::gui::geometry::Size;

pub use optional::Optional;

// ------------------------------------------------------------------------------------------------

pub trait Pipeline {
    fn raw_pipeline(&self) -> PipelineArc;
}

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
