use std::sync::Arc;

use vulkano::command_buffer::{DynamicState, AutoCommandBufferBuilder, SubpassContents};
use vulkano::sampler::Filter;
use vulkano::instance::QueueFamily;

use rusttype::{PositionedGlyph, Scale, point};

use crate::asset::image::Image;
use crate::color::{Color, encode_color};
use crate::renderer::{SubpassSetup, subpass::{self, RenderSubpass}, pipeline::{text::{TextHandle, TextPipeline}, texture_rect::TextureRectPipeline}};
use crate::gui::{Gui, font::{Font, fonts}};
use crate::geometry2d::{Rect, Size};

type TextureRectConstants = crate::renderer::pipeline::texture_rect::PushConstants;
type TextConstants = crate::renderer::pipeline::text::PushConstants;

pub use crate::renderer::pipeline::texture_rect::{Texture, NineSliceTexture};

pub struct DrawCommand {
    drawable: Drawable,
    rect: Rect,
    color: [f32; 4],
}

impl DrawCommand {
    fn texture_rect_constants(&self, screen_dimensions: [f32; 2]) -> TextureRectConstants {
        TextureRectConstants {
            screen_size: screen_dimensions,
            position: self.rect.position.into(),
            size: self.rect.size.into(),
            color: self.color,
            _dummy0: [0; 8],
        }
    }
    fn text_constants(&self, screen_dimensions: [f32; 2]) -> TextConstants {
        TextConstants {
            screen_size: screen_dimensions,
            position: self.rect.position.into(),
            color: self.color,
        }
    }
}

#[derive(Clone)]
pub enum Drawable {
    TextureRect(Texture),
    TextureNineSlice(NineSliceTexture),
    Text(Arc<TextHandle>),
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
    subpass: &'a mut GuiSubpass,
    text_changed: bool,
}

impl<'a> DrawContext<'a> {
    pub fn new_color_rect_drawable(&mut self) -> Drawable {
        Drawable::TextureRect(self.subpass.white_1x1.clone())
    }
    pub fn new_texture_rect_drawable(&mut self, texture: Texture) -> Drawable {
        Drawable::TextureRect(texture)
    }
    pub fn new_texture_nine_slice_drawable(&mut self, texture: NineSliceTexture) -> Drawable {
        Drawable::TextureNineSlice(texture)
    }
    pub fn new_text_drawable(&mut self, font: Font, size: f32, text: &str) -> (Drawable, TextMetrics) {
        if text.is_empty() { panic!("TextDrawable requires a non-empty string"); }
        let font_asset = fonts().get(font);
        let glyphs: Vec<PositionedGlyph> = font_asset.layout(text, Scale::uniform(size), point(0., 0.)).collect();
        let metrics = TextMetrics::new(&glyphs);
        let handle = self.subpass.text_pipeline.add_section(glyphs);
        self.text_changed = true;
        (Drawable::Text(handle), metrics)
    }

    pub fn draw(&mut self, drawable: &Drawable, rect: Rect, color: Color) {
        self.subpass.pending_draw_commands.push(DrawCommand {
            drawable: drawable.clone(),
            rect,
            color: encode_color(color),
        });
    }

    fn update_cache(&mut self, builder: &mut AutoCommandBufferBuilder) {
        if self.text_changed {
            self.subpass.text_pipeline.update_cache(builder);
            self.text_changed = false;
        }
    }
}

pub struct GuiSubpass {
    texture_rect_pipeline: TextureRectPipeline,
    text_pipeline: TextPipeline,
    screen_dimensions: Size,
    pending_draw_commands: Vec<DrawCommand>,

    white_1x1: Texture,
}

impl GuiSubpass {
    fn make_context<'a>(&'a mut self) -> DrawContext<'a> {
        self.pending_draw_commands.clear();
        DrawContext { subpass: self, text_changed: false }
    }

    pub fn load_image(&mut self, subpass_setup: &mut SubpassSetup, image: &Image) -> Texture {
        self.texture_rect_pipeline.load_image(subpass_setup, image, Filter::Linear)
    }
    // pub fn load_nine_slice_image(&mut self, subpass_setup: &mut SubpassSetup, image: &NineSliceImage) -> Texture {
    //     self.pipeline.texture_rect.load_image(subpass_setup, image, Filter::Linear);
    // }
}

impl RenderSubpass for GuiSubpass {
    type SubpassCategory = subpass::Gui;
    type Scene = Gui;
    fn contents() -> SubpassContents { SubpassContents::Inline }
    fn new(subpass_setup: &mut SubpassSetup) -> Self {
        let mut texture_rect_pipeline = TextureRectPipeline::new(subpass_setup);
        let text_pipeline = TextPipeline::new(subpass_setup);
        let white_1x1 = texture_rect_pipeline.load_image(subpass_setup, &Image::new_1x1_white(), Filter::Nearest);

        GuiSubpass {
            texture_rect_pipeline,
            text_pipeline,
            screen_dimensions: Size::zero(),
            pending_draw_commands: Vec::new(),
            white_1x1,
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
            match &draw_command.drawable {
                Drawable::TextureRect(texture) => 
                    self.texture_rect_pipeline.draw_rect(builder, dynamic_state, texture, draw_command.texture_rect_constants(screen_dimensions)),
                Drawable::TextureNineSlice(texture) =>
                    self.texture_rect_pipeline.draw_nine_slice(builder, dynamic_state, texture, draw_command.texture_rect_constants(screen_dimensions)),
                Drawable::Text(handle) =>
                    self.text_pipeline.draw(builder, dynamic_state, handle, draw_command.text_constants(screen_dimensions)),
            }
        }
    }
}
