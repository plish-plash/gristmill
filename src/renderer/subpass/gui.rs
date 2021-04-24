use std::sync::Arc;

use vulkano::buffer::BufferAccess;
use vulkano::command_buffer::{DynamicState, AutoCommandBufferBuilder, SubpassContents};
use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::sampler::Filter;
use vulkano::instance::QueueFamily;

use rusttype::{PositionedGlyph, Scale, point};

use crate::asset::image::Image;
use crate::color::{Color, encode_color};
use crate::renderer::{PipelineArc, SubpassSetup, subpass::{self, Pipeline, RenderSubpass}, pipeline::{text, textured_rect}};
use crate::gui::{Gui, font::{Font, fonts}};
use crate::geometry2d::{Rect, Size};

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, tex_position);

impl textured_rect::Vertex for Vertex {
    fn new(x: f32, y: f32) -> Vertex {
        Vertex {
            position: [x, y],
            tex_position: [x, y],
        }
    }
}

impl text::Vertex for Vertex {
    fn new(position: [i32; 2], tex_position: [f32; 2]) -> Vertex {
        Vertex {
            position: [position[0] as f32, position[1] as f32],
            tex_position,
        }
    }
}

mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: "
            #version 450

            layout(push_constant) uniform PushConstants {
                vec2 screen_size;
                vec2 position;
                vec2 size;
                vec4 color;
            } constants;

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec2 tex_position;
            layout(location = 0) out vec2 v_tex_position;
            layout(location = 1) out vec4 v_color;

            void main() {
                vec2 normalized_position = (constants.position + (position * constants.size)) / constants.screen_size;
                gl_Position = vec4((normalized_position - 0.5) * 2.0, 0.0, 1.0);
                v_tex_position = tex_position;
                v_color = constants.color;
            }
        "
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) in vec2 v_tex_position;
            layout(location = 1) in vec4 v_color;
            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 0) uniform sampler2D tex;

            void main() {
                f_color = v_color * texture(tex, v_tex_position)[0];
            }
        "
    }
}

pub struct GuiPipeline {
    pipeline: PipelineArc,
    textured_rect: textured_rect::PipelineData<Vertex>,
    text: text::PipelineData<Vertex>,

    white_1x1: textured_rect::Texture,
}

impl GuiPipeline {
    fn new(subpass_setup: &mut SubpassSetup) -> GuiPipeline {
        let vs = vs::Shader::load(subpass_setup.device()).unwrap();
        let fs = fs::Shader::load(subpass_setup.device()).unwrap();

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(subpass_setup.subpass())
                .build(subpass_setup.device())
                .unwrap()
        );

        let mut textured_rect = textured_rect::PipelineData::new(pipeline.clone(), subpass_setup);
        let white_1x1 = textured_rect.upload_texture(subpass_setup, &Image::new_1x1_white(), Filter::Nearest);

        let text = text::PipelineData::new(pipeline.clone(), subpass_setup);

        GuiPipeline { pipeline, textured_rect, text, white_1x1 }
    }
}

impl Pipeline for GuiPipeline {
    fn raw_pipeline(&self) -> PipelineArc { self.pipeline.clone() }
}

enum DrawVertexBuffer {
    Square,
    Text(Arc<text::Handle>),
}

pub struct DrawCommand {
    vertex_buffer: DrawVertexBuffer,
    descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
    position: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
}

// Note: it would be nice to build an instance buffer instead of drawing one-at-a-time with push constants.
// tricky part is descriptor_sets, since those do need separate draw commands, and draw order is important.
impl DrawCommand {
    fn add_to_builder(self, pipeline: &GuiPipeline, builder: &mut AutoCommandBufferBuilder, dynamic_state: &DynamicState, screen_dimensions: [f32; 2]) {
        let vertex_buffer: Arc<dyn BufferAccess + Send + Sync> = match self.vertex_buffer {
            DrawVertexBuffer::Square => pipeline.textured_rect.square_vertex_buffer(),
            DrawVertexBuffer::Text(handle) => pipeline.text.get_section_vertex_buffer(&handle),
        };
        let push_constants = vs::ty::PushConstants {
            screen_size: screen_dimensions,
            position: self.position,
            size: self.size,
            color: self.color,
            _dummy0: [0; 8],
        };
        builder.draw(
            pipeline.raw_pipeline(),
            dynamic_state,
            vec![vertex_buffer],
            self.descriptor_set,
            push_constants,
            vec![],
        ).unwrap();
    }
}

#[derive(Clone)]
pub enum Drawable {
    TexturedRect(textured_rect::Texture),
    Text(Arc<text::Handle>),
}

#[derive(Copy, Clone, Debug)]
pub struct TextMetrics {
    width: f32,
    v_metrics: rusttype::VMetrics,
}

impl TextMetrics {
    fn new(text_glyphs: &Vec<PositionedGlyph<'static>>) -> TextMetrics {
        let last_glyph = text_glyphs.last().unwrap();
        let width = last_glyph.position().x + last_glyph.unpositioned().h_metrics().advance_width;
        let v_metrics = last_glyph.font().v_metrics(last_glyph.scale());
        TextMetrics { width, v_metrics }
    }

    pub fn width(&self) -> f32 { self.width }
    pub fn ascent(&self) -> f32 { self.v_metrics.ascent }
    pub fn height(&self) -> f32 { self.v_metrics.ascent - self.v_metrics.descent }
}

pub struct DrawContext<'a> {
    pipeline: &'a mut GuiPipeline,
    pending_draw_commands: &'a mut Vec<DrawCommand>,
    text_changed: bool,
}

impl<'a> DrawContext<'a> {
    pub fn new_color_rect_drawable(&mut self) -> Drawable {
        Drawable::TexturedRect(self.pipeline.white_1x1.clone())
    }
    pub fn new_text_drawable(&mut self, font: Font, size: f32, text: &str) -> (Drawable, TextMetrics) {
        if text.is_empty() { panic!("TextDrawable requires a non-empty string"); }
        let font_asset = fonts().get(font);
        let glyphs: Vec<PositionedGlyph> = font_asset.layout(text, Scale::uniform(size), point(0., 0.)).collect();
        let metrics = TextMetrics::new(&glyphs);
        let handle = self.pipeline.text.add_section(glyphs);
        self.text_changed = true;
        (Drawable::Text(handle), metrics)
    }

    pub fn draw(&mut self, drawable: &Drawable, rect: Rect, color: Color) {
        let (vertex_buffer, descriptor_set, size) = match drawable {
            Drawable::TexturedRect(texture) => (DrawVertexBuffer::Square, texture.descriptor_set(), rect.size),
            Drawable::Text(handle) => (DrawVertexBuffer::Text(handle.clone()), self.pipeline.text.descriptor_set(), Size::new(1, 1)),
        };
        self.pending_draw_commands.push(DrawCommand {
            vertex_buffer,
            descriptor_set,
            position: rect.position.into(),
            size: size.into(),
            color: encode_color(color),
        });
    }

    fn update_cache(&mut self, builder: &mut AutoCommandBufferBuilder) {
        if self.text_changed {
            self.pipeline.text.update_cache(builder);
            self.text_changed = false;
        }
    }
}

pub struct GuiSubpass {
    pipeline: GuiPipeline,
    screen_dimensions: Size,
    pending_draw_commands: Vec<DrawCommand>,
}

impl GuiSubpass {
    fn make_context<'a>(&'a mut self) -> DrawContext<'a> {
        self.pending_draw_commands.clear();
        DrawContext { pipeline: &mut self.pipeline, pending_draw_commands: &mut self.pending_draw_commands, text_changed: false }
    }
}

impl RenderSubpass for GuiSubpass {
    type SubpassCategory = subpass::Gui;
    type Scene = Gui;
    fn contents() -> SubpassContents { SubpassContents::Inline }
    fn new(subpass_setup: &mut SubpassSetup) -> Self {
        let pipeline = GuiPipeline::new(subpass_setup);
        GuiSubpass {
            pipeline,
            screen_dimensions: Size::zero(),
            pending_draw_commands: Vec::new(),
        }
    }
    fn set_dimensions(&mut self, dimensions: Size) {
        self.screen_dimensions = dimensions;
    }

    fn pre_render(&mut self, scene: &mut Gui, builder: &mut AutoCommandBufferBuilder, _queue_family: QueueFamily) {
        scene.layout_if_needed(self.screen_dimensions);
        let mut context = self.make_context();
        scene.draw(&mut context);
        context.update_cache(builder);
    }

    fn render(&mut self, _scene: &Gui, builder: &mut AutoCommandBufferBuilder, dynamic_state: &DynamicState) {
        let screen_dimensions = self.screen_dimensions.into();
        for draw_command in self.pending_draw_commands.drain(..) {
            draw_command.add_to_builder(&self.pipeline, builder, dynamic_state, screen_dimensions);
        }
    }
}
