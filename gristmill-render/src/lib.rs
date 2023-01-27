mod texture;
pub mod texture_rect;

use gristmill_core::{geom2d::Rect, math::Vec2, Color};
use std::sync::Arc;
use vulkano::{
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassContents,
    },
    command_buffer::{PrimaryAutoCommandBuffer, PrimaryCommandBufferAbstract},
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::Queue,
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
    },
    format::{ClearValue, Format},
    image::{view::ImageView, AttachmentImage, ImageAccess, ImageUsage, SwapchainImage},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
    pipeline::graphics::viewport::Viewport,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::Surface,
    swapchain::{
        acquire_next_image, AcquireError, Swapchain, SwapchainCreateInfo, SwapchainCreationError,
        SwapchainPresentInfo,
    },
    sync::{self, FlushError, GpuFuture},
    VulkanLibrary,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub use texture::*;

pub trait Renderable {
    fn pre_render(&mut self, context: &mut RenderContext);
    fn render(&mut self, context: &mut RenderContext);
}

/// This method is called once during initialization, then again whenever the window is resized
fn window_size_dependent_setup(
    memory_allocator: &StandardMemoryAllocator,
    images: &[Arc<SwapchainImage>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

    let depth_buffer = ImageView::new_default(
        AttachmentImage::transient(memory_allocator, dimensions, Format::D16_UNORM).unwrap(),
    )
    .unwrap();

    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view, depth_buffer.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}

pub struct RenderContext {
    surface: Arc<Surface>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: StandardDescriptorSetAllocator,
    command_buffer_allocator: StandardCommandBufferAllocator,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,

    swapchain: Arc<Swapchain>,
    framebuffers: Vec<Arc<Framebuffer>>,
    clear_color: Color,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,

    current_builder: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
    current_framebuffer_index: usize,
    recently_resized: bool,
}

impl RenderContext {
    pub fn create_window(event_loop: &EventLoop<()>) -> Self {
        let library = VulkanLibrary::new().unwrap();
        let required_extensions = vulkano_win::required_extensions(&library);
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                enumerate_portability: true,
                ..Default::default()
            },
        )
        .unwrap();

        let surface = WindowBuilder::new()
            .build_vk_surface(event_loop, instance.clone())
            .unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.graphics
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| {
                // Assign a lower score to device types that are likely to be faster/better.
                match p.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
                    _ => 5,
                }
            })
            .expect("No suitable physical device found");

        log::debug!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();
        let queue = queues.next().unwrap();

        let (swapchain, images) = {
            let surface_capabilities = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();
            let image_format = Some(
                device
                    .physical_device()
                    .surface_formats(&surface, Default::default())
                    .unwrap()[0]
                    .0,
            );
            let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count,
                    image_format,
                    image_extent: window.inner_size().into(),
                    image_usage: ImageUsage {
                        color_attachment: true,
                        ..ImageUsage::empty()
                    },
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .iter()
                        .next()
                        .unwrap(),
                    ..Default::default()
                },
            )
            .unwrap()
        };

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16_UNORM,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        )
        .unwrap();

        let mut viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let framebuffers = window_size_dependent_setup(
            &memory_allocator,
            &images,
            render_pass.clone(),
            &mut viewport,
        );

        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(device.clone(), Default::default());
        let uploads = AutoCommandBufferBuilder::primary(
            &command_buffer_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        RenderContext {
            surface,
            device: device.clone(),
            queue,
            memory_allocator,
            descriptor_set_allocator: StandardDescriptorSetAllocator::new(device),
            command_buffer_allocator,
            render_pass,
            viewport,
            swapchain,
            framebuffers,
            clear_color: Color::WHITE,
            recreate_swapchain: false,
            previous_frame_end: None,
            current_builder: Some(uploads),
            current_framebuffer_index: 0,
            recently_resized: false,
        }
    }
    pub fn window(&self) -> &Window {
        self.surface
            .object()
            .unwrap()
            .downcast_ref::<Window>()
            .unwrap()
    }
    pub fn on_resize(&mut self) {
        self.recreate_swapchain = true;
        self.recently_resized = true;
    }
    pub fn finish_setup(&mut self) {
        let uploads = self.current_builder.take().unwrap();
        self.previous_frame_end = Some(
            uploads
                .build()
                .unwrap()
                .execute(self.queue.clone())
                .unwrap()
                .boxed(),
        );
    }

    fn begin_render_pass(&mut self) {
        self.current_builder
            .as_mut()
            .expect("not rendering")
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some(ClearValue::Float(self.clear_color.into())),
                        Some(ClearValue::Depth(1.0)),
                    ],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[self.current_framebuffer_index].clone(),
                    )
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .set_viewport(0, [self.viewport.clone()]);
    }
    fn end_render_pass(&mut self) {
        self.builder().end_render_pass().unwrap();
    }
    pub fn render_game<R: Renderable>(&mut self, game: &mut R) {
        if self.current_builder.is_some() {
            panic!("Do not call render_game here!");
        }

        // Do not draw frame when screen dimensions are zero.
        let dimensions = self.window().inner_size();
        if dimensions.width == 0 || dimensions.height == 0 {
            return;
        }

        // Clean up GPU resources that are no longer needed.
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        // Recreate the swapchain if desired.
        if self.recreate_swapchain {
            let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
                image_extent: dimensions.into(),
                ..self.swapchain.create_info()
            }) {
                Ok(r) => r,
                Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };

            self.swapchain = new_swapchain;
            self.framebuffers = window_size_dependent_setup(
                &self.memory_allocator,
                &new_images,
                self.render_pass.clone(),
                &mut self.viewport,
            );
            self.recreate_swapchain = false;
        }

        // Acquire an image for rendering.
        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };
        if suboptimal {
            self.recreate_swapchain = true;
        }

        self.current_builder = Some(
            AutoCommandBufferBuilder::primary(
                &self.command_buffer_allocator,
                self.queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap(),
        );
        self.current_framebuffer_index = image_index as usize;
        game.pre_render(self);
        self.begin_render_pass();
        game.render(self);
        self.end_render_pass();
        let command_buffer = self.current_builder.take().unwrap().build().unwrap();
        self.recently_resized = false;

        // Block until the previous frame is finished rendering.
        drop(self.previous_frame_end.take());

        let future = acquire_future
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
            Err(e) => {
                panic!("Failed to flush future: {:?}", e);
            }
        }
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }
    pub fn queue(&self) -> &Arc<Queue> {
        &self.queue
    }
    pub fn render_pass(&self) -> Subpass {
        Subpass::from(self.render_pass.clone(), 0).unwrap()
    }
    pub fn allocator(&self) -> &Arc<StandardMemoryAllocator> {
        &self.memory_allocator
    }
    pub fn descriptor_set_allocator(&self) -> &StandardDescriptorSetAllocator {
        &self.descriptor_set_allocator
    }
    pub fn was_resized(&self) -> bool {
        self.recently_resized
    }
    pub fn viewport(&self) -> Rect {
        Rect::new(
            Vec2::from(self.viewport.origin),
            Vec2::from(self.viewport.dimensions),
        )
    }
    pub fn builder(&mut self) -> &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> {
        self.current_builder.as_mut().expect("not rendering")
    }

    pub fn clear_color(&self) -> Color {
        self.clear_color
    }
    pub fn set_clear_color(&mut self, clear_color: Color) {
        self.clear_color = clear_color;
    }
}
