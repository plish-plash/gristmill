use std::{any::Any, borrow::Cow, cell::RefCell, hash::Hash, marker::PhantomData, rc::Rc};

use emath::{pos2, vec2, Align2, Pos2, Rect, Vec2};
use serde::{Deserialize, Serialize};

use crate::{
    color::Color,
    input::{InputEvent, Trigger},
    style::{style, style_or},
    text::{Font, Text, TextBrush},
    Batch,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GuiLayer {
    Background,
    ContentBackground,
    ContentForeground,
    Foreground,
    SuperForeground,
}

pub trait DrawPrimitive: 'static {
    fn from_text(text: Text<'static>) -> Self;
    fn from_button(rect: Rect, state: ButtonState) -> Self;
}

#[derive(Default, Clone, Copy)]
pub struct LayoutInfo {
    pub size: Vec2,
    pub grow: bool,
}

impl LayoutInfo {
    pub fn with_size(size: Vec2) -> Self {
        LayoutInfo { size, grow: false }
    }
    pub fn grow() -> Self {
        LayoutInfo {
            size: Vec2::ZERO,
            grow: true,
        }
    }
    pub fn from_style(class: &str) -> Self {
        LayoutInfo {
            size: style(class, "size"),
            grow: style(class, "grow"),
        }
    }
}

pub struct WidgetLayout<T> {
    pub widget: WidgetHandle<T>,
    pub rect: Rect,
}

pub type WidgetLayouts<T> = Batch<WidgetLayout<T>>;

pub enum GuiMouseButton {
    Primary,
    Secondary,
}

pub enum GuiInputEvent {
    MouseMotion {
        position: Pos2,
    },
    MouseButton {
        button: GuiMouseButton,
        pressed: bool,
    },
}

impl GuiInputEvent {
    pub fn from_input<Key, MouseButton, F>(
        event: &InputEvent<Key, MouseButton>,
        f: F,
    ) -> Option<GuiInputEvent>
    where
        MouseButton: Copy,
        F: Fn(MouseButton) -> Option<GuiMouseButton>,
    {
        match event {
            InputEvent::MouseMotion { position } => Some(GuiInputEvent::MouseMotion {
                position: *position,
            }),
            InputEvent::MouseButton { button, pressed } => {
                f(*button).map(|button| GuiInputEvent::MouseButton {
                    button,
                    pressed: *pressed,
                })
            }
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct GuiInput {
    pub pointer: Pos2,
    pub primary: Trigger,
    pub secondary: Trigger,
}

impl GuiInput {
    pub fn new() -> Self {
        GuiInput::default()
    }
    pub fn process(&mut self, event: GuiInputEvent) {
        match event {
            GuiInputEvent::MouseMotion { position } => self.pointer = position,
            GuiInputEvent::MouseButton { button, pressed } => match button {
                GuiMouseButton::Primary => self.primary.set_pressed(pressed),
                GuiMouseButton::Secondary => self.secondary.set_pressed(pressed),
            },
        }
    }
    fn update(&mut self) {
        self.primary.update();
        self.secondary.update();
    }
}

pub struct WidgetEvent {
    pub name: Cow<'static, str>,
    pub payload: Option<Rc<dyn Any>>,
}

pub enum WidgetInput {
    Pass,
    Block,
    Event(WidgetEvent),
}

pub trait Widget: 'static {
    type DrawPrimitive;
    fn layout_info(&self) -> LayoutInfo;
    #[allow(unused)]
    fn layout_children(&self, layouts: &mut WidgetLayouts<Self::DrawPrimitive>, rect: Rect) {}
    fn reset_input(&mut self) {}
    #[allow(unused)]
    fn handle_input(&mut self, rect: Rect, input: &GuiInput) -> WidgetInput {
        WidgetInput::Pass
    }
    fn draw(&self, batch: &mut Batch<Self::DrawPrimitive>, rect: Rect);
}

type WidgetHandle<T> = Rc<RefCell<dyn Widget<DrawPrimitive = T>>>;

pub struct WidgetRef<T: ?Sized>(Rc<RefCell<T>>);

impl<T: Widget> WidgetRef<T> {
    pub fn new(widget: T) -> Self {
        WidgetRef(Rc::new(RefCell::new(widget)))
    }
}
impl<T: ?Sized> WidgetRef<T> {
    pub fn borrow(&self) -> std::cell::Ref<T> {
        self.0.borrow()
    }
    pub fn borrow_mut(&self) -> std::cell::RefMut<T> {
        self.0.borrow_mut()
    }
}
impl<T: ?Sized> Clone for WidgetRef<T> {
    fn clone(&self) -> Self {
        WidgetRef(self.0.clone())
    }
}

pub struct Gui<Primitive> {
    layouts: WidgetLayouts<Primitive>,
    batch: Batch<Primitive>,
}

impl<Primitive: 'static> Gui<Primitive> {
    pub fn new() -> Self {
        Gui {
            layouts: WidgetLayouts::new(),
            batch: Batch::new(),
        }
    }
    pub fn clear(&mut self) {
        self.layouts.clear();
        self.batch.clear();
    }
    pub fn layout<W>(&mut self, root: &W, rect: Rect)
    where
        W: Widget<DrawPrimitive = Primitive>,
    {
        self.clear();
        root.layout_children(&mut self.layouts, rect);
    }
    pub fn handle_input(&mut self, input: &mut GuiInput) -> Option<WidgetEvent> {
        let mut widget_event = None;
        let mut blocked = false;
        for item in self.layouts.0.iter().rev() {
            let mut widget = item.widget.borrow_mut();
            if !blocked && item.rect.contains(input.pointer) {
                match widget.handle_input(item.rect, input) {
                    WidgetInput::Pass => {}
                    WidgetInput::Block => blocked = true,
                    WidgetInput::Event(event) => {
                        blocked = true;
                        widget_event = Some(event);
                    }
                }
            } else {
                widget.reset_input();
            }
        }
        input.update();
        widget_event
    }
    pub fn draw(&mut self) -> &[Primitive] {
        self.batch.clear();
        for item in self.layouts.as_slice() {
            item.widget.borrow().draw(&mut self.batch, item.rect);
        }
        self.batch.as_slice()
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PaddingDe {
    all: Option<f32>,
    left: Option<f32>,
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
    between: Option<f32>,
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
#[serde(from = "PaddingDe")]
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
impl From<PaddingDe> for Padding {
    fn from(value: PaddingDe) -> Self {
        let all = value.all.unwrap_or(0.0);
        Padding {
            left: value.left.unwrap_or(all),
            top: value.top.unwrap_or(all),
            right: value.right.unwrap_or(all),
            bottom: value.bottom.unwrap_or(all),
            between: value.between.unwrap_or(all),
        }
    }
}

pub enum ContainerItem<T> {
    Empty(LayoutInfo),
    Widget(WidgetHandle<T>),
}

impl<T: 'static> ContainerItem<T> {
    pub fn from_widget<W>(widget: &WidgetRef<W>) -> Self
    where
        W: Widget<DrawPrimitive = T>,
    {
        ContainerItem::Widget(widget.0.clone())
    }
    pub fn layout_info(&self) -> LayoutInfo {
        match self {
            ContainerItem::Empty(info) => *info,
            ContainerItem::Widget(widget) => widget.borrow().layout_info(),
        }
    }
}

pub struct Container<T> {
    layout: LayoutInfo,
    direction: Direction,
    cross_axis: CrossAxis,
    padding: Padding,
    size: Vec2,
    items: Vec<ContainerItem<T>>,
}

impl<T: 'static> Container<T> {
    pub fn new(direction: Direction, cross_axis: CrossAxis, class: &str) -> Self {
        Container {
            layout: LayoutInfo::from_style(class),
            direction,
            cross_axis,
            padding: style(class, "padding"),
            size: Vec2::ZERO,
            items: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    fn add(&mut self, item: ContainerItem<T>) {
        let between = if self.items.is_empty() {
            0.0
        } else {
            self.padding.between
        };
        let size = item.layout_info().size;
        match self.direction {
            Direction::Horizontal => {
                self.size.x += size.x + between;
                self.size.y = self.size.y.max(size.y);
            }
            Direction::Vertical => {
                self.size.x = self.size.x.max(size.x);
                self.size.y += size.y + between;
            }
        }
        self.items.push(item);
    }
    pub fn add_empty(&mut self, empty: LayoutInfo) {
        self.add(ContainerItem::Empty(empty))
    }
    pub fn add_widget<W>(&mut self, widget: W) -> WidgetRef<W>
    where
        W: Widget<DrawPrimitive = T>,
    {
        let widget = WidgetRef::new(widget);
        self.add(ContainerItem::from_widget(&widget));
        widget
    }
    pub fn add_widget_ref<W>(&mut self, widget: &WidgetRef<W>)
    where
        W: Widget<DrawPrimitive = T>,
    {
        self.add(ContainerItem::from_widget(widget));
    }
}
impl<T> Default for Container<T> {
    fn default() -> Self {
        Container {
            layout: LayoutInfo::default(),
            direction: Direction::Horizontal,
            cross_axis: CrossAxis::Stretch,
            padding: Padding::default(),
            size: Vec2::ZERO,
            items: Vec::new(),
        }
    }
}
impl<T: 'static> Widget for Container<T> {
    type DrawPrimitive = T;
    fn layout_info(&self) -> LayoutInfo {
        let layout_size = self.size + self.padding.min_size();
        LayoutInfo {
            size: self.layout.size.max(layout_size),
            grow: self.layout.grow,
        }
    }
    fn layout_children(&self, layouts: &mut WidgetLayouts<T>, mut rect: Rect) {
        if self.items.is_empty() {
            return;
        }
        rect.min += vec2(self.padding.left, self.padding.top);
        rect.max -= vec2(self.padding.right, self.padding.bottom);
        let main_size = self.direction.main(rect.size());
        let cross_size = self.direction.cross(rect.size());
        let mut main_size_reserved = self.padding.between * ((self.items.len() - 1) as f32);
        let mut grow_items = 0;
        for layout in self.items.iter().map(|item| item.layout_info()) {
            if layout.grow {
                grow_items += 1;
            } else {
                main_size_reserved += self.direction.main(layout.size);
            }
        }
        let grow_size = if grow_items > 0 {
            (main_size - main_size_reserved) / (grow_items as f32)
        } else {
            0.0
        };
        let mut main_pos = 0.0;
        for item in self.items.iter() {
            let layout = item.layout_info();
            let item_main = if layout.grow {
                grow_size
            } else {
                self.direction.main(layout.size)
            };
            let (cross_pos, item_cross) = self
                .cross_axis
                .layout(cross_size, self.direction.cross(layout.size));
            if let ContainerItem::Widget(widget) = item {
                let mut widget_rect = self
                    .direction
                    .rectangle(main_pos, cross_pos, item_main, item_cross);
                widget_rect = widget_rect.translate(rect.min.to_vec2());
                layouts.add(WidgetLayout {
                    widget: widget.clone(),
                    rect: widget_rect,
                });
                widget.borrow().layout_children(layouts, widget_rect);
            }
            main_pos += item_main + self.padding.between;
        }
    }
    fn draw(&self, _batch: &mut Batch<T>, _rect: Rect) {}
}

pub struct GridContainer<T> {
    layout: LayoutInfo,
    padding: Padding,
    item_size: Vec2,
    items: Vec<ContainerItem<T>>,
}

impl<T: 'static> GridContainer<T> {
    pub fn new(class: &str) -> Self {
        GridContainer {
            layout: LayoutInfo::from_style(class),
            padding: style(class, "padding"),
            item_size: Vec2::ZERO,
            items: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    fn add(&mut self, item: ContainerItem<T>) {
        self.item_size = self.item_size.max(item.layout_info().size);
        self.items.push(item);
    }
    pub fn add_empty(&mut self, empty: LayoutInfo) {
        self.add(ContainerItem::Empty(empty))
    }
    pub fn add_widget<W>(&mut self, widget: W) -> WidgetRef<W>
    where
        W: Widget<DrawPrimitive = T>,
    {
        let widget = WidgetRef::new(widget);
        self.add(ContainerItem::from_widget(&widget));
        widget
    }
    pub fn add_widget_ref<W>(&mut self, widget: &WidgetRef<W>)
    where
        W: Widget<DrawPrimitive = T>,
    {
        self.add(ContainerItem::from_widget(widget));
    }
}
impl<T> Default for GridContainer<T> {
    fn default() -> Self {
        GridContainer {
            layout: LayoutInfo::default(),
            padding: Padding::default(),
            item_size: Vec2::ZERO,
            items: Vec::new(),
        }
    }
}
impl<T: 'static> Widget for GridContainer<T> {
    type DrawPrimitive = T;
    fn layout_info(&self) -> LayoutInfo {
        LayoutInfo {
            size: self.layout.size,
            grow: self.layout.grow,
        }
    }
    fn layout_children(&self, layouts: &mut WidgetLayouts<T>, mut rect: Rect) {
        if self.items.is_empty() {
            return;
        }
        rect.min += vec2(self.padding.left, self.padding.top);
        rect.max -= vec2(self.padding.right, self.padding.bottom);
        let mut pos = Pos2::ZERO;
        for item in self.items.iter() {
            if let ContainerItem::Widget(widget) = item {
                let widget_rect =
                    Rect::from_min_size(pos, self.item_size).translate(rect.min.to_vec2());
                layouts.add(WidgetLayout {
                    widget: widget.clone(),
                    rect: widget_rect,
                });
                widget.borrow().layout_children(layouts, widget_rect);
            }
            pos.x += self.item_size.x + self.padding.between;
            if pos.x + self.item_size.x > rect.width() {
                pos.x = 0.0;
                pos.y += self.item_size.y + self.padding.between;
            }
        }
    }
    fn draw(&self, _batch: &mut Batch<T>, _rect: Rect) {}
}

pub struct Label<T> {
    layout: LayoutInfo,
    font: Font,
    color: Color,
    text: Cow<'static, str>,
    align: Align2,
    wrap: bool,
    _marker: PhantomData<T>,
}

impl<T> Label<T> {
    pub fn new<S: Into<Cow<'static, str>>>(class: &str, text: S) -> Self {
        Label {
            layout: LayoutInfo::from_style(class),
            font: Font::from_style(class),
            color: Color::WHITE,
            text: text.into(),
            align: style_or(class, "align", Align2::LEFT_CENTER),
            wrap: style(class, "wrap"),
            _marker: PhantomData,
        }
    }
    pub fn color(&self) -> Color {
        self.color
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn set_text<S: Into<Cow<'static, str>>>(&mut self, text: S) {
        self.text = text.into();
    }
    pub fn autosize<L>(&mut self, text_brush: &mut TextBrush<L>, layer: L)
    where
        L: Clone + Ord + PartialEq + Hash + 'static,
    {
        let size = text_brush.text_bounds(
            layer,
            self.align,
            if self.wrap {
                Some(self.layout.size.x)
            } else {
                None
            },
            self.font,
            &self.text,
        );
        if let Some(size) = size {
            if !self.wrap {
                self.layout.size.x = size.width();
            }
            self.layout.size.y = size.height();
        }
    }
}
impl<T: DrawPrimitive> Widget for Label<T> {
    type DrawPrimitive = T;
    fn layout_info(&self) -> LayoutInfo {
        self.layout
    }
    fn draw(&self, batch: &mut Batch<T>, rect: Rect) {
        batch.add(T::from_text(Text {
            position: self.align.pos_in_rect(&rect),
            align: self.align,
            wrap: if self.wrap { Some(rect.width()) } else { None },
            font: self.font,
            color: self.color,
            text: self.text.clone(),
        }));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hover,
    Press,
    Disable,
}

pub struct Button<T> {
    name: Cow<'static, str>,
    layout: LayoutInfo,
    label: Label<T>,
    state: ButtonState,
    event_payload: Option<Rc<dyn Any>>,
}

impl<T> Button<T> {
    pub fn new<S: Into<Cow<'static, str>>, L: Into<Cow<'static, str>>>(
        class: &str,
        name: S,
        label: L,
    ) -> Self {
        Button {
            name: name.into(),
            layout: LayoutInfo::from_style(class),
            label: Label::new(class, label),
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
    pub fn set_event_payload<P: 'static>(&mut self, payload: P) {
        self.event_payload = Some(Rc::new(payload));
    }
}
impl<T: DrawPrimitive> Widget for Button<T> {
    type DrawPrimitive = T;
    fn layout_info(&self) -> LayoutInfo {
        self.layout
    }
    fn reset_input(&mut self) {
        if self.state != ButtonState::Disable {
            self.state = ButtonState::Normal;
        }
    }
    fn handle_input(&mut self, _rect: Rect, input: &GuiInput) -> WidgetInput {
        if self.state != ButtonState::Disable {
            self.state = if input.primary.pressed() {
                ButtonState::Press
            } else {
                ButtonState::Hover
            };
            if input.primary.just_pressed() {
                return WidgetInput::Event(WidgetEvent {
                    name: self.name.clone(),
                    payload: self.event_payload.clone(),
                });
            }
        }
        WidgetInput::Block
    }
    fn draw(&self, batch: &mut Batch<T>, rect: Rect) {
        batch.add(T::from_button(rect, self.state));
        batch.add(T::from_text(Text {
            position: rect.center(),
            align: Align2::CENTER_CENTER,
            wrap: None,
            font: self.label.font,
            color: Color::BLACK,
            text: self.label.text.clone(),
        }));
    }
}
