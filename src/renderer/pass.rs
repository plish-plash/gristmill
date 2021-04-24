use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::format::{Format, ClearValue};
use vulkano::instance::QueueFamily;

use super::{FramebufferArc, RenderPassArc, RendererSetup, subpass::*};
use crate::color;
use crate::geometry2d::Size;

// -------------------------------------------------------------------------------------------------

pub trait RenderPass: Sized {
    type Scene;
    fn pass_info(&self) -> RenderPassArc;
    fn set_dimensions(&mut self, dimensions: Size);
    fn render(&mut self, scene: &mut Self::Scene, builder: &mut AutoCommandBufferBuilder, queue_family: QueueFamily, framebuffer: FramebufferArc, dynamic_state: &DynamicState);

    fn build_command_buffer(&mut self, queue_family: QueueFamily, framebuffer: FramebufferArc, dynamic_state: &DynamicState, scene: &mut Self::Scene) -> AutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(self.pass_info().device().clone(), queue_family).unwrap();
        self.render(scene, &mut builder, queue_family, framebuffer, dynamic_state);
        builder.build().unwrap()
    }
}

// -------------------------------------------------------------------------------------------------

pub struct GeometryPass<T> where T: RenderSubpass<SubpassCategory=Geometry> {
    render_pass_info: RenderPassArc,
    subpass: T,
    clear_values: Vec<ClearValue>,
}

impl<T> GeometryPass<T> where T: RenderSubpass<SubpassCategory=Geometry> {
    pub fn new(renderer_setup: &mut RendererSetup) -> GeometryPass<T> {
        Self::with_clear_color(renderer_setup, color::black())
    }
    pub fn with_clear_color(renderer_setup: &mut RendererSetup, clear_color: color::Color) -> GeometryPass<T> {
        let render_pass_info = Arc::new(
            vulkano::single_pass_renderpass!(
                renderer_setup.device(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: renderer_setup.swapchain_format(),
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: Format::D16Unorm,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {depth}
                }
            ).unwrap(),
        );

        let subpass = T::new(&mut renderer_setup.subpass_setup(render_pass_info.clone(), 0));
        let clear_values = vec![color::encode_color(clear_color).into(), 1f32.into()];
        GeometryPass { render_pass_info, subpass, clear_values }
    }
    pub fn subpass(&mut self) -> &mut T { &mut self.subpass }
}

impl<T> RenderPass for GeometryPass<T> where T: RenderSubpass<SubpassCategory=Geometry> {
    type Scene = T::Scene;
    fn pass_info(&self) -> RenderPassArc { self.render_pass_info.clone() }
    fn set_dimensions(&mut self, dimensions: Size) {
        self.subpass.set_dimensions(dimensions);
    }
    fn render(&mut self, scene: &mut Self::Scene, builder: &mut AutoCommandBufferBuilder, queue_family: QueueFamily, framebuffer: FramebufferArc, dynamic_state: &DynamicState) {
        self.subpass.pre_render(scene, builder, queue_family);
        builder.begin_render_pass(framebuffer, T::contents(), self.clear_values.clone()).unwrap();
        self.subpass.render(scene, builder, dynamic_state);
        builder.end_render_pass().unwrap();
    }
}

pub struct GeometryGuiPass<T1, T2> where T1: RenderSubpass<SubpassCategory=Geometry>, T2: RenderSubpass<SubpassCategory=Gui> {
    render_pass_info: RenderPassArc,
    subpass0: T1,
    subpass1: T2,
    clear_values: Vec<ClearValue>,
}

impl<T1, T2> GeometryGuiPass<T1, T2> where T1: RenderSubpass<SubpassCategory=Geometry>, T2: RenderSubpass<SubpassCategory=Gui> {
    pub fn new(renderer_setup: &mut RendererSetup) -> GeometryGuiPass<T1, T2> {
        Self::with_clear_color(renderer_setup, color::black())
    }
    pub fn with_clear_color(renderer_setup: &mut RendererSetup, clear_color: color::Color) -> GeometryGuiPass<T1, T2> {
        let render_pass_info = Arc::new(
            vulkano::ordered_passes_renderpass!(
                renderer_setup.device(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: renderer_setup.swapchain_format(),
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: Format::D16Unorm,
                        samples: 1,
                    }
                },
                passes: [
                    {
                        color: [color],
                        depth_stencil: {depth},
                        input: []
                    },
                    {
                        color: [color],
                        depth_stencil: {},
                        input: []
                    }
                ]
            ).unwrap(),
        );

        let subpass0 = T1::new(&mut renderer_setup.subpass_setup(render_pass_info.clone(), 0));
        let subpass1 = T2::new(&mut renderer_setup.subpass_setup(render_pass_info.clone(), 1));
        let clear_values = vec![color::encode_color(clear_color).into(), 1f32.into()];
        GeometryGuiPass { render_pass_info, subpass0, subpass1, clear_values }
    }
    pub fn subpass0(&mut self) -> &mut T1 { &mut self.subpass0 }
    pub fn subpass1(&mut self) -> &mut T2 { &mut self.subpass1 }
}

impl<T1, T2> RenderPass for GeometryGuiPass<T1, T2> where T1: RenderSubpass<SubpassCategory=Geometry>, T2: RenderSubpass<SubpassCategory=Gui> {
    type Scene = (T1::Scene, T2::Scene);
    fn pass_info(&self) -> RenderPassArc { self.render_pass_info.clone() }
    fn set_dimensions(&mut self, dimensions: Size) {
        self.subpass0.set_dimensions(dimensions);
        self.subpass1.set_dimensions(dimensions);
    }
    fn render(&mut self, scene: &mut Self::Scene, builder: &mut AutoCommandBufferBuilder, queue_family: QueueFamily, framebuffer: FramebufferArc, dynamic_state: &DynamicState) {
        self.subpass0.pre_render(&mut scene.0, builder, queue_family);
        self.subpass1.pre_render(&mut scene.1, builder, queue_family);
        builder.begin_render_pass(framebuffer, T1::contents(), self.clear_values.clone()).unwrap();
        self.subpass0.render(&scene.0, builder, dynamic_state);
        builder.next_subpass(T2::contents()).unwrap();
        self.subpass1.render(&scene.1, builder, dynamic_state);
        builder.end_render_pass().unwrap();
    }
}
