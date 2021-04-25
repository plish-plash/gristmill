use std::sync::Arc;

use vulkano::format::{Format, ClearValue};

use super::{RenderPassInfo, RenderContext, Renderer, subpass::*};
use crate::color;
use crate::geometry2d::Size;

// -------------------------------------------------------------------------------------------------

pub trait RenderPass: Sized {
    type Scene;
    fn info(&self) -> RenderPassInfo;
    fn set_dimensions(&mut self, dimensions: Size);
    fn render(&mut self, context: &mut RenderContext, scene: &mut Self::Scene);
}

// -------------------------------------------------------------------------------------------------

pub struct GeometryPass<T> where T: RenderSubpass<SubpassCategory=Geometry> {
    info: RenderPassInfo,
    subpass: T,
    clear_values: Vec<ClearValue>,
}

impl<T> GeometryPass<T> where T: RenderSubpass<SubpassCategory=Geometry> {
    pub fn new(renderer: &mut Renderer) -> GeometryPass<T> {
        Self::with_clear_color(renderer, color::black())
    }
    pub fn with_clear_color(renderer: &mut Renderer, clear_color: color::Color) -> GeometryPass<T> {
        let info = Arc::new(
            vulkano::single_pass_renderpass!(
                renderer.device(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: renderer.swapchain_format(),
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

        let subpass = T::new(&mut renderer.subpass_setup(info.clone(), 0));
        let clear_values = vec![color::encode_color(clear_color).into(), 1f32.into()];
        GeometryPass { info, subpass, clear_values }
    }
    pub fn subpass(&mut self) -> &mut T { &mut self.subpass }
}

impl<T> RenderPass for GeometryPass<T> where T: RenderSubpass<SubpassCategory=Geometry> {
    type Scene = T::Scene;
    fn info(&self) -> RenderPassInfo { self.info.clone() }
    fn set_dimensions(&mut self, dimensions: Size) {
        self.subpass.set_dimensions(dimensions);
    }
    fn render(&mut self, context: &mut RenderContext, scene: &mut Self::Scene) {
        self.subpass.pre_render(context, scene);
        context.builder.begin_render_pass(context.framebuffer.clone(), T::contents(), self.clear_values.clone()).unwrap();
        self.subpass.render(context, scene);
        context.builder.end_render_pass().unwrap();
    }
}

pub struct GeometryGuiPass<T1, T2> where T1: RenderSubpass<SubpassCategory=Geometry>, T2: RenderSubpass<SubpassCategory=Gui> {
    info: RenderPassInfo,
    subpass0: T1,
    subpass1: T2,
    clear_values: Vec<ClearValue>,
}

impl<T1, T2> GeometryGuiPass<T1, T2> where T1: RenderSubpass<SubpassCategory=Geometry>, T2: RenderSubpass<SubpassCategory=Gui> {
    pub fn new(renderer: &mut Renderer) -> GeometryGuiPass<T1, T2> {
        Self::with_clear_color(renderer, color::black())
    }
    pub fn with_clear_color(renderer: &mut Renderer, clear_color: color::Color) -> GeometryGuiPass<T1, T2> {
        let info = Arc::new(
            vulkano::ordered_passes_renderpass!(
                renderer.device(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: renderer.swapchain_format(),
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

        let subpass0 = T1::new(&mut renderer.subpass_setup(info.clone(), 0));
        let subpass1 = T2::new(&mut renderer.subpass_setup(info.clone(), 1));
        let clear_values = vec![color::encode_color(clear_color).into(), 1f32.into()];
        GeometryGuiPass { info, subpass0, subpass1, clear_values }
    }
    pub fn subpass0(&mut self) -> &mut T1 { &mut self.subpass0 }
    pub fn subpass1(&mut self) -> &mut T2 { &mut self.subpass1 }
}

impl<T1, T2> RenderPass for GeometryGuiPass<T1, T2> where T1: RenderSubpass<SubpassCategory=Geometry>, T2: RenderSubpass<SubpassCategory=Gui> {
    type Scene = (T1::Scene, T2::Scene);
    fn info(&self) -> RenderPassInfo { self.info.clone() }
    fn set_dimensions(&mut self, dimensions: Size) {
        self.subpass0.set_dimensions(dimensions);
        self.subpass1.set_dimensions(dimensions);
    }
    fn render(&mut self, context: &mut RenderContext, scene: &mut Self::Scene) {
        self.subpass0.pre_render(context, &mut scene.0);
        self.subpass1.pre_render(context, &mut scene.1);
        context.builder.begin_render_pass(context.framebuffer.clone(), T1::contents(), self.clear_values.clone()).unwrap();
        self.subpass0.render(context, &mut scene.0);
        context.builder.next_subpass(T2::contents()).unwrap();
        self.subpass1.render(context, &mut scene.1);
        context.builder.end_render_pass().unwrap();
    }
}
