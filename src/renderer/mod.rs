pub mod pass;
pub mod subpass;

// -------------------------------------------------------------------------------------------------

use std::sync::Arc;

use vulkano::command_buffer::DynamicState;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass};
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::image::view::ImageView;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::image::attachment::AttachmentImage;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain;
use vulkano::swapchain::{
    AcquireError, ColorSpace, FullscreenExclusive, PresentMode, SurfaceTransform, Swapchain,
    SwapchainCreationError, Surface
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use vulkano::format::Format;

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
use vulkano_win::VkSurfaceBuild;

use super::game::{Game, GameLoop};
use super::input::InputSystem;
use super::geometry2d::Size;

pub use pass::{RenderPass, RenderPassInfo};

// -------------------------------------------------------------------------------------------------

type FramebufferArc = Arc<dyn vulkano::framebuffer::FramebufferAbstract + Send + Sync>;
type RenderPassArc = Arc<dyn vulkano::framebuffer::RenderPassAbstract + Send + Sync>;
pub type PipelineArc = Arc<dyn vulkano::pipeline::GraphicsPipelineAbstract + Send + Sync>;

// -------------------------------------------------------------------------------------------------

pub struct RendererSetup(Renderer, Vec<Arc<SwapchainImage<Window>>>, Option<Box<dyn GpuFuture>>);

impl RendererSetup {
    pub fn device(&self) -> Arc<Device> { self.0.device() }
    pub fn graphics_queue(&self) -> Arc<Queue> { self.0.graphics_queue() }
    //pub fn transfer_queue(&self) -> Arc<Queue> { self.0.transfer_queue() }
    pub fn swapchain_format(&self) -> Format { self.0.swapchain.format() }

    pub fn subpass_setup(&mut self, render_pass: RenderPassArc, id: u32) -> SubpassSetup {
        SubpassSetup {
            queue: self.graphics_queue(),
            subpass: Subpass::from(render_pass, id).unwrap(),
            setup_future: &mut self.2,
        }
    }
}

pub struct RendererLoader<'a> {
    queue: Arc<Queue>,
    setup_future: &'a mut Option<Box<dyn GpuFuture>>,
}

impl<'a> RendererLoader<'a> {
    fn new(renderer: &Renderer, setup_future: &'a mut Option<Box<dyn GpuFuture>>) -> RendererLoader<'a> {
        RendererLoader { queue: renderer.graphics_queue(), setup_future }
    }
    pub fn subpass_setup(&mut self, render_pass: RenderPassArc, id: u32) -> SubpassSetup {
        SubpassSetup {
            queue: self.queue.clone(),
            subpass: Subpass::from(render_pass, id).unwrap(),
            setup_future: self.setup_future,
        }
    }
}

pub struct SubpassSetup<'a> {
    queue: Arc<Queue>,
    subpass: Subpass<RenderPassArc>,
    setup_future: &'a mut Option<Box<dyn GpuFuture>>,
}

impl<'a> SubpassSetup<'a> {
    pub fn device(&self) -> Arc<Device> { self.queue.device().clone() }
    pub fn queue(&self) -> Arc<Queue> { self.queue.clone() }
    pub fn subpass(&self) -> Subpass<RenderPassArc> {
        self.subpass.clone()
    }
    pub fn queue_join<F>(&mut self, future: F) where F: GpuFuture + 'static {
        if let Some(prev_future) = self.setup_future.take() {
            *self.setup_future = Some(Box::new(prev_future.join(future)));
        } else {
            *self.setup_future = Some(Box::new(future));
        };
    }
}

pub struct Renderer {
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,
    surface_dimensions: Size,
    dynamic_state: DynamicState,

    graphics_queue: Arc<Queue>,
    //transfer_queue: Arc<Queue>,
    swapchain: Arc<Swapchain<Window>>,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
}

impl Renderer {
    pub fn device(&self) -> Arc<Device> { self.device.clone() }
    pub fn graphics_queue(&self) -> Arc<Queue> { self.graphics_queue.clone() }
    //pub fn transfer_queue(&self) -> Arc<Queue> { self.transfer_queue.clone() }
    pub fn swapchain(&self) -> Arc<Swapchain<Window>> { self.swapchain.clone() }
    fn surface_dimensions(&self) -> Size { self.surface_dimensions }
}

impl Renderer {
    pub fn create_window() -> (RendererSetup, EventLoop<()>) {
        let required_extensions = vulkano_win::required_extensions();
        let instance = Instance::new(None, &required_extensions, None).unwrap();

        // For the sake of the example we are just going to use the first device, which should work
        // most of the time.
        let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

        // Some little debug infos.
        println!(
            "Using device: {} (type: {:?})",
            physical.name(),
            physical.ty()
        );

        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();

        // In a real-life application, we would probably use at least a graphics queue and a transfers
        // queue to handle data transfers in parallel. In this example we only use one queue.
        let queue_family = physical
            .queue_families()
            .find(|&q| {
                // We take the first queue that supports drawing to our window.
                q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
            })
            .unwrap();
        
        let device_ext = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        let (device, mut queues) = Device::new(
            physical,
            physical.supported_features(),
            &device_ext,
            [(queue_family, 0.5)].iter().cloned(),
        ).unwrap();

        // Since we can request multiple queues, the `queues` variable is in fact an iterator. In this
        // example we use only one queue, so we just retrieve the first and only element of the
        // iterator and throw it away.
        let queue = queues.next().unwrap();

        // Before we can draw on the surface, we have to create what is called a swapchain. Creating
        // a swapchain allocates the color buffers that will contain the image that will ultimately
        // be visible on the screen. These images are returned alongside with the swapchain.
        let surface_dimensions: [u32; 2] = surface.window().inner_size().into();
        let (swapchain, images) = {
            let caps = surface.capabilities(physical).unwrap();
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;

            Swapchain::new(
                device.clone(),
                surface.clone(),
                caps.min_image_count,
                format,
                surface_dimensions,
                1,
                ImageUsage::color_attachment(),
                &queue,
                SurfaceTransform::Identity,
                alpha,
                PresentMode::Fifo,
                FullscreenExclusive::Default,
                true,
                ColorSpace::SrgbNonLinear,
            ).unwrap()
        };

        let dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        };

        (RendererSetup(
            Renderer {
                device,
                surface,
                surface_dimensions: surface_dimensions.into(),
                dynamic_state,
                graphics_queue: queue.clone(),
                swapchain,
                framebuffers: Vec::new()
            },
            images,
            None,
        ), event_loop)
    }

    fn now_future(&self) -> Option<Box<dyn GpuFuture>> {
        Some(sync::now(self.device()).boxed())
    }

    fn recreate_swapchain(&mut self, render_pass: RenderPassArc) {
        let dimensions: [u32; 2] = self.surface.window().inner_size().into();
        self.surface_dimensions = dimensions.into();
        let (new_swapchain, new_images) =
            match self.swapchain.recreate_with_dimensions(dimensions) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };

        self.swapchain = new_swapchain;
        self.framebuffers = self.window_size_dependent_setup(&new_images, render_pass);
    }

    fn window_size_dependent_setup(&mut self, images: &[Arc<SwapchainImage<Window>>], render_pass: RenderPassArc) -> Vec<FramebufferArc> {
        let dimensions = images[0].dimensions();
    
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        };
        self.dynamic_state.viewports = Some(vec![viewport]);
    
        let depth_buffer = AttachmentImage::transient(self.device(), dimensions, Format::D16Unorm).unwrap();
        let depth_view = ImageView::new(depth_buffer).unwrap();
        images
            .iter()
            .map(|image| {
                let view = ImageView::new(image.clone()).unwrap();
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(view).unwrap()
                        .add(depth_view.clone()).unwrap()
                        .build().unwrap(),
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            })
            .collect::<Vec<_>>()
    }
}

// -------------------------------------------------------------------------------------------------

pub(crate) struct RenderLoop<G> where G: Game {
    renderer: Renderer,
    render_pass: RenderPassInfo<G::RenderPass>,
    game: G,
    scene: <G::RenderPass as RenderPass>::Scene,
    input_system: InputSystem,

    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl<G> RenderLoop<G> where G: Game {
    pub fn new(renderer: RendererSetup, mut render_pass: RenderPassInfo<G::RenderPass>, mut game: G, mut scene: <G::RenderPass as RenderPass>::Scene, input_system: InputSystem) -> RenderLoop<G> {
        let (mut renderer, images, setup_future) = (renderer.0, renderer.1, renderer.2);
        let framebuffers = renderer.window_size_dependent_setup(&images, render_pass.raw_info());
        renderer.framebuffers = framebuffers;
        let dimensions = renderer.surface_dimensions();
        render_pass.set_dimensions(dimensions);
        game.resize(&mut scene, dimensions);

        let previous_frame_end = setup_future.or_else(|| renderer.now_future());
        RenderLoop {
            renderer,
            render_pass,
            game,
            scene,
            input_system,
            recreate_swapchain: false,
            previous_frame_end,
        }
    }
}

impl<G> GameLoop for RenderLoop<G> where G: Game + 'static {
    fn window(&self) -> &Window { self.renderer.surface.window() }
    fn update(&mut self, delta: f64) -> bool {
        self.input_system.start_frame();
        let continue_loop = self.game.update(&mut self.scene, self.renderer.surface.window(), &mut self.input_system, delta);
        self.input_system.end_frame();
        continue_loop
    }
    fn event(&mut self, event: Event<()>) {
        if let Event::WindowEvent { event, .. } = &event {
            match event {
                WindowEvent::Resized(_) => {
                    self.recreate_swapchain = true;
                }
                _ => ()
            }
        }
        self.input_system.input_event(event);
    }
    fn render(&mut self) {
        // Calling this function polls various fences in order to determine what the GPU has
        // already processed, and frees the resources that are no longer needed.
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swapchain {
            self.renderer.recreate_swapchain(self.render_pass.raw_info());
            let dimensions = self.renderer.surface_dimensions();
            self.render_pass.set_dimensions(dimensions);
            self.game.resize(&mut self.scene, dimensions);
            self.recreate_swapchain = false;
        }

        // Give the game an opportunity to change the render pass, including making new subpasses and pipelines.
        let mut loader = RendererLoader::new(&self.renderer, &mut self.previous_frame_end);
        self.game.update_renderer(&mut self.scene, &mut self.render_pass, &mut loader);

        // Before we can draw on the output, we have to *acquire* an image from the swapchain. If
        // no image is available (which happens if you submit draw commands too quickly), then the
        // function will block.
        let (image_num, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.renderer.swapchain(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

        // acquire_next_image can be successful, but suboptimal. This means that the swapchain image
        // will still work, but it may not display correctly. With some drivers this can be when
        // the window resizes, but it may not cause the swapchain to become out of date.
        if suboptimal {
            self.recreate_swapchain = true;
        }

        let queue = self.renderer.graphics_queue();
        let command_buffer = self.render_pass.build_command_buffer(
            queue.family(),
            self.renderer.framebuffers[image_num].clone(),
            &self.renderer.dynamic_state,
            &mut self.scene,
        );

        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(queue, self.renderer.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = self.renderer.now_future();
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self.previous_frame_end = self.renderer.now_future();
            }
        }
    }
}
