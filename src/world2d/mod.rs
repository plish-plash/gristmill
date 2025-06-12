use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use euclid::{vec2, Box2D};
use silica_gui::Rgba;
use silica_wgpu::{
    wgpu::{self, util::DeviceExt},
    Context, ResizableBuffer, Surface, SurfaceSize, Texture, TextureConfig, UvRect,
};

pub type Point = euclid::Point2D<f32, crate::WorldSpace>;
pub type Vector = euclid::Vector2D<f32, crate::WorldSpace>;
pub type Size = euclid::Size2D<f32, crate::WorldSpace>;
pub type Rect = euclid::Box2D<f32, crate::WorldSpace>;
pub type Transform = euclid::Transform2D<f32, crate::LocalSpace, crate::WorldSpace>;
pub type CameraTransform = euclid::Transform2D<f32, crate::WorldSpace, crate::ScreenSpace>;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Uniforms {
    view_matrix: CameraTransform,
    screen_resolution: [f32; 2],
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Quad {
    pub transform: Transform,
    pub uv: UvRect,
    pub color: Rgba,
}

impl Quad {
    pub fn rect_transform(rect: Rect) -> Transform {
        Transform::scale(rect.width(), rect.height()).then_translate(rect.min.to_vector())
    }
}

pub struct Pipeline2D {
    pipeline: wgpu::RenderPipeline,
    uniforms_buffer: wgpu::Buffer,
    uniforms_bind_group: wgpu::BindGroup,
}

impl Pipeline2D {
    pub fn new(
        context: &Context,
        surface_format: wgpu::TextureFormat,
        texture_config: &TextureConfig,
    ) -> Self {
        use wgpu::*;
        let device = &context.device;
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("world2d shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let uniforms_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("world2d uniforms bind group layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(size_of::<Uniforms>() as _),
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uniforms_layout, texture_config.bind_group_layout()],
            push_constant_ranges: &[],
        });

        let uniforms = Uniforms {
            view_matrix: CameraTransform::identity(),
            screen_resolution: [0.0; 2],
        };
        let uniforms_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("world2d uniforms"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let uniforms_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("world2d uniforms bind group"),
            layout: &uniforms_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniforms_buffer.as_entire_binding(),
            }],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("world2d pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[
                    VertexBufferLayout {
                        array_stride: std::mem::size_of::<Quad>() as u64,
                        step_mode: VertexStepMode::Instance,
                        attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x4, 3 => Float32x4],
                    },
                ],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::default(),
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                ..PrimitiveState::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Pipeline2D {
            pipeline,
            uniforms_buffer,
            uniforms_bind_group,
        }
    }

    pub fn set_camera(&mut self, context: &Context, camera: CameraTransform, size: SurfaceSize) {
        let uniforms = Uniforms {
            view_matrix: camera,
            screen_resolution: size.to_f32().to_array(),
        };
        context
            .queue
            .write_buffer(&self.uniforms_buffer, 0, bytemuck::bytes_of(&uniforms));
    }
    pub fn bind(&self, pass: &mut wgpu::RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniforms_bind_group, &[]);
    }
    pub fn bind_texture(&self, pass: &mut wgpu::RenderPass, texture: &Texture) {
        pass.set_bind_group(1, texture.bind_group(), &[]);
    }
    pub fn bind_buffer(&self, pass: &mut wgpu::RenderPass, buffer: &ResizableBuffer<Quad>) {
        pass.set_vertex_buffer(0, buffer.buffer().slice(..));
    }
    pub fn draw(&self, pass: &mut wgpu::RenderPass, range: Range<u32>) {
        pass.draw(0..4, range);
    }
}

struct DrawCall {
    buffer: Option<wgpu::Buffer>,
    texture: wgpu::BindGroup,
    range: Range<u32>,
}

pub struct DrawCallBuilder<'a> {
    pub context: &'a Context,
    buffer_data: &'a mut Vec<Quad>,
    draw_calls: &'a mut Vec<DrawCall>,
    current_texture: Option<wgpu::BindGroup>,
    last_index: usize,
}

impl DrawCallBuilder<'_> {
    fn flush(&mut self) {
        if let Some(current_texture) = self.current_texture.take() {
            if self.last_index < self.buffer_data.len() {
                self.draw_calls.push(DrawCall {
                    buffer: None,
                    texture: current_texture,
                    range: (self.last_index as u32)..(self.buffer_data.len() as u32),
                });
                self.last_index = self.buffer_data.len();
            }
        }
    }
    pub fn set_texture(&mut self, texture: &Texture) {
        let texture = texture.bind_group();
        if self.current_texture.as_ref() != Some(texture) {
            self.flush();
        }
        self.current_texture = Some(texture.clone());
    }
    pub fn draw(&mut self, quad: Quad) {
        self.buffer_data.push(quad);
    }
    pub fn draw_buffer(&mut self, texture: &Texture, buffer: &ResizableBuffer<Quad>) {
        self.flush();
        self.current_texture = None;
        let range = 0..(buffer.len() as u32);
        let buffer = buffer.buffer().clone();
        let texture = texture.bind_group().clone();
        self.draw_calls.push(DrawCall {
            buffer: Some(buffer),
            texture,
            range,
        })
    }
}
impl Drop for DrawCallBuilder<'_> {
    fn drop(&mut self) {
        self.flush();
    }
}

#[derive(Clone)]
pub struct Camera2D {
    pub viewport: Option<Box2D<u32, Surface>>,
    pub center: Point,
    pub bounds: Option<Rect>,
    pub scale: f32,
}

impl Camera2D {
    pub fn transform(&self, size: SurfaceSize) -> CameraTransform {
        let viewport_center = self
            .viewport
            .map(|viewport| viewport.center())
            .unwrap_or_else(|| (size / 2).to_vector().to_point())
            .to_f32();
        CameraTransform::translation(-self.center.x, -self.center.y)
            .then_scale(self.scale, self.scale)
            .then_translate(vec2(viewport_center.x, viewport_center.y))
    }
}
impl Default for Camera2D {
    fn default() -> Self {
        Camera2D {
            viewport: None,
            center: Point::zero(),
            bounds: None,
            scale: 1.0,
        }
    }
}

pub struct Renderer2D {
    pipeline: Pipeline2D,
    surface_size: SurfaceSize,
    buffer: ResizableBuffer<Quad>,
    buffer_data: Vec<Quad>,
    draw_calls: Vec<DrawCall>,
}

impl Renderer2D {
    pub fn new(
        context: &Context,
        surface_format: wgpu::TextureFormat,
        texture_config: &TextureConfig,
    ) -> Self {
        let pipeline = Pipeline2D::new(context, surface_format, texture_config);
        Renderer2D {
            pipeline,
            surface_size: SurfaceSize::zero(),
            buffer: ResizableBuffer::new(context),
            buffer_data: Vec::new(),
            draw_calls: Vec::new(),
        }
    }
    pub fn surface_resize(&mut self, size: SurfaceSize) {
        self.surface_size = size;
    }
    pub fn render(
        &mut self,
        context: &Context,
        pass: &mut wgpu::RenderPass,
        camera: Camera2D,
        f: impl FnOnce(&mut DrawCallBuilder),
    ) {
        self.pipeline.set_camera(
            context,
            camera.transform(self.surface_size),
            self.surface_size,
        );
        {
            let mut renderer = DrawCallBuilder {
                context,
                buffer_data: &mut self.buffer_data,
                draw_calls: &mut self.draw_calls,
                current_texture: None,
                last_index: 0,
            };
            f(&mut renderer);
        }
        self.buffer.set_data(context, &self.buffer_data);
        self.buffer_data.clear();

        self.pipeline.bind(pass);
        self.pipeline.bind_buffer(pass, &self.buffer);
        let mut viewport_set = false;
        let mut buffer_set = false;
        if let Some(viewport) = camera.viewport {
            let viewport = viewport.to_u32();
            pass.set_scissor_rect(
                viewport.min.x,
                viewport.min.y,
                viewport.width(),
                viewport.height(),
            );
            viewport_set = true;
        }
        for DrawCall {
            buffer,
            texture,
            range,
        } in self.draw_calls.drain(..)
        {
            if let Some(buffer) = buffer {
                pass.set_vertex_buffer(0, buffer.slice(..));
                buffer_set = true;
            } else if buffer_set {
                self.pipeline.bind_buffer(pass, &self.buffer);
                buffer_set = false;
            }
            pass.set_bind_group(1, &texture, &[]);
            self.pipeline.draw(pass, range);
        }
        if viewport_set {
            pass.set_scissor_rect(0, 0, self.surface_size.width, self.surface_size.height);
        }
    }
}
