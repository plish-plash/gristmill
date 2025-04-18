use std::{
    any::Any, borrow::Cow, cell::RefCell, collections::BTreeMap, hash::Hash, marker::PhantomData,
    rc::Rc,
};

use emath::{pos2, vec2, Align2, Pos2, Rect, TSTransform, Vec2};
use serde::{Deserialize, Serialize};

use crate::{
    color::Color,
    input::{InputEvent, Trigger},
    scene2d::sprite::ColorRect,
    style::{style, style_or},
    text::{Font, Text, TextBrush, TextPipeline},
    Batcher, Pipeline,
};

pub trait GuiRenderer: 'static {
    type GuiLayer: Ord;
    type TextLayer: Eq + Hash + Clone;
    type Pipeline: TextPipeline;
    fn text_layer(layer: &Self::GuiLayer, sublayer: usize) -> Self::TextLayer;
    fn button_material(&self, state: ButtonState) -> <Self::Pipeline as Pipeline>::Material;
    fn button_color(&self, state: ButtonState) -> Color {
        match state {
            ButtonState::Normal => Color::new_rgb(0.5, 0.5, 0.5),
            ButtonState::Hover => Color::new_rgb(0.6, 0.6, 0.6),
            ButtonState::Press => Color::new_rgb(0.4, 0.4, 0.4),
            ButtonState::Disable => Color::new_rgba(0.5, 0.5, 0.5, 0.5),
        }
    }
}

pub type GuiTextBrush<Renderer> =
    TextBrush<<Renderer as GuiRenderer>::Pipeline, <Renderer as GuiRenderer>::TextLayer>;
pub type GuiBatcher<'a, Renderer> = Batcher<'a, <Renderer as GuiRenderer>::Pipeline>;

#[derive(Default, Clone, Copy)]
pub struct LayoutInfo {
    pub size: Vec2,
    pub grow: bool,
}

impl LayoutInfo {
    pub fn from_size(size: Vec2) -> Self {
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

pub struct WidgetLayout<Renderer> {
    pub widget: Wrc<Renderer>,
    pub rect: Rect,
}

impl<Renderer> Clone for WidgetLayout<Renderer> {
    fn clone(&self) -> Self {
        WidgetLayout {
            widget: self.widget.clone(),
            rect: self.rect.clone(),
        }
    }
}

pub struct WidgetLayouts<Renderer> {
    layouts: Vec<WidgetLayout<Renderer>>,
    sublayers: Vec<WidgetLayout<Renderer>>,
}

impl<Renderer: GuiRenderer> WidgetLayouts<Renderer> {
    fn new() -> Self {
        WidgetLayouts {
            layouts: Vec::new(),
            sublayers: Vec::new(),
        }
    }
    fn clear(&mut self) {
        self.layouts.clear();
        self.sublayers.clear();
    }
    fn root(&self) -> Option<Wrc<Renderer>> {
        self.layouts.first().map(|layout| layout.widget.clone())
    }
    fn iter(&self) -> std::slice::Iter<WidgetLayout<Renderer>> {
        self.layouts.iter()
    }
    pub fn add_dyn(&mut self, widget: &Wrc<Renderer>, rect: Rect) -> Vec2 {
        self.layouts.push(WidgetLayout {
            widget: widget.clone(),
            rect,
        });
        if widget.borrow().sublayer(rect).is_some() {
            self.sublayers.push(WidgetLayout {
                widget: widget.clone(),
                rect,
            });
        }
        widget.borrow_mut().layout_children(self, rect)
    }
    pub fn add<W>(&mut self, widget: &WidgetRc<W>, rect: Rect) -> Vec2
    where
        W: Widget<Renderer = Renderer>,
    {
        let widget: Wrc<Renderer> = widget.0.clone();
        self.add_dyn(&widget, rect)
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

#[derive(Default, Clone)]
pub struct GuiInput {
    pub grabbed: bool,
    pub pointer: Pos2,
    pub primary: Trigger,
    pub secondary: Trigger,
}

impl GuiInput {
    pub fn new() -> Self {
        GuiInput::default()
    }
    pub fn with_pointer_offset(&self, offset: Vec2) -> Self {
        GuiInput {
            grabbed: self.grabbed,
            pointer: self.pointer - offset,
            primary: self.primary.clone(),
            secondary: self.secondary.clone(),
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
    pub fn update(&mut self) {
        self.grabbed = false;
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
    Grab,
    Event(WidgetEvent),
}

pub struct GuiSublayer<'a, Renderer> {
    pub layer: &'a GuiLayer<Renderer>,
    pub offset: Vec2,
    pub clip: Rect,
}

#[allow(unused)]
pub trait Widget: 'static {
    type Renderer: GuiRenderer;
    fn layout_info(&self) -> LayoutInfo;
    fn layout_children(&mut self, layouts: &mut WidgetLayouts<Self::Renderer>, rect: Rect) -> Vec2 {
        self.layout_info().size
    }
    fn reset_input(&mut self) {}
    fn handle_input(&mut self, input: &GuiInput, rect: Rect) -> WidgetInput {
        WidgetInput::Pass
    }
    fn sublayer(&self, rect: Rect) -> Option<GuiSublayer<Self::Renderer>> {
        None
    }
    fn draw_text(
        &self,
        renderer: &Self::Renderer,
        text_brush: &mut GuiTextBrush<Self::Renderer>,
        layer: <Self::Renderer as GuiRenderer>::TextLayer,
        rect: Rect,
    ) {
    }
    fn draw(
        &self,
        renderer: &Self::Renderer,
        batcher: &mut GuiBatcher<Self::Renderer>,
        rect: Rect,
    ) {
    }
}

type Wrc<Renderer> = Rc<RefCell<dyn Widget<Renderer = Renderer>>>;

pub struct WidgetRc<T: ?Sized>(Rc<RefCell<T>>);

impl<T: Widget> WidgetRc<T> {
    pub fn new(widget: T) -> Self {
        WidgetRc(Rc::new(RefCell::new(widget)))
    }
}
impl<T: ?Sized> WidgetRc<T> {
    pub fn borrow(&self) -> std::cell::Ref<T> {
        self.0.borrow()
    }
    pub fn borrow_mut(&self) -> std::cell::RefMut<T> {
        self.0.borrow_mut()
    }
}
impl<T: ?Sized> Clone for WidgetRc<T> {
    fn clone(&self) -> Self {
        WidgetRc(self.0.clone())
    }
}

pub struct GuiLayer<Renderer> {
    layouts: WidgetLayouts<Renderer>,
    grabbed_input: Option<WidgetLayout<Renderer>>,
}

impl<Renderer: GuiRenderer> GuiLayer<Renderer> {
    pub fn new() -> Self {
        GuiLayer::default()
    }
    pub fn is_input_grabbed(&self) -> bool {
        self.grabbed_input.is_some()
    }
    pub fn clear(&mut self) {
        self.layouts.clear();
        self.grabbed_input = None;
    }
    fn layout_dyn(&mut self, root: &Wrc<Renderer>, rect: Rect) -> Vec2 {
        self.clear();
        self.layouts.add_dyn(root, rect)
    }
    pub fn layout_rc<W>(&mut self, root: &WidgetRc<W>, rect: Rect) -> Vec2
    where
        W: Widget<Renderer = Renderer>,
    {
        self.clear();
        self.layouts.add(root, rect)
    }
    pub fn relayout(&mut self, rect: Rect) -> Vec2 {
        if let Some(root) = self.layouts.root() {
            self.layout_dyn(&root, rect)
        } else {
            Vec2::ZERO
        }
    }
    pub fn reset_input(&mut self) {
        self.grabbed_input = None;
        for item in self.layouts.iter() {
            let mut widget = item.widget.borrow_mut();
            widget.reset_input();
        }
    }
    pub fn handle_input(&mut self, input: &GuiInput) -> WidgetInput {
        if let Some(item) = self.grabbed_input.as_ref() {
            let input = GuiInput {
                grabbed: true,
                ..input.clone()
            };
            let widget_input = item.widget.borrow_mut().handle_input(&input, item.rect);
            if !matches!(widget_input, WidgetInput::Grab) {
                self.grabbed_input = None;
            }
            if let WidgetInput::Event(event) = widget_input {
                WidgetInput::Event(event)
            } else if self.grabbed_input.is_some() {
                WidgetInput::Grab
            } else {
                widget_input
            }
        } else {
            let mut blocked = false;
            let mut widget_event = None;
            for item in self.layouts.iter().rev() {
                let mut widget = item.widget.borrow_mut();
                if !blocked && item.rect.contains(input.pointer) {
                    match widget.handle_input(input, item.rect) {
                        WidgetInput::Pass => {}
                        WidgetInput::Block => blocked = true,
                        WidgetInput::Grab => {
                            blocked = true;
                            self.grabbed_input = Some(item.clone());
                        }
                        WidgetInput::Event(event) => {
                            blocked = true;
                            widget_event = Some(event);
                        }
                    }
                } else {
                    widget.reset_input();
                }
            }
            if let Some(event) = widget_event {
                WidgetInput::Event(event)
            } else if self.grabbed_input.is_some() {
                WidgetInput::Grab
            } else if blocked {
                WidgetInput::Block
            } else {
                WidgetInput::Pass
            }
        }
    }
    fn draw_text(
        &self,
        renderer: &Renderer,
        text_brush: &mut GuiTextBrush<Renderer>,
        layer: Renderer::TextLayer,
    ) {
        for item in self.layouts.iter() {
            item.widget
                .borrow_mut()
                .draw_text(renderer, text_brush, layer.clone(), item.rect);
        }
    }
    fn draw(&self, renderer: &Renderer, batcher: &mut GuiBatcher<Renderer>) {
        for item in self.layouts.iter() {
            item.widget.borrow_mut().draw(renderer, batcher, item.rect);
        }
    }
}
impl<Renderer: GuiRenderer> Default for GuiLayer<Renderer> {
    fn default() -> Self {
        GuiLayer {
            layouts: WidgetLayouts::new(),
            grabbed_input: None,
        }
    }
}

pub struct Gui<Renderer: GuiRenderer> {
    layers: BTreeMap<Renderer::GuiLayer, GuiLayer<Renderer>>,
    input: GuiInput,
}

impl<Renderer: GuiRenderer> Gui<Renderer> {
    pub fn new() -> Self {
        Gui {
            layers: BTreeMap::new(),
            input: GuiInput::new(),
        }
    }
    pub fn clear_layout(&mut self, layer: &Renderer::GuiLayer) {
        if let Some(gui) = self.layers.get_mut(layer) {
            gui.clear();
        }
    }
    pub fn layout<W>(&mut self, layer: Renderer::GuiLayer, root: W, rect: Rect) -> Vec2
    where
        W: Widget<Renderer = Renderer>,
    {
        self.layout_rc(layer, &WidgetRc::new(root), rect)
    }
    pub fn layout_rc<W>(
        &mut self,
        layer: Renderer::GuiLayer,
        root: &WidgetRc<W>,
        rect: Rect,
    ) -> Vec2
    where
        W: Widget<Renderer = Renderer>,
    {
        self.layers.entry(layer).or_default().layout_rc(root, rect)
    }
    pub fn relayout(&mut self, rect: Rect) {
        for gui in self.layers.values_mut() {
            gui.relayout(rect);
        }
    }
    pub fn relayout_with<F>(&mut self, f: F)
    where
        F: Fn(&Renderer::GuiLayer) -> Rect,
    {
        for (layer, gui) in self.layers.iter_mut() {
            gui.relayout(f(layer));
        }
    }
    pub fn process_input(&mut self, event: Option<GuiInputEvent>) {
        if let Some(event) = event {
            self.input.process(event);
        }
    }
    pub fn update_input(&mut self) -> WidgetInput {
        let mut final_input = WidgetInput::Pass;
        for gui in self.layers.values_mut().rev() {
            let widget_input = gui.handle_input(&self.input);
            if !matches!(widget_input, WidgetInput::Pass) {
                final_input = widget_input;
                break;
            }
        }
        self.input.update();
        final_input
    }
    fn foreach_sublayer<F>(
        layer: &Renderer::GuiLayer,
        gui: &GuiLayer<Renderer>,
        scroll: Option<(Vec2, Rect)>,
        index: &mut usize,
        f: &mut F,
    ) where
        F: FnMut(&GuiLayer<Renderer>, Renderer::TextLayer, Option<(Vec2, Rect)>),
    {
        f(gui, Renderer::text_layer(layer, *index), scroll);
        for sublayer in gui.layouts.sublayers.iter() {
            *index += 1;
            let sublayer_widget = sublayer.widget.borrow();
            let GuiSublayer {
                layer: gui,
                offset,
                clip,
            } = sublayer_widget.sublayer(sublayer.rect).unwrap();
            Self::foreach_sublayer(layer, gui, Some((offset, clip)), index, f);
        }
    }
    pub fn draw_text(&self, gui_renderer: &Renderer, text_brush: &mut GuiTextBrush<Renderer>) {
        for (layer, gui) in self.layers.iter() {
            let mut index = 0;
            Self::foreach_sublayer(layer, gui, None, &mut index, &mut |gui, layer, _| {
                gui.draw_text(gui_renderer, text_brush, layer)
            });
        }
    }
    pub fn draw(
        &mut self,
        gui_renderer: &Renderer,
        text_brush: &mut GuiTextBrush<Renderer>,
        batcher: &mut GuiBatcher<Renderer>,
    ) {
        for (layer, gui) in self.layers.iter() {
            let mut index = 0;
            Self::foreach_sublayer(layer, gui, None, &mut index, &mut |gui, layer, scroll| {
                let (camera, clip) = scroll
                    .map(|(offset, clip)| (TSTransform::from_translation(offset), Some(clip)))
                    .unwrap_or_default();
                batcher.pipeline.set_clip(batcher.context, clip);
                batcher.set_camera(camera);
                gui.draw(gui_renderer, batcher);
                text_brush.draw_layer(batcher, &layer);
            });
        }
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

pub enum ContainerItem<Renderer> {
    Empty(LayoutInfo),
    Widget(Wrc<Renderer>),
}

impl<Renderer: GuiRenderer> ContainerItem<Renderer> {
    pub fn from_widget<W>(widget: &WidgetRc<W>) -> Self
    where
        W: Widget<Renderer = Renderer>,
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

pub struct Container<Renderer> {
    layout: LayoutInfo,
    direction: Direction,
    cross_axis: CrossAxis,
    padding: Padding,
    size: Vec2,
    items: Vec<ContainerItem<Renderer>>,
}

impl<Renderer: GuiRenderer> Container<Renderer> {
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
    fn add_item(&mut self, item: ContainerItem<Renderer>) {
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
        self.add_item(ContainerItem::Empty(empty))
    }
    pub fn add<W>(&mut self, widget: W) -> WidgetRc<W>
    where
        W: Widget<Renderer = Renderer>,
    {
        let widget = WidgetRc::new(widget);
        self.add_item(ContainerItem::from_widget(&widget));
        widget
    }
    pub fn add_rc<W>(&mut self, widget: &WidgetRc<W>)
    where
        W: Widget<Renderer = Renderer>,
    {
        self.add_item(ContainerItem::from_widget(widget));
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
impl<Renderer: GuiRenderer> Widget for Container<Renderer> {
    type Renderer = Renderer;
    fn layout_info(&self) -> LayoutInfo {
        let layout_size = self.size + self.padding.min_size();
        LayoutInfo {
            size: self.layout.size.max(layout_size),
            grow: self.layout.grow,
        }
    }
    fn layout_children(&mut self, layouts: &mut WidgetLayouts<Renderer>, mut rect: Rect) -> Vec2 {
        if self.items.is_empty() {
            return Vec2::ZERO;
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
                layouts.add_dyn(widget, widget_rect);
            }
            main_pos += item_main + self.padding.between;
        }
        match self.direction {
            Direction::Horizontal => {
                vec2(main_pos - self.padding.between, cross_size) + self.padding.min_size()
            }
            Direction::Vertical => {
                vec2(cross_size, main_pos - self.padding.between) + self.padding.min_size()
            }
        }
    }
}

pub struct GridContainer<Renderer> {
    layout: LayoutInfo,
    padding: Padding,
    item_size: Vec2,
    items: Vec<ContainerItem<Renderer>>,
}

impl<Renderer: GuiRenderer> GridContainer<Renderer> {
    pub fn new(class: &str) -> Self {
        GridContainer {
            layout: LayoutInfo::from_style(class, Vec2::ZERO),
            padding: style(class, "padding"),
            item_size: Vec2::ZERO,
            items: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    fn add_item(&mut self, item: ContainerItem<Renderer>) {
        self.item_size = self.item_size.max(item.layout_info().size);
        self.items.push(item);
    }
    pub fn add_empty(&mut self, empty: LayoutInfo) {
        self.add_item(ContainerItem::Empty(empty))
    }
    pub fn add<W>(&mut self, widget: W) -> WidgetRc<W>
    where
        W: Widget<Renderer = Renderer>,
    {
        let widget = WidgetRc::new(widget);
        self.add_item(ContainerItem::from_widget(&widget));
        widget
    }
    pub fn add_rc<W>(&mut self, widget: &WidgetRc<W>)
    where
        W: Widget<Renderer = Renderer>,
    {
        self.add_item(ContainerItem::from_widget(widget));
    }
}
impl<Renderer> Default for GridContainer<Renderer> {
    fn default() -> Self {
        GridContainer {
            layout: LayoutInfo::default(),
            padding: Padding::default(),
            item_size: Vec2::ZERO,
            items: Vec::new(),
        }
    }
}
impl<Renderer: GuiRenderer> Widget for GridContainer<Renderer> {
    type Renderer = Renderer;
    fn layout_info(&self) -> LayoutInfo {
        self.layout
    }
    fn layout_children(&mut self, layouts: &mut WidgetLayouts<Renderer>, mut rect: Rect) -> Vec2 {
        if self.items.is_empty() {
            return Vec2::ZERO;
        }
        rect.min += vec2(self.padding.left, self.padding.top);
        rect.max -= vec2(self.padding.right, self.padding.bottom);
        let mut pos = Pos2::ZERO;
        for item in self.items.iter() {
            if let ContainerItem::Widget(widget) = item {
                let widget_rect =
                    Rect::from_min_size(pos, self.item_size).translate(rect.min.to_vec2());
                layouts.add_dyn(widget, widget_rect);
            }
            pos.x += self.item_size.x + self.padding.between;
            if pos.x + self.item_size.x > rect.width() {
                pos.x = 0.0;
                pos.y += self.item_size.y + self.padding.between;
            }
        }
        if pos.x > 0.0 {
            pos.y += self.item_size.y;
        } else {
            pos.y -= self.padding.between;
        }
        vec2(rect.width(), pos.y) + self.padding.min_size()
    }
}

pub struct Label<Renderer> {
    layout: LayoutInfo,
    font: Font,
    color: Color,
    text: Cow<'static, str>,
    align: Align2,
    wrap: bool,
    _marker: PhantomData<Renderer>,
}

impl<Renderer> Label<Renderer> {
    pub fn new<S: Into<Cow<'static, str>>>(class: &str, text: S) -> Self {
        Label {
            layout: LayoutInfo::from_style(class, Vec2::ZERO),
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
}
impl<Renderer: GuiRenderer> Label<Renderer> {
    pub fn autosize(
        &mut self,
        text_brush: &mut GuiTextBrush<Renderer>,
        layer: Renderer::TextLayer,
    ) {
        let size = text_brush.text_bounds(
            layer,
            &Text {
                position: Pos2::ZERO,
                align: self.align,
                wrap: if self.wrap {
                    Some(self.layout.size.x)
                } else {
                    None
                },
                font: self.font,
                color: self.color,
                text: self.text.as_ref().into(),
            },
        );
        if let Some(size) = size {
            if !self.wrap {
                self.layout.size.x = size.width();
            }
            self.layout.size.y = size.height();
        }
    }
}
impl<Renderer: GuiRenderer> Widget for Label<Renderer> {
    type Renderer = Renderer;
    fn layout_info(&self) -> LayoutInfo {
        self.layout
    }
    fn draw_text(
        &self,
        _renderer: &Renderer,
        text_brush: &mut GuiTextBrush<Renderer>,
        layer: Renderer::TextLayer,
        rect: Rect,
    ) {
        text_brush.queue(
            layer,
            &Text {
                position: self.align.pos_in_rect(&rect),
                align: self.align,
                wrap: if self.wrap { Some(rect.width()) } else { None },
                font: self.font,
                color: self.color,
                text: self.text.as_ref().into(),
            },
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hover,
    Press,
    Disable,
}

pub struct Button<Renderer> {
    name: Cow<'static, str>,
    layout: LayoutInfo,
    label: Label<Renderer>,
    state: ButtonState,
    event_payload: Option<Rc<dyn Any>>,
}

impl<Renderer> Button<Renderer> {
    pub fn new<S: Into<Cow<'static, str>>, L: Into<Cow<'static, str>>>(
        class: &str,
        name: S,
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
impl<Renderer: GuiRenderer> Widget for Button<Renderer> {
    type Renderer = Renderer;
    fn layout_info(&self) -> LayoutInfo {
        self.layout
    }
    fn reset_input(&mut self) {
        if self.state != ButtonState::Disable {
            self.state = ButtonState::Normal;
        }
    }
    fn handle_input(&mut self, input: &GuiInput, _rect: Rect) -> WidgetInput {
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
    fn draw_text(
        &self,
        _renderer: &Renderer,
        text_brush: &mut GuiTextBrush<Renderer>,
        layer: Renderer::TextLayer,
        rect: Rect,
    ) {
        text_brush.queue(
            layer,
            &Text {
                position: rect.center(),
                align: Align2::CENTER_CENTER,
                wrap: None,
                font: self.label.font,
                color: Color::BLACK,
                text: self.label.text.as_ref().into(),
            },
        );
    }
    fn draw(&self, renderer: &Renderer, batcher: &mut GuiBatcher<Renderer>, rect: Rect) {
        ColorRect(renderer.button_color(self.state), rect)
            .draw(batcher, &renderer.button_material(self.state));
    }
}

pub struct Slider<Renderer> {
    layout: LayoutInfo,
    state: ButtonState,
    direction: Direction,
    handle_size: f32,
    value: f32,
    _marker: PhantomData<Renderer>,
}

impl<Renderer> Slider<Renderer> {
    pub fn new(class: &str, direction: Direction) -> Self {
        Slider {
            layout: LayoutInfo::from_style(class, vec2(24.0, 24.0)),
            state: ButtonState::Normal,
            direction,
            handle_size: 0.0,
            value: 0.0,
            _marker: PhantomData,
        }
    }
    fn handle_size(&self, rect: Rect) -> f32 {
        match self.direction {
            Direction::Horizontal => f32::max(rect.width() * self.handle_size, rect.height()),
            Direction::Vertical => f32::max(rect.height() * self.handle_size, rect.width()),
        }
    }
    fn handle_rect(&self, mut rect: Rect) -> Rect {
        let handle_size = self.handle_size(rect);
        match self.direction {
            Direction::Horizontal => {
                let track_size = rect.width() - handle_size;
                rect.set_width(handle_size);
                rect.translate(vec2(self.value * track_size, 0.0))
            }
            Direction::Vertical => {
                let track_size = rect.height() - handle_size;
                rect.set_height(handle_size);
                rect.translate(vec2(0.0, self.value * track_size))
            }
        }
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.state = ButtonState::Normal;
        } else {
            self.state = ButtonState::Disable;
        }
    }
}
impl<Renderer: GuiRenderer> Widget for Slider<Renderer> {
    type Renderer = Renderer;
    fn layout_info(&self) -> LayoutInfo {
        self.layout
    }
    fn reset_input(&mut self) {
        if self.state != ButtonState::Disable {
            self.state = ButtonState::Normal;
        }
    }
    fn handle_input(&mut self, input: &GuiInput, mut rect: Rect) -> WidgetInput {
        if self.state != ButtonState::Disable {
            self.state = if input.primary.pressed() {
                ButtonState::Press
            } else {
                ButtonState::Hover
            };
        }
        if self.state == ButtonState::Press {
            let handle_size = self.handle_size(rect);
            match self.direction {
                Direction::Horizontal => {
                    rect = rect.shrink2(vec2(handle_size / 2.0, 0.0));
                    self.value = emath::inverse_lerp(rect.min.x..=rect.max.x, input.pointer.x)
                        .unwrap_or_default()
                        .clamp(0.0, 1.0);
                }
                Direction::Vertical => {
                    rect = rect.shrink2(vec2(0.0, handle_size / 2.0));
                    self.value = emath::inverse_lerp(rect.min.y..=rect.max.y, input.pointer.y)
                        .unwrap_or_default()
                        .clamp(0.0, 1.0);
                }
            }
            WidgetInput::Grab
        } else {
            WidgetInput::Block
        }
    }
    fn draw(&self, renderer: &Renderer, batcher: &mut GuiBatcher<Renderer>, rect: Rect) {
        ColorRect(renderer.button_color(self.state), self.handle_rect(rect))
            .draw(batcher, &renderer.button_material(self.state));
    }
}

pub struct ScrollArea<Renderer> {
    layout: LayoutInfo,
    scrollbar: WidgetRc<Slider<Renderer>>,
    content: Wrc<Renderer>,
    content_layer: GuiLayer<Renderer>,
    content_size: Vec2,
}

impl<Renderer: GuiRenderer> ScrollArea<Renderer> {
    pub fn new<W>(class: &str, direction: Direction, content: W) -> Self
    where
        W: Widget<Renderer = Renderer>,
    {
        let scrollbar = Slider::new("scrollbar", direction);
        ScrollArea {
            layout: LayoutInfo::from_style(class, Vec2::ZERO),
            scrollbar: WidgetRc::new(scrollbar),
            content: Rc::new(RefCell::new(content)),
            content_layer: GuiLayer::new(),
            content_size: Vec2::ZERO,
        }
    }
    fn rects(&self, rect: Rect) -> (Rect, Rect) {
        let scrollbar = self.scrollbar.borrow();
        match scrollbar.direction {
            Direction::Horizontal => {
                rect.split_top_bottom_at_y(rect.bottom() - scrollbar.layout.size.y)
            }
            Direction::Vertical => {
                rect.split_left_right_at_x(rect.right() - scrollbar.layout.size.x)
            }
        }
    }
    fn scroll(&self, rect: Rect) -> Vec2 {
        let scrollbar = self.scrollbar.borrow();
        match scrollbar.direction {
            Direction::Horizontal => vec2(
                (self.content_size.x - rect.width()).max(0.0) * -scrollbar.value,
                0.0,
            ),
            Direction::Vertical => vec2(
                0.0,
                (self.content_size.y - rect.height()).max(0.0) * -scrollbar.value,
            ),
        }
    }
}
impl<Renderer: GuiRenderer> Widget for ScrollArea<Renderer> {
    type Renderer = Renderer;
    fn layout_info(&self) -> LayoutInfo {
        self.layout
    }
    fn layout_children(&mut self, layouts: &mut WidgetLayouts<Renderer>, rect: Rect) -> Vec2 {
        let (content_rect, scrollbar_rect) = self.rects(rect);
        self.content_size = self.content_layer.layout_dyn(
            &self.content,
            Rect::from_min_size(Pos2::ZERO, content_rect.size()),
        );
        let mut scrollbar = self.scrollbar.borrow_mut();
        match scrollbar.direction {
            Direction::Horizontal => {
                scrollbar.handle_size =
                    (scrollbar_rect.width() / self.content_size.x).clamp(0.0, 1.0)
            }
            Direction::Vertical => {
                scrollbar.handle_size =
                    (scrollbar_rect.height() / self.content_size.y).clamp(0.0, 1.0)
            }
        }
        std::mem::drop(scrollbar);
        layouts.add(&self.scrollbar, scrollbar_rect);
        rect.size()
    }
    fn reset_input(&mut self) {
        self.content_layer.reset_input();
    }
    fn handle_input(&mut self, input: &GuiInput, rect: Rect) -> WidgetInput {
        let offset = rect.min.to_vec2() + self.scroll(rect);
        self.content_layer
            .handle_input(&input.with_pointer_offset(offset))
    }
    fn sublayer(&self, rect: Rect) -> Option<GuiSublayer<Self::Renderer>> {
        let (content_rect, _) = self.rects(rect);
        let offset = rect.min.to_vec2() + self.scroll(rect);
        Some(GuiSublayer {
            layer: &self.content_layer,
            offset,
            clip: content_rect,
        })
    }
}

// pub struct ScrollList<I, W, T> {
//     items: Vec<I>,
//     item_fn: Box<dyn Fn(&I) -> W>,
//     visible_items: RangeInclusive<usize>,
//     padding: Padding,
//     item_size: Vec2,
//     scroll_area: ScrollArea<T>,
// }

// impl<I, W, T> ScrollList<I, W, T>
// where
//     I: 'static,
//     W: Widget<Primitive = T>,
//     T: Primitive,
// {
//     pub fn new<F>(class: &str, items: Vec<I>, item_fn: F) -> Self
//     where
//         F: Fn(&I) -> W + 'static,
//     {
//         let padding: Padding = style(class, "padding");
//         let item_size = items
//             .first()
//             .map(|item| item_fn(item).layout_info().size)
//             .unwrap_or_default();
//         let content_size = padding.min_size()
//             + Vec2::new(
//                 item_size.x,
//                 (item_size.y * items.len() as f32)
//                     + (padding.between * items.len().saturating_sub(1) as f32),
//             );
//         let mut content = Container::default();
//         content.add_empty(LayoutInfo::from_size(content_size));
//         ScrollList {
//             items,
//             item_fn: Box::new(item_fn),
//             visible_items: 1..=0,
//             padding,
//             item_size,
//             scroll_area: ScrollArea::new(class, Direction::Vertical, content),
//         }
//     }
//     fn empty_items_size(&self, count: usize) -> Vec2 {
//         Vec2::new(
//             self.item_size.x,
//             (self.item_size.y * count as f32)
//                 + (self.padding.between * count.saturating_sub(1) as f32),
//         )
//     }
//     fn visible_items(&self, rect: Rect) -> RangeInclusive<usize> {
//         let scroll = self.scroll_area.scroll(rect);
//         let item_height = self.item_size.y + self.padding.between;
//         let first = -scroll.y / item_height;
//         let last = (-scroll.y + rect.height()) / item_height;
//         let last_index = self.items.len().saturating_sub(1);
//         (first.floor() as usize).min(last_index)..=(last.floor() as usize).min(last_index)
//     }
//     fn build_content(&mut self, rect: Rect, layout: bool) {
//         let mut content = Container {
//             direction: Direction::Vertical,
//             padding: self.padding,
//             ..Default::default()
//         };
//         if *self.visible_items.start() > 0 {
//             content.add_empty(LayoutInfo::from_size(
//                 self.empty_items_size(*self.visible_items.start()),
//             ));
//         }
//         for index in self.visible_items.clone() {
//             content.add((self.item_fn)(&self.items[index]));
//         }
//         let last_index = self.items.len().saturating_sub(1);
//         if *self.visible_items.end() < last_index {
//             content.add_empty(LayoutInfo::from_size(
//                 self.empty_items_size(last_index - *self.visible_items.end()),
//             ));
//         }
//         self.scroll_area.content = Rc::new(RefCell::new(content));
//         if layout {
//             let (content_rect, _) = self.scroll_area.rects(rect);
//             self.scroll_area.content_layout.layout_dyn(
//                 &self.scroll_area.content,
//                 Rect::from_min_size(Pos2::ZERO, content_rect.size()),
//             );
//         }
//     }
// }
// impl<I, W, T> Widget for ScrollList<I, W, T>
// where
//     I: 'static,
//     W: Widget<Primitive = T>,
//     T: Primitive,
// {
//     type Primitive = T;
//     fn layout_info(&self) -> LayoutInfo {
//         self.scroll_area.layout_info()
//     }
//     fn layout_children(&mut self, layouts: &mut WidgetLayouts<T>, rect: Rect) -> Vec2 {
//         if self.visible_items.is_empty() {
//             self.visible_items = self.visible_items(rect);
//             self.build_content(rect, false);
//         }
//         self.scroll_area.layout_children(layouts, rect)
//     }
//     fn reset_input(&mut self) {
//         self.scroll_area.reset_input();
//     }
//     fn handle_input(&mut self, rect: Rect, input: &GuiInput) -> WidgetInput {
//         self.scroll_area.handle_input(rect, input)
//     }
//     fn draw(&mut self, renderer: &mut GuiRenderer<T>, rect: Rect) {
//         let visible_items = self.visible_items(rect);
//         if visible_items != self.visible_items {
//             self.visible_items = visible_items;
//             self.build_content(rect, true);
//         }
//         self.scroll_area.draw(renderer, rect);
//     }
// }
