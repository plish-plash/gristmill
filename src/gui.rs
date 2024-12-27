use std::{
    any::Any, borrow::Cow, cell::RefCell, collections::BTreeMap, marker::PhantomData, rc::Rc,
};

use emath::{pos2, vec2, Align2, Pos2, Rect, Vec2};
use serde::{Deserialize, Serialize};

use crate::{
    color::Color,
    input::{InputEvent, Trigger},
    style::{style, style_or},
    text::{Font, Text},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GuiLayer {
    Panel,
    Background,
    Content,
    Foreground,
}

pub trait DrawPrimitive: 'static {
    fn from_text(text: Text<'static, GuiLayer>) -> Self;
    fn from_button_background(rect: Rect, state: ButtonState) -> Self;
}

pub trait Widget: 'static {
    type DrawPrimitive;
    fn layout(&self) -> LayoutInfo;
    fn children(&self) -> Option<&Container<Self::DrawPrimitive>> {
        None
    }
    fn reset_input(&mut self) {}
    #[allow(unused)]
    fn handle_input(&mut self, rect: Rect, input: &GuiInput) -> Option<WidgetEvent> {
        None
    }
    fn draw(&self, rect: Rect) -> Vec<Self::DrawPrimitive>;
}

type WidgetHandle<T> = Rc<RefCell<dyn Widget<DrawPrimitive = T>>>;

pub struct WidgetRef<T>(Rc<RefCell<T>>);

impl<T: Widget> WidgetRef<T> {
    fn new(widget: T) -> Self {
        WidgetRef(Rc::new(RefCell::new(widget)))
    }
    fn to_handle(&self) -> WidgetHandle<T::DrawPrimitive> {
        self.0.clone()
    }

    pub fn borrow(&self) -> std::cell::Ref<T> {
        self.0.borrow()
    }
    pub fn borrow_mut(&self) -> std::cell::RefMut<T> {
        self.0.borrow_mut()
    }
}
impl<T> Clone for WidgetRef<T> {
    fn clone(&self) -> Self {
        WidgetRef(self.0.clone())
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

#[derive(Default, Clone, Copy)]
pub struct LayoutInfo {
    size: Vec2,
    grow: bool,
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
    pub fn from_style(class: &str, default_size: Vec2) -> Self {
        LayoutInfo {
            size: style_or(class, "size", default_size),
            grow: style(class, "grow"),
        }
    }
}

enum ContainerItem<T> {
    Empty(LayoutInfo),
    Widget(WidgetHandle<T>),
}

impl<T: 'static> ContainerItem<T> {
    fn layout(&self) -> LayoutInfo {
        match self {
            ContainerItem::Empty(layout) => *layout,
            ContainerItem::Widget(widget) => widget.borrow().layout(),
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
            layout: LayoutInfo::from_style(class, Vec2::ZERO),
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
        let size = item.layout().size;
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
        self.add(ContainerItem::Widget(widget.to_handle()));
        widget
    }

    fn layout(&self, mut rect: Rect, widget_layouts: &mut Vec<WidgetLayout<T>>) {
        if self.items.is_empty() {
            return;
        }
        rect.min += vec2(self.padding.left, self.padding.top);
        rect.max -= vec2(self.padding.right, self.padding.bottom);
        let main_size = self.direction.main(rect.size());
        let cross_size = self.direction.cross(rect.size());
        let mut main_size_reserved = self.padding.between * ((self.items.len() - 1) as f32);
        let mut grow_items = 0;
        for layout in self.items.iter().map(|item| item.layout()) {
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
            let layout = item.layout();
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
    fn layout(&self) -> LayoutInfo {
        let layout_size = self.size + self.padding.min_size();
        LayoutInfo {
            size: self.layout.size.max(layout_size),
            grow: self.layout.grow,
        }
    }
    fn children(&self) -> Option<&Container<T>> {
        Some(self)
    }
    fn draw(&self, _rect: Rect) -> Vec<Self::DrawPrimitive> {
        Vec::new()
    }
}

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
        event: InputEvent<Key, MouseButton>,
        f: F,
    ) -> Option<GuiInputEvent>
    where
        F: Fn(MouseButton) -> Option<GuiMouseButton>,
    {
        match event {
            InputEvent::MouseMotion { position } => Some(GuiInputEvent::MouseMotion { position }),
            InputEvent::MouseButton { button, pressed } => {
                f(button).map(|button| GuiInputEvent::MouseButton { button, pressed })
            }
            _ => None,
        }
    }
}

pub struct GuiInput {
    pointer: Pos2,
    primary: Trigger,
    secondary: Trigger,
}

impl GuiInput {
    pub fn new() -> Self {
        GuiInput {
            pointer: Pos2::ZERO,
            primary: Trigger::new(),
            secondary: Trigger::new(),
        }
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

pub struct WidgetLayout<T> {
    widget: WidgetHandle<T>,
    rect: Rect,
}

pub struct Gui<L, T> {
    layouts: BTreeMap<L, Vec<WidgetLayout<T>>>,
}

impl<L: Ord, T: 'static> Gui<L, T> {
    pub fn new() -> Self {
        Gui {
            layouts: BTreeMap::new(),
        }
    }
    pub fn layout(&mut self, layer: L, container: &Container<T>, rect: Rect) {
        let mut layout = self
            .layouts
            .entry(layer)
            .and_modify(|vec| vec.clear())
            .or_default();
        container.layout(rect, &mut layout);
    }
    pub fn handle_input(&mut self, input: &mut GuiInput) -> Option<WidgetEvent> {
        let mut widget_event = None;
        for item in self
            .layouts
            .iter_mut()
            .rev()
            .flat_map(|(_, layout)| layout.iter_mut().rev())
        {
            let mut widget = item.widget.borrow_mut();
            if widget_event.is_none() && item.rect.contains(input.pointer) {
                if let Some(event) = widget.handle_input(item.rect, &input) {
                    widget_event = Some(event);
                }
            } else {
                widget.reset_input();
            }
        }
        input.update();
        widget_event
    }
    pub fn draw(&self, layer: &L) -> Vec<T> {
        let mut primitives = Vec::new();
        if let Some(items) = self.layouts.get(layer) {
            for item in items.iter() {
                primitives.append(&mut item.widget.borrow().draw(item.rect));
            }
        }
        primitives
    }
}

pub struct Label<T> {
    layout: LayoutInfo,
    font: Font,
    text: Cow<'static, str>,
    align: Align2,
    wrap: bool,
    _marker: PhantomData<T>,
}

impl<T> Label<T> {
    pub fn new<S: Into<Cow<'static, str>>>(class: &str, text: S) -> Self {
        Label {
            layout: LayoutInfo::from_style(class, Vec2::ZERO),
            font: Font::new(style(class, "font-id"), style(class, "font-scale")),
            text: text.into(),
            align: style_or(class, "align", Align2::LEFT_CENTER),
            wrap: style(class, "wrap"),
            _marker: PhantomData,
        }
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn set_text<S: Into<Cow<'static, str>>>(&mut self, text: S) {
        self.text = text.into();
    }
}

impl<T: DrawPrimitive> Widget for Label<T> {
    type DrawPrimitive = T;
    fn layout(&self) -> LayoutInfo {
        self.layout
    }
    fn draw(&self, rect: Rect) -> Vec<Self::DrawPrimitive> {
        vec![T::from_text(Text {
            layer: GuiLayer::Content,
            position: self.align.pos_in_rect(&rect),
            align: self.align,
            wrap: if self.wrap { Some(rect.width()) } else { None },
            font: self.font,
            color: Color::WHITE,
            text: self.text.clone(),
        })]
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
        name: S,
        class: &str,
        label: L,
    ) -> Self {
        Button {
            name: name.into(),
            layout: LayoutInfo::from_style(class, vec2(128.0, 32.0)),
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
    fn layout(&self) -> LayoutInfo {
        self.layout
    }
    fn reset_input(&mut self) {
        if self.state != ButtonState::Disable {
            self.state = ButtonState::Normal;
        }
    }
    fn handle_input(&mut self, _rect: Rect, input: &GuiInput) -> Option<WidgetEvent> {
        if self.state != ButtonState::Disable {
            self.state = if input.primary.pressed() {
                ButtonState::Press
            } else {
                ButtonState::Hover
            };
            if input.primary.just_pressed() {
                return Some(WidgetEvent {
                    name: self.name.clone(),
                    payload: self.event_payload.clone(),
                });
            }
        }
        None
    }
    fn draw(&self, rect: Rect) -> Vec<Self::DrawPrimitive> {
        vec![
            T::from_button_background(rect, self.state),
            T::from_text(Text {
                layer: GuiLayer::Content,
                position: rect.center(),
                align: Align2::CENTER_CENTER,
                wrap: None,
                font: self.label.font,
                color: Color::BLACK,
                text: self.label.text.clone(),
            }),
        ]
    }
}
