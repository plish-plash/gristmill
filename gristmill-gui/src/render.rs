use crate::{GuiDraw, GuiNodeStorage};
use glyph_brush::*;
use gristmill_core::{
    geom2d::{IRect, Rect, Size},
    math::IVec2,
};
use gristmill_render::{
    texture_rect::{TextureRect, TextureRectRenderer},
    ImageDimensionsExt, RenderContext, Texture,
};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::CopyBufferToImageInfo,
    format::Format,
    image::{
        view::{ImageView, ImageViewCreateInfo},
        ImageCreateFlags, ImageDimensions, ImageUsage, ImageViewAbstract, StorageImage,
    },
    sampler::{ComponentMapping, ComponentSwizzle},
};

fn text_screen_position(rect: IRect, layout: Layout<BuiltInLineBreaker>) -> IVec2 {
    let (h_align, v_align) = match layout {
        Layout::SingleLine {
            h_align, v_align, ..
        } => (h_align, v_align),
        Layout::Wrap {
            h_align, v_align, ..
        } => (h_align, v_align),
    };
    let x = match h_align {
        HorizontalAlign::Left => rect.top_left().x,
        HorizontalAlign::Center => rect.center().x,
        HorizontalAlign::Right => rect.bottom_right().x,
    };
    let y = match v_align {
        VerticalAlign::Top => rect.top_left().y,
        VerticalAlign::Center => rect.center().y,
        VerticalAlign::Bottom => rect.bottom_right().y,
    };
    IVec2::new(x, y)
}

pub struct GuiRenderer {
    rect_renderer: TextureRectRenderer,
    glyph_brush: GlyphBrush<TextureRect>,
    glyph_texture: Texture,
    glyph_draw: Vec<TextureRect>,
}

impl GuiRenderer {
    pub fn new(context: &mut RenderContext) -> Self {
        let font =
            ab_glyph::FontArc::try_from_slice(include_bytes!("./OpenSans-Regular.ttf")).unwrap();
        let glyph_brush = GlyphBrushBuilder::using_font(font)
            .multithread(false)
            .build();
        let glyph_texture =
            Self::create_glyph_texture(context, glyph_brush.texture_dimensions().into());

        GuiRenderer {
            rect_renderer: TextureRectRenderer::new(context),
            glyph_brush,
            glyph_texture,
            glyph_draw: Vec::new(),
        }
    }

    pub fn rect_renderer(&mut self) -> &mut TextureRectRenderer {
        &mut self.rect_renderer
    }

    fn glyph_vertex(glyph_texture: &Texture, glyph: GlyphVertex) -> TextureRect {
        fn convert_rect(rect: ab_glyph::Rect) -> Rect {
            [rect.min.x, rect.min.y, rect.width(), rect.height()].into()
        }
        TextureRect {
            texture: Some(glyph_texture.clone()),
            rect: convert_rect(glyph.pixel_coords),
            uv_rect: convert_rect(glyph.tex_coords),
            color: gristmill_core::Color::from(glyph.extra.color),
            z: glyph.extra.z as u16,
        }
    }
    fn create_glyph_texture(context: &mut RenderContext, dimensions: Size) -> Texture {
        let image = StorageImage::with_usage(
            context.allocator(),
            ImageDimensions::from_size(dimensions),
            Format::R8_SRGB,
            ImageUsage {
                transfer_dst: true,
                sampled: true,
                ..ImageUsage::empty()
            },
            ImageCreateFlags::empty(),
            [context.queue().queue_family_index()],
        )
        .unwrap();
        let mut image_info = ImageViewCreateInfo::from_image(&image);
        image_info.component_mapping = ComponentMapping {
            r: ComponentSwizzle::One,
            g: ComponentSwizzle::One,
            b: ComponentSwizzle::One,
            a: ComponentSwizzle::Red,
        };
        let image_view: Arc<dyn ImageViewAbstract> = ImageView::new(image, image_info).unwrap();
        let texture: Texture = image_view.into();
        texture
    }
    fn update_glyph_texture(
        context: &mut RenderContext,
        glyph_texture: &Texture,
        region: Rectangle<u32>,
        tex_data: &[u8],
    ) {
        let transfer_buffer = CpuAccessibleBuffer::from_iter(
            context.allocator(),
            BufferUsage {
                transfer_src: true,
                ..BufferUsage::empty()
            },
            false,
            tex_data.iter().cloned(),
        )
        .unwrap();
        let mut copy_info =
            CopyBufferToImageInfo::buffer_image(transfer_buffer, glyph_texture.image());
        copy_info.regions[0].image_offset = [region.min[0], region.min[1], 0];
        copy_info.regions[0].image_extent = [region.width(), region.height(), 1];
        context.builder().copy_buffer_to_image(copy_info).unwrap();
    }

    pub fn process(&mut self, context: &mut RenderContext, nodes: &GuiNodeStorage) {
        for (_, node) in nodes.iter() {
            if !node.visible {
                continue;
            }
            match &node.draw {
                GuiDraw::None => (),
                GuiDraw::Rect(texture, color) => {
                    let (rect, z) = node.draw_rect();
                    self.rect_renderer.queue(TextureRect {
                        texture: texture.clone(),
                        rect: rect.as_rect(),
                        uv_rect: Rect::ONE,
                        color: *color,
                        z,
                    });
                }
                GuiDraw::Text(owned_section) => {
                    let (rect, z) = node.draw_rect();
                    let mut section = owned_section.to_borrowed();
                    section.screen_position =
                        text_screen_position(rect, section.layout).as_vec2().into();
                    section.bounds = rect.size.as_vec2().into();
                    for text in section.text.iter_mut() {
                        text.extra.z = z as f32;
                    }
                    self.glyph_brush.queue(section);
                }
            }
        }

        // Process queued text.
        let mut brush_action;
        loop {
            brush_action = self.glyph_brush.process_queued(
                |region, tex_data| {
                    Self::update_glyph_texture(context, &self.glyph_texture, region, tex_data)
                },
                |glyph| Self::glyph_vertex(&self.glyph_texture, glyph),
            );
            // If the cache texture is too small to fit all the glyphs, resize and try again.
            match brush_action {
                Ok(_) => break,
                Err(BrushError::TextureTooSmall { suggested, .. }) => {
                    let dimensions = suggested.into();
                    log::debug!("Resizing glyph texture to {}.", dimensions);
                    self.rect_renderer.remove(&self.glyph_texture);
                    self.glyph_texture = Self::create_glyph_texture(context, dimensions);
                    self.glyph_brush
                        .resize_texture(dimensions.width, dimensions.height);
                }
            }
        }
        // If the text has changed from what was last drawn, upload the new vertices to GPU.
        match brush_action.unwrap() {
            BrushAction::Draw(vertices) => self.glyph_draw = vertices,
            BrushAction::ReDraw => (),
        }
        self.rect_renderer
            .queue_all(self.glyph_draw.iter().cloned());
    }

    pub fn draw_all(&mut self, context: &mut RenderContext) {
        self.rect_renderer.draw_all(context);
    }
}
