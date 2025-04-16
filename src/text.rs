use std::{
    borrow::Cow,
    collections::HashMap,
    hash::{Hash, Hasher},
    io::Read,
    path::{Path, PathBuf},
};

use emath::{Align, Align2, OrderedFloat, Pos2, Rect, TSTransform};
use glyph_brush::*;
use serde::Deserialize;

use crate::{
    asset::{self, Asset, AssetError, YamlAsset},
    color::Color,
    impl_sub_asset,
    scene2d::Instance,
    style::{style, style_or},
    Batcher, Buffer, Pipeline, Size,
};

impl Asset for ab_glyph::FontArc {
    fn load(path: &Path) -> asset::Result<Self> {
        let mut reader = asset::load_file(path)?;
        let mut data = Vec::new();
        reader
            .read_to_end(&mut data)
            .map_err(|e| AssetError::new_io(path.to_owned(), false, e))?;
        ab_glyph::FontArc::try_from_vec(data)
            .map_err(|e| AssetError::new_format(path.to_owned(), false, e))
    }
}

#[derive(Deserialize, Clone)]
#[serde(try_from = "PathBuf")]
pub struct FontAsset(ab_glyph::FontArc);

impl_sub_asset!(FontAsset, "fonts", false);

impl YamlAsset for Vec<FontAsset> {}

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
    pub fn from_style(class: &str) -> Self {
        Font {
            font_id: FontId(style(class, "font-id")),
            scale: style_or(class, "font-scale", 24.0),
        }
    }
}

#[derive(Clone)]
pub struct Text<'a> {
    pub position: Pos2,
    pub align: Align2,
    pub wrap: Option<f32>,
    pub font: Font,
    pub color: Color,
    pub text: Cow<'a, str>,
}

#[derive(Clone, PartialEq)]
struct Extra<Layer> {
    layer: Layer,
    color: Color,
}

impl<Layer: Hash> Hash for Extra<Layer> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.layer.hash(state);
        OrderedFloat(self.color.r).hash(state);
        OrderedFloat(self.color.g).hash(state);
        OrderedFloat(self.color.b).hash(state);
        OrderedFloat(self.color.a).hash(state);
    }
}

pub trait GlyphTexture<P: Pipeline + ?Sized> {
    fn resize(&mut self, context: &mut P::Context, size: Size);
    fn update(&mut self, context: &mut P::Context, min: [u32; 2], max: [u32; 2], data: &[u8]);
    fn material(&self) -> P::Material;
}

pub trait TextPipeline: Pipeline<Instance = Instance, Camera = TSTransform> {
    fn glyph_texture(&mut self) -> &mut impl GlyphTexture<Self>;
    fn set_clip(&mut self, context: &mut Self::Context, clip: Option<Rect>);
}

pub struct TextBrush<P: TextPipeline, L> {
    glyph_brush: GlyphBrush<(L, Instance), Extra<L>>,
    vertices: HashMap<L, P::InstanceBuffer>,
}

impl<P, L> TextBrush<P, L>
where
    P: TextPipeline,
    L: Clone + Eq + Hash + 'static,
{
    pub fn new(fonts: Vec<FontAsset>) -> Self {
        let fonts = fonts.into_iter().map(|font| font.0).collect();
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

    fn to_rect(rect: ab_glyph::Rect) -> Rect {
        fn to_pos(point: ab_glyph::Point) -> Pos2 {
            Pos2::new(point.x, point.y)
        }
        Rect::from_min_max(to_pos(rect.min), to_pos(rect.max))
    }
    fn to_vertex(vertex: GlyphVertex<Extra<L>>) -> (L, Instance) {
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
        (
            vertex.extra.layer.clone(),
            Instance {
                rect: Self::to_rect(pixel_coords),
                uv: Self::to_rect(tex_coords).into(),
                color: extra.color,
            },
        )
    }
    fn make_layout(align: Align2, wrap: bool) -> Layout<BuiltInLineBreaker> {
        let h_align = match align.x() {
            Align::Min => HorizontalAlign::Left,
            Align::Center => HorizontalAlign::Center,
            Align::Max => HorizontalAlign::Right,
        };
        let v_align = match align.y() {
            Align::Min => VerticalAlign::Top,
            Align::Center => VerticalAlign::Center,
            Align::Max => VerticalAlign::Bottom,
        };
        if wrap {
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
        }
    }

    pub fn text_bounds(&mut self, layer: L, text: &Text) -> Option<Rect> {
        let position = text.position;
        let bounds_width = text.wrap.unwrap_or(f32::INFINITY);
        let layout = Self::make_layout(text.align, text.wrap.is_some());
        let text = glyph_brush::Text {
            text: &text.text,
            scale: text.font.scale.into(),
            font_id: text.font.font_id,
            extra: Extra {
                layer,
                color: text.color,
            },
        };
        self.glyph_brush
            .glyph_bounds(
                Section::builder()
                    .with_screen_position(position)
                    .with_bounds((bounds_width, f32::INFINITY))
                    .with_layout(layout)
                    .add_text(text),
            )
            .map(Self::to_rect)
    }

    pub fn queue(&mut self, layer: L, text: &Text) {
        let position = text.position;
        let bounds_width = text.wrap.unwrap_or(f32::INFINITY);
        let layout = Self::make_layout(text.align, text.wrap.is_some());
        let text = glyph_brush::Text {
            text: &text.text,
            scale: text.font.scale.into(),
            font_id: text.font.font_id,
            extra: Extra {
                layer,
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

    pub fn process(&mut self, pipeline: &mut P, context: &mut P::Context) {
        let glyph_texture = pipeline.glyph_texture();
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
                for buffer in self.vertices.values_mut() {
                    buffer.clear();
                }
                for (layer, instance) in vertices {
                    self.vertices
                        .entry(layer)
                        .or_insert_with(|| P::InstanceBuffer::new())
                        .push(instance);
                }
            }
            BrushAction::ReDraw => (),
        }
    }

    pub fn draw_layer(&mut self, batcher: &mut Batcher<P>, layer: &L) {
        if let Some(instances) = self.vertices.get_mut(layer) {
            batcher.flush();
            let material = batcher.pipeline.glyph_texture().material();
            batcher
                .pipeline
                .draw(batcher.context, &batcher.camera, &material, instances);
        }
    }
}
