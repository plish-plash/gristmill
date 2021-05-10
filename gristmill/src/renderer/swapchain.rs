use std::sync::Arc;

use vulkano::command_buffer::DynamicState;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract};
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::image::view::ImageView;
use vulkano::image::attachment::AttachmentImage;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::{
    ColorSpace, FullscreenExclusive, PresentMode, SurfaceTransform,
    SwapchainCreationError, Surface
};
use vulkano::format::Format;

use winit::window::Window;

use super::{RenderPassArc, FramebufferArc, RenderLoader};
use crate::geometry2d::Size;

// -------------------------------------------------------------------------------------------------

type SwapchainArc = Arc<vulkano::swapchain::Swapchain<Window>>;

pub struct Swapchain {
    pub(crate) dynamic_state: DynamicState,
    swapchain: SwapchainArc,
    dimensions: Size,
    render_pass: RenderPassArc,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
}

impl Swapchain {
    pub fn create(loader: &mut RenderLoader, render_pass: RenderPassArc) -> Self {
        let dimensions: [u32; 2] = loader.surface.window().inner_size().into();
        let (swapchain, images) = vulkano::swapchain::Swapchain::new(
            loader.device.clone(),
            loader.surface.clone(),
            loader.swapchain_info.image_count,
            loader.swapchain_info.format,
            dimensions,
            1,
            ImageUsage::color_attachment(),
            &loader.graphics_queue,
            SurfaceTransform::Identity,
            loader.swapchain_info.composite_alpha,
            PresentMode::Fifo,
            FullscreenExclusive::Default,
            true,
            ColorSpace::SrgbNonLinear,
        ).unwrap();

        let dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        };

        let mut swapchain = Swapchain {
            dynamic_state, swapchain, dimensions: Size::zero(), render_pass, framebuffers: Vec::new()
        };
        swapchain.window_size_dependent_setup(&images);
        swapchain
    }
    pub fn recreate(&mut self, surface: &Arc<Surface<Window>>) -> bool {
        let dimensions: [u32; 2] = surface.window().inner_size().into();
        let (new_swapchain, new_images) =
            match self.swapchain.recreate_with_dimensions(dimensions) {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => return false,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };

        self.swapchain = new_swapchain;
        self.window_size_dependent_setup(&new_images);
        true
    }

    pub fn swapchain(&self) -> SwapchainArc {
        self.swapchain.clone()
    }
    pub fn dimensions(&self) -> Size {
        self.dimensions
    }
    pub fn get_framebuffer(&self, image_num: usize) -> FramebufferArc {
        self.framebuffers[image_num].clone()
    }

    fn window_size_dependent_setup(&mut self, images: &[Arc<SwapchainImage<Window>>]) {
        let dimensions = images[0].dimensions();
        self.dimensions = dimensions.into();
    
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        };
        self.dynamic_state.viewports = Some(vec![viewport]);
    
        let depth_buffer = AttachmentImage::transient(self.render_pass.device().clone(), dimensions, Format::D16Unorm).unwrap();
        let depth_view = ImageView::new(depth_buffer).unwrap();
        self.framebuffers = images
            .iter()
            .map(|image| {
                let view = ImageView::new(image.clone()).unwrap();
                Arc::new(
                    Framebuffer::start(self.render_pass.clone())
                        .add(view).unwrap()
                        .add(depth_view.clone()).unwrap()
                        .build().unwrap(),
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            })
            .collect::<Vec<_>>();
    }
}
