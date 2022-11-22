use glyph_brush::{ab_glyph::FontArc, *};
use gristmill::{
    geom2d::{Rect, Size},
    math::IVec2,
};
use std::{collections::HashMap, hash::Hash};

use crate::{Gui, GuiDraw};

#[derive(Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct GuiTexture(pub(crate) usize);

pub trait Renderer {
    type Vertex: Clone + 'static;
    type Context;
    fn make_rect_vertex(&self, rect: GuiDrawRect) -> Self::Vertex;
    fn make_glyph_vertex(&self, glyph: GlyphVertex) -> Self::Vertex;
    fn resize_glyph_texture(&mut self, context: &mut Self::Context, size: Size) -> GuiTexture;
    fn update_glyph_texture(&self, context: &mut Self::Context, region: Rect, tex_data: &[u8]);
    fn set_vertices(
        &mut self,
        context: &mut Self::Context,
        texture: GuiTexture,
        vertices: Vec<Self::Vertex>,
        screen_size: Size,
    );
}

#[derive(Clone, PartialEq)]
pub struct GuiDrawRect {
    pub rect: Rect,
    pub texture: GuiTexture,
    pub color: gristmill::Color,
}

struct RectCache(HashMap<GuiTexture, Vec<GuiDrawRect>>);

impl RectCache {
    fn new() -> RectCache {
        RectCache(HashMap::new())
    }
    fn get(&self, texture: &GuiTexture) -> &Vec<GuiDrawRect> {
        self.0.get(texture).unwrap()
    }
    fn create_frame_cache(&self) -> RectCache {
        RectCache(
            self.0
                .iter()
                .map(|(k, v)| (*k, Vec::with_capacity(v.len())))
                .collect(),
        )
    }
    fn queue(&mut self, rect: GuiDrawRect) {
        self.0.entry(rect.texture).or_default().push(rect);
    }
    fn process_frame_cache(&mut self, frame_cache: RectCache) -> Vec<GuiTexture> {
        let mut changed_textures = Vec::new();
        for (k, v) in frame_cache.0.iter() {
            if let Some(prev) = self.0.get(k) {
                if v.as_slice() != prev.as_slice() {
                    changed_textures.push(*k);
                }
            } else {
                changed_textures.push(*k);
            }
        }
        *self = frame_cache;
        changed_textures
    }
}

pub struct GuiRenderBrush<R: Renderer> {
    rect_cache: RectCache,
    glyph_brush: GlyphBrush<R::Vertex>,
    glyph_texture: GuiTexture,
}

fn text_screen_position(rect: Rect, layout: Layout<BuiltInLineBreaker>) -> IVec2 {
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
fn glyph_rect_to_gui_rect(rect: Rectangle<u32>) -> Rect {
    Rect::new(
        IVec2::new(rect.min[0] as i32, rect.min[1] as i32),
        Size::new(rect.width(), rect.height()),
    )
}

impl<R: Renderer> GuiRenderBrush<R> {
    pub fn new(context: &mut R::Context, renderer: &mut R) -> GuiRenderBrush<R> {
        let font = FontArc::try_from_slice(include_bytes!("./OpenSans-Regular.ttf")).unwrap();
        let glyph_brush = GlyphBrushBuilder::using_font(font)
            .multithread(false)
            .build();
        let glyph_texture =
            renderer.resize_glyph_texture(context, glyph_brush.texture_dimensions().into());
        GuiRenderBrush {
            rect_cache: RectCache::new(),
            glyph_brush,
            glyph_texture,
        }
    }
    pub(crate) fn set_viewport_rect(&mut self, gui: &mut Gui, viewport: Rect) {
        gui.viewport = viewport;
        // TODO invalidate caches.
    }

    pub fn process(&mut self, context: &mut R::Context, renderer: &mut R, gui: &mut Gui) {
        // TODO profile and optimize for big guis.
        let mut frame_cache = self.rect_cache.create_frame_cache();
        for (_, node) in gui.nodes.read().iter() {
            if !node.visible.get() {
                continue;
            }
            match &node.draw {
                GuiDraw::None => (),
                &GuiDraw::Rect(texture, color) => frame_cache.queue(GuiDrawRect {
                    rect: node.get_draw_rect(),
                    texture,
                    color,
                }),
                GuiDraw::Text(owned_section) => {
                    let rect = node.get_draw_rect();
                    let mut section = owned_section.to_borrowed();
                    section.screen_position =
                        text_screen_position(rect, section.layout).as_vec2().into();
                    section.bounds = rect.size.as_vec2().into();
                    self.glyph_brush.queue(section);
                }
            }
        }
        let changed_textures = self.rect_cache.process_frame_cache(frame_cache);
        for texture in changed_textures {
            let vertices = self
                .rect_cache
                .get(&texture)
                .iter()
                .map(|rect| renderer.make_rect_vertex(rect.clone()))
                .collect();
            renderer.set_vertices(context, texture, vertices, gui.viewport.size);
        }

        // Process queued text.
        let mut brush_action;
        loop {
            brush_action = self.glyph_brush.process_queued(
                |region, tex_data| {
                    renderer.update_glyph_texture(context, glyph_rect_to_gui_rect(region), tex_data)
                },
                |glyph| renderer.make_glyph_vertex(glyph),
            );
            // If the cache texture is too small to fit all the glyphs, resize and try again.
            match brush_action {
                Ok(_) => break,
                Err(BrushError::TextureTooSmall { suggested, .. }) => {
                    let (new_width, new_height) = suggested;
                    eprintln!("Resizing glyph texture -> {}x{}", new_width, new_height);
                    self.glyph_texture =
                        renderer.resize_glyph_texture(context, Size::new(new_width, new_height));
                    self.glyph_brush.resize_texture(new_width, new_height);
                }
            }
        }
        // If the text has changed from what was last drawn, upload the new vertices to GPU.
        match brush_action.unwrap() {
            BrushAction::Draw(vertices) => {
                renderer.set_vertices(context, self.glyph_texture, vertices, gui.viewport.size)
            }
            BrushAction::ReDraw => {}
        }
    }
}
