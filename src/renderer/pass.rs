use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::device::Device;
use vulkano::format::{Format, ClearValue};
use vulkano::instance::QueueFamily;

use super::{FramebufferArc, RenderPassArc, RendererSetup, RendererLoader, SubpassSetup, subpass::*};
use crate::geometry2d::Size;

// ------------------------------------------------------------------------------------------------

pub trait RenderPass: Sized {
    type Scene;
    fn new(renderer_setup: &mut RendererSetup) -> RenderPassInfo<Self>;
    fn set_dimensions(&mut self, dimensions: Size);
    fn render(&mut self, scene: &mut Self::Scene, builder: &mut AutoCommandBufferBuilder, queue_family: QueueFamily, framebuffer: FramebufferArc, dynamic_state: &DynamicState);
}

pub struct RenderPassInfo<T> where T: RenderPass {
    device: Arc<Device>,
    info: RenderPassArc,
    render_pass: T,
}

impl<T> RenderPassInfo<T> where T: RenderPass {
    pub fn new(device: Arc<Device>, info: RenderPassArc, render_pass: T) -> RenderPassInfo<T> {
        RenderPassInfo { device, info, render_pass }
    }

    pub fn raw_info(&self) -> RenderPassArc { self.info.clone() }
    // TODO consider replacing this with Deref so RenderPassInfo can act like its inner RenderPass.
    pub fn render_pass(&mut self) -> &mut T { &mut self.render_pass }
    
    pub fn subpass_setup<'a>(&'_ self, loader: &'a mut RendererLoader<'_>, index: u32) -> SubpassSetup<'a> {
        loader.subpass_setup(self.info.clone(), index)
    }

    pub fn set_dimensions(&mut self, dimensions: Size) {
        self.render_pass.set_dimensions(dimensions);
    }
    pub fn build_command_buffer(&mut self, queue_family: QueueFamily, framebuffer: FramebufferArc, dynamic_state: &DynamicState, scene: &mut T::Scene) -> AutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), queue_family).unwrap();
        self.render_pass.render(scene, &mut builder, queue_family, framebuffer, dynamic_state);
        builder.build().unwrap()
    }
}

// ------------------------------------------------------------------------------------------------

pub struct GeometryPass<T> where T: RenderSubpass<SubpassCategory=Geometry> {
    subpass: T,
    clear_values: Vec<ClearValue>,
}

impl<T> GeometryPass<T> where T: RenderSubpass<SubpassCategory=Geometry> {
    pub fn subpass(&mut self) -> &mut T { &mut self.subpass }
}

impl<T> RenderPass for GeometryPass<T> where T: RenderSubpass<SubpassCategory=Geometry> {
    type Scene = T::Scene;
    fn new(renderer_setup: &mut RendererSetup) -> RenderPassInfo<Self> {
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
        let clear_values = vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()];
        RenderPassInfo::new(
            renderer_setup.device(),
            render_pass_info,
            GeometryPass { subpass, clear_values }
        )
    }
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
    subpass0: T1,
    subpass1: T2,
    clear_values: Vec<ClearValue>,
}

impl<T1, T2> GeometryGuiPass<T1, T2> where T1: RenderSubpass<SubpassCategory=Geometry>, T2: RenderSubpass<SubpassCategory=Gui> {
    pub fn subpass0(&mut self) -> &mut T1 { &mut self.subpass0 }
    pub fn subpass1(&mut self) -> &mut T2 { &mut self.subpass1 }
}

impl<T1, T2> RenderPass for GeometryGuiPass<T1, T2> where T1: RenderSubpass<SubpassCategory=Geometry>, T2: RenderSubpass<SubpassCategory=Gui> {
    type Scene = (T1::Scene, T2::Scene);
    fn new(renderer_setup: &mut RendererSetup) -> RenderPassInfo<Self> {
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
        let clear_values = vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()];
        RenderPassInfo::new(
            renderer_setup.device(),
            render_pass_info,
            GeometryGuiPass { subpass0, subpass1, clear_values }
        )
    }
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
