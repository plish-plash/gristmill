use std::{
    borrow::Cow,
    collections::HashMap,
    hash::{Hash, Hasher},
    io::Read,
    path::Path,
};

use emath::{Align, Align2, Pos2, Rect};
use glyph_brush::*;

use crate::{
    asset::{self, Asset, AssetError},
    color::Color,
    scene2d::Instance,
    Scene, Size,
};

pub use glyph_brush::ab_glyph::FontArc as FontAsset;

impl Asset for FontAsset {
    fn load(path: &Path) -> asset::Result<Self> {
        let mut reader = asset::load_file(path)?;
        let mut data = Vec::new();
        reader
            .read_to_end(&mut data)
            .map_err(|e| AssetError::new_io(path.to_owned(), false, e))?;
        FontAsset::try_from_vec(data).map_err(|e| AssetError::new_format(path.to_owned(), false, e))
    }
}

#[derive(Clone, Copy)]
pub struct Font {
    font_id: FontId,
    scale: f32,
}

impl Default for Font {
    fn default() -> Self {
        Font {
            font_id: FontId::default(),
            scale: 24.0,
        }
    }
}
impl Font {
    pub fn new(font_id: usize, scale: f32) -> Self {
        Font {
            font_id: FontId(font_id),
            scale,
        }
    }
}

#[derive(Clone)]
pub struct Text<'a, L> {
    pub layer: L,
    pub position: Pos2,
    pub align: Align2,
    pub wrap: Option<f32>,
    pub font: Font,
    pub color: Color,
    pub text: Cow<'a, str>,
}

#[derive(Clone, PartialEq)]
struct Extra<L> {
    layer: L,
    color: Color,
}

impl<L: Hash> Hash for Extra<L> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.layer.hash(state);
    }
}

pub trait GlyphTexture {
    type Context;
    type DrawParams: Clone + Eq + Hash;
    fn resize(&mut self, context: &mut Self::Context, size: Size);
    fn update(&mut self, context: &mut Self::Context, min: [u32; 2], max: [u32; 2], data: &[u8]);
    fn draw_params(&self) -> Self::DrawParams;
}

pub struct TextBrush<L> {
    glyph_brush: GlyphBrush<(L, Instance), Extra<L>>,
    vertices: HashMap<L, Vec<Instance>>,
}

impl<L> TextBrush<L>
where
    L: Clone + Ord + PartialEq + Hash + 'static,
{
    pub fn new(fonts: Vec<FontAsset>) -> Self {
        let glyph_brush = GlyphBrushBuilder::using_fonts(fonts)
            .multithread(false)
            .build();
        TextBrush {
            glyph_brush,
            vertices: HashMap::new(),
        }
    }
    pub fn glyph_texture_size(&self) -> Size {
        let (width, height) = self.glyph_brush.texture_dimensions();
        Size { width, height }
    }

    fn to_vertex(vertex: GlyphVertex<Extra<L>>) -> (L, Instance) {
        fn to_pos(point: ab_glyph::Point) -> Pos2 {
            Pos2::new(point.x, point.y)
        }

        let GlyphVertex {
            mut tex_coords,
            mut pixel_coords,
            bounds,
            extra,
        } = vertex;

        // handle overlapping bounds, modify uv_rect to preserve texture aspect
        if pixel_coords.max.x > bounds.max.x {
            let old_width = pixel_coords.width();
            pixel_coords.max.x = bounds.max.x;
            tex_coords.max.x =
                tex_coords.min.x + tex_coords.width() * pixel_coords.width() / old_width;
        }
        if pixel_coords.min.x < bounds.min.x {
            let old_width = pixel_coords.width();
            pixel_coords.min.x = bounds.min.x;
            tex_coords.min.x =
                tex_coords.max.x - tex_coords.width() * pixel_coords.width() / old_width;
        }
        if pixel_coords.max.y > bounds.max.y {
            let old_height = pixel_coords.height();
            pixel_coords.max.y = bounds.max.y;
            tex_coords.max.y =
                tex_coords.min.y + tex_coords.height() * pixel_coords.height() / old_height;
        }
        if pixel_coords.min.y < bounds.min.y {
            let old_height = pixel_coords.height();
            pixel_coords.min.y = bounds.min.y;
            tex_coords.min.y =
                tex_coords.max.y - tex_coords.height() * pixel_coords.height() / old_height;
        }

        let rect = Rect::from_min_max(to_pos(pixel_coords.min), to_pos(pixel_coords.max));
        let uv = Rect::from_min_max(to_pos(tex_coords.min), to_pos(tex_coords.max)).into();
        (
            vertex.extra.layer.clone(),
            Instance {
                rect,
                uv,
                color: extra.color,
            },
        )
    }

    pub fn queue(&mut self, text: &Text<L>) {
        let position = text.position;
        let bounds_width = text.wrap.unwrap_or(f32::INFINITY);
        let h_align = match text.align.x() {
            Align::Min => HorizontalAlign::Left,
            Align::Center => HorizontalAlign::Center,
            Align::Max => HorizontalAlign::Right,
        };
        let v_align = match text.align.y() {
            Align::Min => VerticalAlign::Top,
            Align::Center => VerticalAlign::Center,
            Align::Max => VerticalAlign::Bottom,
        };
        let layout = if text.wrap.is_some() {
            Layout::Wrap {
                line_breaker: BuiltInLineBreaker::default(),
                h_align,
                v_align,
            }
        } else {
            Layout::SingleLine {
                line_breaker: BuiltInLineBreaker::default(),
                h_align,
                v_align,
            }
        };
        let text = glyph_brush::Text {
            text: &text.text,
            scale: text.font.scale.into(),
            font_id: text.font.font_id,
            extra: Extra {
                layer: text.layer.clone(),
                color: text.color,
            },
        };
        self.glyph_brush.queue(
            Section::builder()
                .with_screen_position(position)
                .with_bounds((bounds_width, f32::INFINITY))
                .with_layout(layout)
                .add_text(text),
        );
    }

    pub fn draw<T: GlyphTexture>(
        &mut self,
        context: &mut T::Context,
        glyph_texture: &mut T,
        scene: &mut Scene<L, T::DrawParams, Instance>,
    ) {
        let mut brush_action;
        loop {
            // Process the queued glyphs.
            brush_action = self.glyph_brush.process_queued(
                |rect, data| glyph_texture.update(context, rect.min, rect.max, data),
                move |vertex| Self::to_vertex(vertex),
            );

            // If the cache texture is too small to fit all the glyphs, resize and try again.
            match brush_action {
                Ok(_) => break,
                Err(BrushError::TextureTooSmall {
                    suggested: (width, height),
                    ..
                }) => {
                    // Recreate the cache texture with a larger size.
                    log::trace!("Resizing glyph texture to {}x{}", width, height);
                    glyph_texture.resize(context, Size { width, height });
                    self.glyph_brush.resize_texture(width, height);
                }
            }
        }

        // If the text has changed from what was last drawn, store new vertices.
        match brush_action.unwrap() {
            BrushAction::Draw(vertices) => {
                self.vertices.clear();
                for (layer, instance) in vertices {
                    self.vertices.entry(layer).or_default().push(instance);
                }
            }
            BrushAction::ReDraw => (),
        }

        // Draw the stored vertices.
        let params = glyph_texture.draw_params();
        for (layer, instances) in self.vertices.iter() {
            scene.queue_all(layer.clone(), params.clone(), instances.iter().cloned());
        }
    }
}
