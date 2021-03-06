mod text;
mod texture_rect;

use std::sync::Arc;

use vulkano::command_buffer::{AutoCommandBufferBuilder, SubpassContents};
use vulkano::sampler::Filter;

use rusttype::{PositionedGlyph, Scale, point};

use gristmill::asset::{Asset, image::{Image, NineSliceImage}};
use gristmill::color::{Color, encode_color};
use gristmill::renderer::{RenderAsset, LoadContext, RenderContext, scene};
use gristmill::renderer::loader::RenderAssetLoader;
use gristmill::geometry2d::{Rect, Size};
use super::{Gui, font::{Font, fonts}};

use text::{TextHandle, TextPipeline};
use texture_rect::{Texture, NineSliceTexture, TextureRectPipeline};

type TextureRectConstants = texture_rect::PushConstants;
type TextConstants = text::PushConstants;

#[derive(Clone)]
pub enum GuiTexture {
    None,
    Simple(Texture),
    NineSlice(NineSliceTexture),
}

impl GuiTexture {
    pub fn size(&self) -> Option<Size> {
        match self {
            GuiTexture::None => None,
            GuiTexture::Simple(texture) => Some(texture.size()),
            GuiTexture::NineSlice(_) => None,
        }
    }
}

impl RenderAsset for GuiTexture {
    fn type_name() -> &'static str { "GuiTexture" }
    fn none() -> Self { GuiTexture::None }
}

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
    None,
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
    fn empty() -> TextMetrics {
        TextMetrics { width: 0.0, v_metrics: rusttype::VMetrics { ascent: 0.0, descent: 0.0, line_gap: 0.0 } }
    }

    pub fn width(&self) -> f32 { self.width }
    pub fn ascent(&self) -> f32 { self.v_metrics.ascent }
    pub fn height(&self) -> f32 { self.v_metrics.ascent - self.v_metrics.descent }
}

pub struct DrawContext<'a> {
    render: &'a mut GuiRenderer,
    text_changed: bool,
}

impl<'a> DrawContext<'a> {
    pub fn new_color_rect_drawable(&mut self) -> Drawable {
        self.new_texture_rect_drawable(GuiTexture::None)
    }
    pub fn new_texture_rect_drawable(&mut self, texture: GuiTexture) -> Drawable {
        match texture {
            GuiTexture::None => Drawable::TextureRect(self.render.white_1x1.clone()),
            GuiTexture::Simple(tex) => Drawable::TextureRect(tex),
            GuiTexture::NineSlice(tex) => Drawable::TextureNineSlice(tex),
        }
    }
    pub fn new_text_drawable(&mut self, font: Font, size: f32, text: &str) -> (Drawable, TextMetrics) {
        if text.is_empty() {
            log::warn!("Can't create text Drawable with empty string");
            return (Drawable::None, TextMetrics::empty());
        }
        let font_asset = match fonts().get(font) {
            Some(font) => font,
            None => return (Drawable::None, TextMetrics::empty()),
        };
        let glyphs: Vec<PositionedGlyph> = font_asset.layout(text, Scale::uniform(size), point(0., 0.)).collect();
        let metrics = TextMetrics::new(&glyphs);
        let handle = self.render.text_pipeline.add_section(glyphs);
        self.text_changed = true;
        (Drawable::Text(handle), metrics)
    }

    pub fn draw(&mut self, drawable: &Drawable, mut rect: Rect, color: Color) {
        if let Drawable::TextureNineSlice(tex) = drawable {
            rect = rect.inset(tex.slices());
        }
        self.render.pending_draw_commands.push(DrawCommand {
            drawable: drawable.clone(),
            rect,
            color: encode_color(color),
        });
    }

    fn update_cache(&mut self, builder: &mut AutoCommandBufferBuilder) {
        if self.text_changed {
            self.render.text_pipeline.update_cache(builder);
            self.text_changed = false;
        }
    }
}

pub struct GuiRenderer {
    texture_rect_pipeline: TextureRectPipeline,
    text_pipeline: TextPipeline,
    screen_dimensions: Size,
    pending_draw_commands: Vec<DrawCommand>,

    white_1x1: Texture,
}

impl GuiRenderer {
    fn make_context<'a>(&'a mut self) -> DrawContext<'a> {
        self.pending_draw_commands.clear();
        DrawContext { render: self, text_changed: false }
    }

    pub fn load_image(&mut self, context: &mut LoadContext, image: &Image) -> GuiTexture {
        GuiTexture::Simple(self.texture_rect_pipeline.load_image(context, image, Filter::Linear))
    }
    pub fn load_nine_slice_image(&mut self, context: &mut LoadContext, image: &NineSliceImage) -> GuiTexture {
        GuiTexture::NineSlice(self.texture_rect_pipeline.load_nine_slice_image(context, image))
    }
}

impl scene::SceneRenderer for GuiRenderer {
    type RenderType = scene::Geometry2D;
    type Scene = Gui;
    fn contents() -> SubpassContents { SubpassContents::Inline }
    fn new(context: &mut LoadContext) -> Self {
        let mut texture_rect_pipeline = TextureRectPipeline::new(context);
        let text_pipeline = TextPipeline::new(context);
        let white_1x1 = texture_rect_pipeline.load_image(context, &Image::new_1x1_white(), Filter::Nearest);

        GuiRenderer {
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

    fn pre_render(&mut self, context: &mut RenderContext, scene: &mut Gui) {
        scene.layout_if_needed(self.screen_dimensions);
        let mut draw_context = self.make_context();
        scene.draw(&mut draw_context);
        draw_context.update_cache(context.command_buffer_builder());
    }

    fn render(&mut self, context: &mut RenderContext, _scene: &mut Gui) {
        let screen_dimensions = self.screen_dimensions.into();
        for draw_command in self.pending_draw_commands.drain(..) {
            match &draw_command.drawable {
                Drawable::None => (),
                Drawable::TextureRect(texture) => 
                    self.texture_rect_pipeline.draw_rect(context, texture, draw_command.texture_rect_constants(screen_dimensions)),
                Drawable::TextureNineSlice(texture) =>
                    self.texture_rect_pipeline.draw_nine_slice(context, texture, draw_command.texture_rect_constants(screen_dimensions)),
                Drawable::Text(handle) =>
                    self.text_pipeline.draw(context, handle, draw_command.text_constants(screen_dimensions)),
            }
        }
    }
}

impl RenderAssetLoader for GuiRenderer {
    type RenderAsset = GuiTexture;
    fn name() -> &'static str { "gui" }
    fn load(&mut self, context: &mut LoadContext, asset_type: &str, asset_path: &str) -> Option<Self::RenderAsset> {
        match asset_type {
            "" => {
                Image::try_read(asset_path).map(|image| self.load_image(context, &image))
            }
            "nine_slice" => {
                NineSliceImage::try_read(asset_path).map(|image| self.load_nine_slice_image(context, &image))
            }
            _ => {
                log::warn!("Invalid asset type \"{}\" for loader {}", asset_type, Self::name());
                None
            }
        }
        
    }
}
