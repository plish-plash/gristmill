pub mod loader;
pub mod pass;
pub mod scene;
mod swapchain;

// -------------------------------------------------------------------------------------------------

use std::sync::Arc;
use std::collections::HashMap;

use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::descriptor::descriptor_set::DescriptorSetsCollection;
use vulkano::framebuffer::Subpass;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::{GraphicsPipelineAbstract, vertex::VertexSource};
use vulkano::swapchain::acquire_next_image;
use vulkano::swapchain::{
    AcquireError, CompositeAlpha, Surface
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

use crate::asset::resource::AssetList;
use super::game::{Game, GameLoop};
use super::input::InputSystem;

use swapchain::Swapchain;
use loader::RenderAssetLoader;

// -------------------------------------------------------------------------------------------------

type FramebufferArc = Arc<dyn vulkano::framebuffer::FramebufferAbstract + Send + Sync>;
pub type RenderPassArc = Arc<dyn vulkano::framebuffer::RenderPassAbstract + Send + Sync>;
pub type PipelineArc = Arc<dyn vulkano::pipeline::GraphicsPipelineAbstract + Send + Sync>;

pub use pass::RenderPass;
pub use scene::SceneRenderer;

// -------------------------------------------------------------------------------------------------

pub trait RenderAsset: Clone {
    fn type_name() -> &'static str;
    fn none() -> Self;
}

pub struct RenderAssetList<T: RenderAsset> {
    name: String,
    map: HashMap<String, T>,
}

impl<T: RenderAsset> RenderAssetList<T> {
    fn new(name: String) -> RenderAssetList<T> { RenderAssetList { name, map: HashMap::new() } }
    pub fn name(&self) -> &str { &self.name }
    pub fn get(&self, key: &str) -> T {
        self.map.get(key).map(|a| a.clone()).unwrap_or_else(|| {
            log::warn!("{} {} not found in {} list", T::type_name(), key, self.name);
            T::none()
        })
    }
    pub fn insert(&mut self, key: String, value: T) {
        self.map.insert(key, value);
    }
}

// -------------------------------------------------------------------------------------------------

pub struct LoadContext<'a> {
    queue: Arc<Queue>,
    subpass: Subpass<RenderPassArc>,
    setup_future: &'a mut Option<Box<dyn GpuFuture>>,
}

impl<'a> LoadContext<'a> {
    pub fn device(&self) -> Arc<Device> { self.queue.device().clone() }
    pub fn queue(&self) -> Arc<Queue> { self.queue.clone() }
    pub fn subpass(&self) -> Subpass<RenderPassArc> {
        self.subpass.clone()
    }
    pub fn load_future<F>(&mut self, future: F) where F: GpuFuture + 'static {
        if let Some(prev_future) = self.setup_future.take() {
            *self.setup_future = Some(Box::new(prev_future.join(future)));
        } else {
            *self.setup_future = Some(Box::new(future));
        };
    }
}

pub struct LoadRef<'a, T> {
    pub context: LoadContext<'a>,
    pub inner: &'a mut T,
}

impl<'a, T> LoadRef<'a, T> where T: RenderAssetLoader {
    pub fn load_asset(&mut self, asset_type: &str, asset_path: &str) -> Option<T::RenderAsset> {
        self.inner.load(&mut self.context, asset_type, asset_path)
    }
    pub fn load_assets(&mut self, asset_list: &AssetList) -> RenderAssetList<T::RenderAsset> {
        let mut list = RenderAssetList::new(asset_list.name().to_owned());
        if asset_list.loader().is_empty() {
            return list;
        }
        if asset_list.loader() != T::name() {
            log::error!("Invalid loader for {} list (expected \"{}\", got \"{}\")", asset_list.name(), T::name(), asset_list.loader());
            return list;
        }
        for item in asset_list.iter() {
            if let Some(asset) = self.load_asset(&item.asset_type, &item.asset_path) {
                list.insert(item.name.to_owned(), asset);
            }
        }
        log::debug!("Renderer loaded {} list", list.name());
        list
    }
}

pub struct RenderContext<'a> {
    framebuffer: FramebufferArc,
    builder: &'a mut AutoCommandBufferBuilder,
    dynamic_state: &'a DynamicState,
}

impl<'a> RenderContext<'a> {
    pub fn command_buffer_builder(&mut self) -> &mut AutoCommandBufferBuilder { &mut self.builder }
    pub fn draw<V, Gp, S, Pc>(
        &mut self,
        pipeline: Gp,
        vertex_buffer: V,
        sets: S,
        constants: Pc,
    )
    where
        Gp: GraphicsPipelineAbstract + VertexSource<V> + Send + Sync + 'static + Clone,
        S: DescriptorSetsCollection,
    {
        self.builder.draw(pipeline, self.dynamic_state, vertex_buffer, sets, constants, Vec::new()).unwrap();
    }
}

struct SwapchainInfo {
    image_count: u32,
    format: Format,
    composite_alpha: CompositeAlpha,
}

pub struct RenderLoader {
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,
    swapchain_info: SwapchainInfo,
    graphics_queue: Arc<Queue>,
    //transfer_queue: Arc<Queue>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl RenderLoader {
    pub fn device(&self) -> Arc<Device> { self.device.clone() }
    pub fn graphics_queue(&self) -> Arc<Queue> { self.graphics_queue.clone() }
    //pub fn transfer_queue(&self) -> Arc<Queue> { self.transfer_queue.clone() }
    pub fn swapchain_format(&self) -> Format { self.swapchain_info.format }

    fn load_context(&mut self, render_pass: RenderPassArc, subpass_id: u32) -> LoadContext {
        LoadContext {
            queue: self.graphics_queue(),
            subpass: Subpass::from(render_pass, subpass_id).unwrap(),
            setup_future: &mut self.previous_frame_end,
        }
    }
    fn load_ref_subpass<'a, T>(&'a mut self, render_pass: RenderPassArc, subpass_id: u32, subpass: &'a mut T) -> LoadRef<'a, T> {
        LoadRef {
            context: self.load_context(render_pass, subpass_id),
            inner: subpass,
        }
    }

    fn now_future(&self) -> Option<Box<dyn GpuFuture>> {
        Some(sync::now(self.device()).boxed())
    }

    pub(crate) fn create_window() -> (RenderLoader, EventLoop<()>) {
        let required_extensions = vulkano_win::required_extensions();
        let instance = Instance::new(None, &required_extensions, None).unwrap();

        // For the sake of the example we are just going to use the first device, which should work
        // most of the time.
        let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
        log::debug!(
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

        let caps = surface.capabilities(physical).unwrap();
        let swapchain_info = SwapchainInfo {
            image_count: caps.min_image_count,
            composite_alpha: caps.supported_composite_alpha.iter().next().unwrap(),
            format: caps.supported_formats[0].0,
        };

        (RenderLoader {
            device,
            surface,
            swapchain_info,
            graphics_queue: queue,
            previous_frame_end: None,
        }, event_loop)
    }
}

// -------------------------------------------------------------------------------------------------

pub(crate) struct RenderLoop<G> where G: Game {
    loader: RenderLoader,
    swapchain: Swapchain,
    recreate_swapchain: bool,
    game: G,
    render_pass: G::RenderPass,
    input_system: InputSystem,
}

impl<G> RenderLoop<G> where G: Game {
    pub fn new(mut loader: RenderLoader, mut game: G, mut render_pass: G::RenderPass, input_system: InputSystem) -> RenderLoop<G> {
        let swapchain = Swapchain::create(&mut loader, render_pass.info());
        game.resize(swapchain.dimensions());
        render_pass.set_dimensions(swapchain.dimensions());
        if loader.previous_frame_end.is_none() {
            loader.previous_frame_end = loader.now_future();
        }
        RenderLoop {
            loader,
            swapchain,
            recreate_swapchain: false,
            game,
            render_pass,
            input_system,
        }
    }
}

impl<G> GameLoop for RenderLoop<G> where G: Game + 'static {
    fn window(&self) -> &Window { self.loader.surface.window() }
    fn update(&mut self, delta: f64) -> bool {
        self.input_system.start_frame();
        let continue_loop = self.game.update(self.loader.surface.window(), &mut self.input_system, delta);
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
        self.loader.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swapchain {
            if self.swapchain.recreate(&self.loader.surface) {
                self.game.resize(self.swapchain.dimensions());
                self.render_pass.set_dimensions(self.swapchain.dimensions());
                self.recreate_swapchain = false;
            }
            else {
                // Got SwapchainCreationError::UnsupportedDimensions.
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                return;
            }
        }

        // Before we can draw on the output, we have to *acquire* an image from the swapchain. If
        // no image is available (which happens if you submit draw commands too quickly), then the
        // function will block.
        let (image_num, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.swapchain(), None) {
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

        let queue = self.loader.graphics_queue();
        let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(self.loader.device(), queue.family()).unwrap();
        {
            let framebuffer = self.swapchain.get_framebuffer(image_num);
            let mut render_context = RenderContext { framebuffer, builder: &mut builder, dynamic_state: &self.swapchain.dynamic_state };
            self.game.render(&mut self.loader, &mut render_context, &mut self.render_pass);
        }
        let command_buffer = builder.build().unwrap();

        let future = self.loader.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(queue, self.swapchain.swapchain(), image_num)
            .then_signal_fence_and_flush();

        self.loader.previous_frame_end = match future {
            Ok(future) => {
                Some(future.boxed())
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.loader.now_future()
            }
            Err(e) => {
                log::warn!("Failed to flush future: {:?}", e);
                self.loader.now_future()
            }
        };
    }
}
