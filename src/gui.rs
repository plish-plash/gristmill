use std::{any::Any, borrow::Cow, cell::RefCell, rc::Rc};

use emath::{pos2, vec2, Align2, Pos2, Rect, RectTransform, Vec2};

use crate::{
    color::{Color, Palette},
    input::{InputEvent, MouseButton},
    render2d::QuadDrawQueue,
    sprite::ColorRect,
    text::{Font, Text, TextDrawQueue},
};

pub trait GuiRenderer {
    fn quads(&mut self) -> &mut QuadDrawQueue;
    fn text(&mut self) -> &mut TextDrawQueue;
}

pub trait Widget: 'static {
    fn default_size(&self) -> Vec2;
    fn children(&self) -> Option<&Container> {
        None
    }
    fn reset_input(&mut self) {}
    #[allow(unused)]
    fn handle_input(&mut self, rect: Rect, input: &GuiInputFrame) -> Option<GuiEvent> {
        None
    }
    fn draw(&self, renderer: &mut dyn GuiRenderer, rect: Rect);
}

type WidgetRc = Rc<RefCell<dyn Widget>>;

pub struct WidgetRef<T>(Rc<RefCell<T>>);

impl<T> WidgetRef<T> {
    pub fn borrow(&self) -> std::cell::Ref<T> {
        self.0.borrow()
    }
    pub fn borrow_mut(&self) -> std::cell::RefMut<T> {
        self.0.borrow_mut()
    }
}

impl<T: Widget> WidgetRef<T> {
    pub fn with_default_size(&self) -> ContainerItem {
        let size = self.borrow().default_size();
        ContainerItem {
            size,
            grow: false,
            widget: Some(self.0.clone()),
        }
    }
    pub fn grow(&self) -> ContainerItem {
        let mut item = self.with_default_size();
        item.grow = true;
        item
    }
    pub fn with_size(&self, size: Vec2) -> ContainerItem {
        let mut item = self.with_default_size();
        item.size = size;
        item
    }
}

impl<T> Clone for WidgetRef<T> {
    fn clone(&self) -> Self {
        WidgetRef(self.0.clone())
    }
}

impl<T: Widget> From<T> for WidgetRef<T> {
    fn from(value: T) -> Self {
        WidgetRef(Rc::new(RefCell::new(value)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl Direction {
    fn main(self, size: Vec2) -> f32 {
        match self {
            Direction::Horizontal => size.x,
            Direction::Vertical => size.y,
        }
    }
    fn cross(self, size: Vec2) -> f32 {
        match self {
            Direction::Horizontal => size.y,
            Direction::Vertical => size.x,
        }
    }
    fn rectangle(self, main_pos: f32, cross_pos: f32, main_size: f32, cross_size: f32) -> Rect {
        match self {
            Direction::Horizontal => {
                Rect::from_min_size(pos2(main_pos, cross_pos), vec2(main_size, cross_size))
            }
            Direction::Vertical => {
                Rect::from_min_size(pos2(cross_pos, main_pos), vec2(cross_size, main_size))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossAxis {
    Start,
    End,
    Center,
    Stretch,
}

impl CrossAxis {
    fn layout(self, container: f32, item: f32) -> (f32, f32) {
        match self {
            CrossAxis::Start => (0.0, item),
            CrossAxis::End => (container - item, item),
            CrossAxis::Center => ((container - item) / 2.0, item),
            CrossAxis::Stretch => (0.0, container),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Padding {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub between: f32,
}

impl Padding {
    pub fn all(padding: f32) -> Self {
        Padding {
            left: padding,
            top: padding,
            right: padding,
            bottom: padding,
            between: padding,
        }
    }
    pub fn between(padding: f32) -> Self {
        Padding {
            between: padding,
            ..Default::default()
        }
    }
    fn min_size(&self) -> Vec2 {
        vec2(self.left + self.right, self.bottom + self.top)
    }
}

pub struct ContainerItem {
    size: Vec2,
    grow: bool,
    widget: Option<WidgetRc>,
}

impl ContainerItem {
    pub fn empty(size: Vec2) -> Self {
        ContainerItem {
            size,
            grow: false,
            widget: None,
        }
    }
    pub fn empty_grow() -> Self {
        ContainerItem {
            size: Vec2::ZERO,
            grow: true,
            widget: None,
        }
    }
}

pub struct Container {
    direction: Direction,
    cross_axis: CrossAxis,
    padding: Padding,
    size: Vec2,
    items: Vec<ContainerItem>,
}

impl Container {
    pub fn new(direction: Direction, cross_axis: CrossAxis, padding: Padding) -> Self {
        Container {
            direction,
            cross_axis,
            padding,
            size: Vec2::ZERO,
            items: Vec::new(),
        }
    }
    pub fn with_items(
        direction: Direction,
        cross_axis: CrossAxis,
        padding: Padding,
        items: Vec<ContainerItem>,
    ) -> Self {
        let mut size = Vec2::ZERO;
        let mut between = 0.0;
        for item in items.iter() {
            match direction {
                Direction::Horizontal => {
                    size.x += item.size.x + between;
                    size.y = size.y.max(item.size.y);
                }
                Direction::Vertical => {
                    size.x = size.x.max(item.size.x);
                    size.y += item.size.y + between;
                }
            }
            between = padding.between;
        }
        Container {
            direction,
            cross_axis,
            padding,
            size,
            items,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn add(&mut self, item: ContainerItem) {
        let between = if self.items.is_empty() {
            0.0
        } else {
            self.padding.between
        };
        match self.direction {
            Direction::Horizontal => {
                self.size.x += item.size.x + between;
                self.size.y = self.size.y.max(item.size.y);
            }
            Direction::Vertical => {
                self.size.x = self.size.x.max(item.size.x);
                self.size.y += item.size.y + between;
            }
        }
        self.items.push(item);
    }
    pub fn add_widget<T: Widget>(&mut self, widget: T) {
        self.add(WidgetRef::from(widget).with_default_size());
    }
    pub fn add_widget_grow<T: Widget>(&mut self, widget: T) {
        self.add(WidgetRef::from(widget).grow());
    }
    pub fn add_widget_with_size<T: Widget>(&mut self, widget: T, size: Vec2) {
        self.add(WidgetRef::from(widget).with_size(size));
    }

    fn layout(&self, mut rect: Rect, widget_layouts: &mut Vec<WidgetLayout>) {
        if self.items.is_empty() {
            return;
        }
        rect.min += vec2(self.padding.left, self.padding.top);
        rect.max -= vec2(self.padding.right, self.padding.bottom);
        let main_size = self.direction.main(rect.size());
        let cross_size = self.direction.cross(rect.size());
        let mut main_size_reserved = self.padding.between * ((self.items.len() - 1) as f32);
        let mut grow_items = 0;
        for item in self.items.iter() {
            if item.grow {
                grow_items += 1;
            } else {
                main_size_reserved += self.direction.main(item.size);
            }
        }
        let grow_size = if grow_items > 0 {
            (main_size - main_size_reserved) / (grow_items as f32)
        } else {
            0.0
        };
        let mut main_pos = 0.0;
        for item in self.items.iter() {
            let item_main = if item.grow {
                grow_size
            } else {
                self.direction.main(item.size)
            };
            let (cross_pos, item_cross) = self
                .cross_axis
                .layout(cross_size, self.direction.cross(item.size));
            if let Some(widget) = item.widget.as_ref() {
                let mut widget_rect = self
                    .direction
                    .rectangle(main_pos, cross_pos, item_main, item_cross);
                widget_rect = widget_rect.translate(rect.min.to_vec2());
                widget_layouts.push(WidgetLayout {
                    widget: widget.clone(),
                    rect: widget_rect,
                });
                if let Some(children) = widget.borrow().children() {
                    children.layout(widget_rect, widget_layouts);
                }
            }
            main_pos += item_main + self.padding.between;
        }
    }
}

impl Default for Container {
    fn default() -> Self {
        Container {
            direction: Direction::Horizontal,
            cross_axis: CrossAxis::Stretch,
            padding: Default::default(),
            size: Vec2::ZERO,
            items: Vec::new(),
        }
    }
}

impl Widget for Container {
    fn default_size(&self) -> Vec2 {
        self.size + self.padding.min_size()
    }
    fn children(&self) -> Option<&Container> {
        Some(self)
    }
    fn draw(&self, _renderer: &mut dyn GuiRenderer, _rect: Rect) {}
}

#[derive(Default)]
pub struct GuiInput {
    pointer: Pos2,
    pressed: bool,
    just_pressed: bool,
}

pub struct GuiInputFrame {
    pub pointer: Pos2,
    pub pressed: bool,
    pub just_pressed: bool,
}

impl GuiInput {
    pub fn event<Key>(&mut self, event: InputEvent<Key>) {
        match event {
            InputEvent::MouseMotion { position } => self.pointer = position,
            InputEvent::MouseButton { button, pressed } => {
                if button == MouseButton::Left {
                    self.just_pressed = pressed && !self.pressed;
                    self.pressed = pressed;
                }
            }
            _ => (),
        }
    }
    pub fn finish(&mut self, screen_transform: &RectTransform) -> GuiInputFrame {
        let pointer = screen_transform.inverse().transform_pos(self.pointer);
        let just_pressed = self.just_pressed;
        self.just_pressed = false;
        GuiInputFrame {
            pointer,
            pressed: self.pressed,
            just_pressed,
        }
    }
}

pub struct GuiEvent {
    pub name: Cow<'static, str>,
    pub payload: Option<Rc<dyn Any>>,
}

pub struct WidgetLayout {
    widget: WidgetRc,
    rect: Rect,
}

pub struct ContainerLayout(Rect, Vec<WidgetLayout>);

impl ContainerLayout {
    pub fn new() -> Self {
        ContainerLayout(Rect::ZERO, Vec::new())
    }
    pub fn mark_dirty(&mut self) {
        self.0 = Rect::ZERO;
    }
    pub fn layout(&mut self, container: &Container, rect: Rect) {
        if self.0 != rect {
            self.0 = rect;
            self.1.clear();
            container.layout(rect, &mut self.1);
        }
    }
    pub fn handle_input(&mut self, input: &GuiInputFrame) -> Option<GuiEvent> {
        for item in self.1.iter_mut().rev() {
            let mut widget = item.widget.borrow_mut();
            widget.reset_input();
            if item.rect.contains(input.pointer) {
                if let Some(event) = widget.handle_input(item.rect, &input) {
                    return Some(event);
                }
            }
        }
        None
    }
    pub fn draw(&self, renderer: &mut dyn GuiRenderer) {
        for item in self.1.iter() {
            item.widget.borrow().draw(renderer, item.rect);
        }
    }
}

pub struct Label {
    pub font: Font,
    pub text: Cow<'static, str>,
    pub align: Align2,
}

impl Label {
    pub fn new<S: Into<Cow<'static, str>>>(text: S) -> Self {
        Label {
            font: Default::default(),
            text: text.into(),
            align: Align2::LEFT_CENTER,
        }
    }
    pub fn with_font<S: Into<Cow<'static, str>>>(font: Font, text: S) -> Self {
        Label {
            font,
            text: text.into(),
            align: Align2::LEFT_CENTER,
        }
    }
}

impl Widget for Label {
    fn default_size(&self) -> Vec2 {
        vec2(128.0, 32.0)
    }
    fn handle_input(&mut self, _rect: Rect, _input: &GuiInputFrame) -> Option<GuiEvent> {
        None
    }
    fn draw(&self, renderer: &mut dyn GuiRenderer, rect: Rect) {
        renderer.text().queue(&Text {
            position: self.align.pos_in_rect(&rect),
            align: self.align,
            wrap: None,
            font: self.font,
            color: Color::WHITE,
            text: self.text.as_ref().into(),
        });
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ButtonState {
    Normal,
    Hover,
    Press,
    Disable,
}

#[derive(Clone)]
pub struct ButtonPalette {
    pub normal: Color,
    pub hover: Color,
    pub press: Color,
    pub disable: Color,
}

impl ButtonPalette {
    pub fn new(palette: &Palette) -> Self {
        ButtonPalette {
            normal: Color::from_palette(palette, "button_normal"),
            hover: Color::from_palette(palette, "button_hover"),
            press: Color::from_palette(palette, "button_press"),
            disable: Color::from_palette(palette, "button_disable"),
        }
    }
    fn state_color(&self, state: ButtonState) -> Color {
        match state {
            ButtonState::Normal => self.normal,
            ButtonState::Hover => self.hover,
            ButtonState::Press => self.press,
            ButtonState::Disable => self.disable,
        }
    }
}

pub struct Button {
    name: Cow<'static, str>,
    palette: ButtonPalette,
    label: Label,
    state: ButtonState,
    event_payload: Option<Rc<dyn Any>>,
}

impl Button {
    pub fn new<S: Into<Cow<'static, str>>>(name: S, palette: ButtonPalette, label: Label) -> Self {
        Button {
            name: name.into(),
            palette,
            label,
            state: ButtonState::Normal,
            event_payload: None,
        }
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.state = ButtonState::Normal;
        } else {
            self.state = ButtonState::Disable;
        }
    }
    pub fn set_event_payload<T: 'static>(&mut self, payload: T) {
        self.event_payload = Some(Rc::new(payload));
    }
}

impl Widget for Button {
    fn default_size(&self) -> Vec2 {
        Vec2::new(128.0, 32.0)
    }
    fn reset_input(&mut self) {
        if self.state != ButtonState::Disable {
            self.state = ButtonState::Normal;
        }
    }
    fn handle_input(&mut self, _rect: Rect, input: &GuiInputFrame) -> Option<GuiEvent> {
        if self.state != ButtonState::Disable {
            if input.just_pressed {
                self.state = ButtonState::Press;
                return Some(GuiEvent {
                    name: self.name.clone(),
                    payload: self.event_payload.clone(),
                });
            } else {
                self.state = ButtonState::Hover;
            }
        }
        None
    }
    fn draw(&self, renderer: &mut dyn GuiRenderer, rect: Rect) {
        renderer
            .quads()
            .queue(&ColorRect(self.palette.state_color(self.state), rect));
        renderer.text().queue(&Text {
            position: rect.center(),
            align: Align2::CENTER_CENTER,
            wrap: None,
            font: self.label.font,
            color: Color::BLACK,
            text: self.label.text.as_ref().into(),
        });
    }
}
