use std::{borrow::Cow, io::Read, path::Path};

use emath::{Align, Align2, Pos2, Rect, RectTransform};
use glyph_brush::*;

use crate::{
    asset::*,
    color::Color,
    render2d::{Quad, UNIT_RECT},
    Dispatcher,
};

pub use glyph_brush::ab_glyph::FontArc as FontAsset;

impl Asset for FontAsset {
    fn load(path: &Path) -> Result<Self> {
        let mut reader = load_file(path)?;
        let mut data = Vec::new();
        reader
            .read_to_end(&mut data)
            .map_err(|e| AssetError::new_io(path.to_owned(), false, e))?;
        FontAsset::try_from_vec(data).map_err(|e| AssetError::new_format(path.to_owned(), false, e))
    }
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct FontId(usize);

#[derive(Clone, Copy)]
pub struct Font(FontId, f32);

impl Default for Font {
    fn default() -> Self {
        Font(FontId::default(), 24.0)
    }
}

impl Font {
    pub fn new(id: FontId, scale: f32) -> Self {
        Font(id, scale)
    }
    pub fn id(&self) -> FontId {
        self.0
    }
    pub fn scale(&self) -> f32 {
        self.1
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

#[derive(Clone)]
struct Extra {
    id: usize,
    color: Color,
}

impl PartialEq for Extra {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl std::hash::Hash for Extra {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

type Vertex = (usize, Quad);

pub trait GlyphTexture {
    fn resize(&mut self, width: u32, height: u32);
    fn update(&mut self, min: [u32; 2], max: [u32; 2], data: &[u8]);
}

pub struct TextDrawQueue {
    dispatcher: Dispatcher,
    glyph_brush: GlyphBrush<Vertex, Extra>,
    current_id: usize,
    current_id_used: bool,
    screen_transform: RectTransform,
    vertices: Vec<(usize, Quad)>,
    quads: Vec<Quad>,
    barriers: Vec<usize>,
    current_barrier: usize,
}

impl TextDrawQueue {
    pub fn new(dispatcher: Dispatcher, fonts: Vec<FontAsset>) -> Self {
        let glyph_brush = GlyphBrushBuilder::using_fonts(fonts)
            .multithread(false)
            .build();
        TextDrawQueue {
            dispatcher,
            glyph_brush,
            current_id: 0,
            current_id_used: false,
            screen_transform: RectTransform::identity(Rect::ZERO),
            vertices: Vec::new(),
            quads: Vec::new(),
            barriers: Vec::new(),
            current_barrier: 0,
        }
    }
    pub fn glyph_texture_size(&self) -> (u32, u32) {
        self.glyph_brush.texture_dimensions()
    }

    fn to_vertex(vertex: GlyphVertex<Extra>) -> Vertex {
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
        let texture_rect = Rect::from_min_max(to_pos(tex_coords.min), to_pos(tex_coords.max));
        (
            extra.id,
            Quad {
                rect,
                texture_rect,
                color: extra.color,
            },
        )
    }

    pub fn start(&mut self, screen_transform: RectTransform) {
        if self.screen_transform.to() != screen_transform.to() {
            self.quads.clear();
        }
        self.screen_transform = screen_transform;
        self.current_id = 0;
        self.current_id_used = false;
        self.current_barrier = 0;
    }

    pub fn queue(&mut self, text: &Text) {
        let position = self.screen_transform.transform_pos(text.position);
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
        let bounds_width = text.wrap.unwrap_or(f32::INFINITY);
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
            scale: text.font.scale().into(),
            font_id: glyph_brush::FontId(text.font.0 .0),
            extra: Extra {
                id: self.current_id,
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
        self.current_id_used = true;
    }
    pub fn dispatch(&mut self) {
        if self.current_id_used {
            self.current_id += 1;
            self.current_id_used = false;
            self.dispatcher.dispatch();
        }
    }

    pub fn finish<T: GlyphTexture>(&mut self, glyph_texture: &mut T) {
        let transform = RectTransform::from_to(*self.screen_transform.to(), UNIT_RECT);
        let mut brush_action;
        loop {
            brush_action = self.glyph_brush.process_queued(
                |rect, data| glyph_texture.update(rect.min, rect.max, data),
                |vertex| Self::to_vertex(vertex),
            );

            // If the cache texture is too small to fit all the glyphs, resize and try again
            match brush_action {
                Ok(_) => break,
                Err(BrushError::TextureTooSmall {
                    suggested: (width, height),
                    ..
                }) => {
                    // Recreate texture as a larger size to fit more
                    log::trace!("Resizing glyph texture to {}x{}", width, height);
                    glyph_texture.resize(width, height);
                    self.glyph_brush.resize_texture(width, height);
                }
            }
        }

        // If the text has changed from what was last drawn, store new vertices
        match brush_action.unwrap() {
            BrushAction::Draw(vertices) => {
                self.vertices = vertices;
                self.vertices.sort_unstable_by_key(|(id, _)| *id);
            }
            BrushAction::ReDraw => (),
        }
        if self.quads.len() != self.vertices.len() {
            self.barriers.clear();
            self.barriers.push(0);
            let mut last_id = None;
            for (index, (id, _)) in self.vertices.iter().enumerate() {
                if last_id != Some(*id) {
                    if last_id.is_some() {
                        self.barriers.push(index);
                    }
                    last_id = Some(*id);
                }
            }
            if last_id.is_some() {
                self.barriers.push(self.vertices.len());
            }
            self.quads = self
                .vertices
                .iter()
                .map(|(_, quad)| Quad {
                    rect: transform.transform_rect(quad.rect),
                    texture_rect: quad.texture_rect,
                    color: quad.color,
                })
                .collect();
        }
    }
    pub fn draw_next(&mut self) -> &[Quad] {
        let previous_barrier = self.current_barrier;
        self.current_barrier += 1;
        let start = self.barriers[previous_barrier];
        let end = self.barriers[self.current_barrier];
        &self.quads[start..end]
    }
}
