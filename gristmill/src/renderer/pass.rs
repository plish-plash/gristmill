use std::sync::Arc;

use vulkano::format::{Format, ClearValue};

use super::{RenderPassArc, RenderContext, RenderLoader, LoadRef, scene::*};
use crate::color;
use crate::geometry2d::Size;

// -------------------------------------------------------------------------------------------------

pub trait RenderPass: Sized {
    type Scene;
    fn info(&self) -> RenderPassArc;
    fn set_dimensions(&mut self, dimensions: Size);
    fn render(&mut self, context: &mut RenderContext, scene: &mut Self::Scene);
}

// -------------------------------------------------------------------------------------------------

pub struct RenderPass3D<T> where T: SceneRenderer<RenderType=Geometry3D> {
    info: RenderPassArc,
    clear_values: Vec<ClearValue>,
    scene_render: T,
}

impl<T> RenderPass3D<T> where T: SceneRenderer<RenderType=Geometry3D> {
    pub fn new(loader: &mut RenderLoader) -> RenderPass3D<T> {
        Self::with_clear_color(loader, color::black())
    }
    pub fn with_clear_color(loader: &mut RenderLoader, clear_color: color::Color) -> RenderPass3D<T> {
        let info = Arc::new(
            vulkano::single_pass_renderpass!(
                loader.device(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: loader.swapchain_format(),
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

        let clear_values = vec![color::encode_color(clear_color).into(), 1f32.into()];
        let scene_render = T::new(&mut loader.load_context(info.clone(), 0));
        RenderPass3D { info, clear_values, scene_render }
    }
    pub fn scene_render<'a>(&'a mut self, loader: &'a mut RenderLoader) -> LoadRef<'a, T> { loader.load_ref_subpass(self.info.clone(), 0, &mut self.scene_render) }
}

impl<T> RenderPass for RenderPass3D<T> where T: SceneRenderer<RenderType=Geometry3D> {
    type Scene = T::Scene;
    fn info(&self) -> RenderPassArc { self.info.clone() }
    fn set_dimensions(&mut self, dimensions: Size) {
        self.scene_render.set_dimensions(dimensions);
    }
    fn render(&mut self, context: &mut RenderContext, scene: &mut Self::Scene) {
        self.scene_render.pre_render(context, scene);
        context.builder.begin_render_pass(context.framebuffer.clone(), T::contents(), self.clear_values.clone()).unwrap();
        self.scene_render.render(context, scene);
        context.builder.end_render_pass().unwrap();
    }
}

pub struct RenderPass3D2D<T0, T1> where T0: SceneRenderer<RenderType=Geometry3D>, T1: SceneRenderer<RenderType=Geometry2D> {
    info: RenderPassArc,
    clear_values: Vec<ClearValue>,
    scene_render0: T0,
    scene_render1: T1,
}

impl<T0, T1> RenderPass3D2D<T0, T1> where T0: SceneRenderer<RenderType=Geometry3D>, T1: SceneRenderer<RenderType=Geometry2D> {
    pub fn new(loader: &mut RenderLoader) -> RenderPass3D2D<T0, T1> {
        Self::with_clear_color(loader, color::black())
    }
    pub fn with_clear_color(loader: &mut RenderLoader, clear_color: color::Color) -> RenderPass3D2D<T0, T1> {
        let info = Arc::new(
            vulkano::ordered_passes_renderpass!(
                loader.device(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: loader.swapchain_format(),
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

        let clear_values = vec![color::encode_color(clear_color).into(), 1f32.into()];
        let scene_render0 = T0::new(&mut loader.load_context(info.clone(), 0));
        let scene_render1 = T1::new(&mut loader.load_context(info.clone(), 1));
        RenderPass3D2D { info, clear_values, scene_render0, scene_render1 }
    }
    pub fn scene_render0<'a>(&'a mut self, loader: &'a mut RenderLoader) -> LoadRef<'a, T0> { loader.load_ref_subpass(self.info.clone(), 0, &mut self.scene_render0) }
    pub fn scene_render1<'a>(&'a mut self, loader: &'a mut RenderLoader) -> LoadRef<'a, T1> { loader.load_ref_subpass(self.info.clone(), 1, &mut self.scene_render1) }
}

impl<T0, T1> RenderPass for RenderPass3D2D<T0, T1> where T0: SceneRenderer<RenderType=Geometry3D>, T1: SceneRenderer<RenderType=Geometry2D> {
    type Scene = (T0::Scene, T1::Scene);
    fn info(&self) -> RenderPassArc { self.info.clone() }
    fn set_dimensions(&mut self, dimensions: Size) {
        self.scene_render0.set_dimensions(dimensions);
        self.scene_render1.set_dimensions(dimensions);
    }
    fn render(&mut self, context: &mut RenderContext, scene: &mut Self::Scene) {
        self.scene_render0.pre_render(context, &mut scene.0);
        self.scene_render1.pre_render(context, &mut scene.1);
        context.builder.begin_render_pass(context.framebuffer.clone(), T0::contents(), self.clear_values.clone()).unwrap();
        self.scene_render0.render(context, &mut scene.0);
        context.builder.next_subpass(T1::contents()).unwrap();
        self.scene_render1.render(context, &mut scene.1);
        context.builder.end_render_pass().unwrap();
    }
}
