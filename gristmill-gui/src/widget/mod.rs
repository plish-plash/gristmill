mod button;
mod image;
mod panel;
mod style;
mod text;

pub use button::*;
pub use image::*;
pub use panel::*;
pub use style::{StyleValue, StyleValues};
pub use text::*;

use gristmill::{impl_downcast, CastObj, Downcast, Obj};
use serde::Deserialize;
use std::{collections::HashMap, str::FromStr};

use crate::{Gui, GuiLayout, GuiNode};
use style::{OwnedStyleRule, StyleRule};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetType(pub &'static str);

impl WidgetType {
    pub fn image() -> WidgetType {
        WidgetType("image")
    }
    pub fn text() -> WidgetType {
        WidgetType("text")
    }
    pub fn button() -> WidgetType {
        WidgetType("button")
    }
    pub fn panel() -> WidgetType {
        WidgetType("panel")
    }
}

#[derive(Default, Deserialize)]
#[serde(from = "HashMap<String, StyleValues>")]
pub struct WidgetStyles {
    styles: HashMap<OwnedStyleRule, StyleValues>,
    no_style: StyleValues,
}

impl From<HashMap<String, StyleValues>> for WidgetStyles {
    fn from(map: HashMap<String, StyleValues>) -> Self {
        let styles = map
            .into_iter()
            .map(|(k, v)| (OwnedStyleRule::from_str(&k).unwrap(), v))
            .collect();
        WidgetStyles {
            styles,
            no_style: StyleValues::new(),
        }
    }
}

impl WidgetStyles {
    pub fn new() -> WidgetStyles {
        Default::default()
    }
    pub fn add_rule(&mut self, widget: WidgetType, class: Option<&str>) -> &mut StyleValues {
        use style::Key;
        self.styles
            .insert(OwnedStyleRule::new(widget, class), StyleValues::new());
        let key = StyleRule::new(widget, class);
        let key: &dyn Key = &key;
        self.styles.get_mut(key).unwrap()
    }
    pub fn get(&self, widget: WidgetType, class: Option<&str>) -> &StyleValues {
        use style::Key;
        let key = StyleRule::new(widget, class);
        let key: &dyn Key = &key;
        self.styles.get(key).unwrap_or(&self.no_style)
    }
}

#[derive(Copy, Clone)]
pub struct InputState<'a> {
    pub input: &'a dyn crate::GuiInputActions,
    pub cursor_over: bool,
}

pub trait WidgetBehavior: Downcast {
    fn node(&self) -> Obj<GuiNode>;
    fn update(&mut self, state: InputState);
}
impl_downcast!(WidgetBehavior);

pub trait Widget: Sized {
    fn widget_type() -> WidgetType;
    fn create_with_style(gui: &mut Gui, parent: Obj<GuiNode>, style: &StyleValues) -> Self;
    fn create(gui: &mut Gui, parent: Obj<GuiNode>, class: Option<&str>) -> Self {
        let styles = gui.styles();
        let style = styles.get(Self::widget_type(), class);
        Self::create_with_style(gui, parent, style)
    }

    fn node(&self) -> Obj<GuiNode>;
    fn set_visible(&self, visible: bool) {
        self.node().write().flags.visible = visible;
    }
    fn set_layout(&self, layout: GuiLayout) {
        self.node().write().layout = layout;
    }
}

pub type WidgetObj<T> = CastObj<dyn WidgetBehavior, T>;
