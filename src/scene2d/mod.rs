pub mod sprite;

use emath::{Pos2, Rect, TSTransform, Vec2};

use crate::{color::Color, Size};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq)]
pub struct UvRect(Rect);

impl UvRect {
    pub fn from_region(region: Rect, size: Size) -> Self {
        let size = size.to_vec2();
        UvRect(Rect {
            min: Pos2 {
                x: region.min.x / size.x,
                y: region.min.y / size.y,
            },
            max: Pos2 {
                x: region.max.x / size.x,
                y: region.max.y / size.y,
            },
        })
    }
    pub fn from_frame(frame: usize, frame_size: Vec2, size: Size) -> Self {
        let region = Rect::from_min_size(Pos2::new(frame as f32 * frame_size.x, 0.0), frame_size);
        Self::from_region(region, size)
    }
}
impl Default for UvRect {
    fn default() -> Self {
        UvRect(Rect {
            min: Pos2::ZERO,
            max: Pos2::new(1.0, 1.0),
        })
    }
}
impl From<Rect> for UvRect {
    fn from(value: Rect) -> Self {
        UvRect(value)
    }
}

#[repr(C)]
#[derive(Clone, PartialEq)]
pub struct Instance {
    pub rect: Rect,
    pub uv: UvRect,
    pub color: Color,
}

impl From<Rect> for Instance {
    fn from(value: Rect) -> Self {
        Instance {
            rect: value,
            uv: UvRect::default(),
            color: Color::WHITE,
        }
    }
}

// #[derive(Clone, Copy)]
// pub struct Camera {
//     pub translate: Vec2,
//     pub scale: Vec2,
//     pub clip: Option<Rect>,
// }

// impl Camera {
//     pub fn from_viewport(viewport: Rect) -> Self {
//         let translate = -viewport.min.to_vec2().round();
//         let scale = Vec2::new(2.0 / viewport.width(), 2.0 / viewport.height());
//         Camera {
//             translate,
//             scale,
//             clip: None,
//         }
//     }
//     pub fn from_translate_scale(translate: Vec2, scale: f32) -> Self {
//         Camera {
//             translate,
//             scale: Vec2::splat(scale),
//             clip: None,
//         }
//     }
//     pub fn with_scroll(mut self, offset: Vec2, clip: Rect) -> Self {
//         self.translate += offset;
//         let clip_offset = self.clip.map(|rect| rect.min).unwrap_or_default();
//         self.clip = Some(clip.translate(clip_offset.to_vec2()));
//         self
//     }
// }

#[derive(Clone)]
pub struct ScrollCamera {
    pub screen_size: Vec2,
    pub center: Pos2,
    pub scale: f32,
}

impl ScrollCamera {
    pub fn constrain(&mut self, rect: Rect) {
        let screen_extent = (self.screen_size / 2.0) / self.scale;
        if rect.width() < screen_extent.x * 2.0 {
            self.center.x = rect.center().x;
        } else if self.center.x - screen_extent.x < rect.min.x {
            self.center.x = rect.min.x + screen_extent.x;
        } else if self.center.x + screen_extent.x > rect.max.x {
            self.center.x = rect.max.x - screen_extent.x;
        }
        if rect.height() < screen_extent.y * 2.0 {
            self.center.y = rect.center().y;
        } else if self.center.y - screen_extent.y < rect.min.y {
            self.center.y = rect.min.y + screen_extent.y;
        } else if self.center.y + screen_extent.y > rect.max.y {
            self.center.y = rect.max.y - screen_extent.y;
        }
    }
    pub fn viewport(&self) -> Rect {
        Rect::from_center_size(self.center, self.screen_size / self.scale)
    }
    pub fn transform(&self) -> TSTransform {
        let screen_center = self.screen_size / 2.0;
        TSTransform::new(self.center.to_vec2() - screen_center, self.scale).inverse()
    }
    // pub fn camera(&self) -> Camera {
    //     Camera::from_viewport(self.viewport())
    // }
    // pub fn camera_view(&self, view: Rect) -> Camera {
    //     let center_offset = view.center() - (self.screen_size / 2.0).to_pos2();
    //     let mut camera = Camera::from_viewport(Rect::from_center_size(
    //         self.center - (center_offset / self.scale),
    //         self.screen_size / self.scale,
    //     ));
    //     camera.clip = Some(view);
    //     camera
    // }
    pub fn screen_to_world(&self, pos: Pos2) -> Pos2 {
        let viewport = self.viewport();
        let x = emath::remap(
            pos.x,
            0.0..=self.screen_size.x,
            viewport.min.x..=viewport.max.x,
        );
        let y = emath::remap(
            pos.y,
            0.0..=self.screen_size.y,
            viewport.min.y..=viewport.max.y,
        );
        Pos2::new(x, y)
    }
}
