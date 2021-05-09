use std::cmp::Ordering;
use super::{Gui, WidgetNode, text::Text, button::Button};

pub trait GuiValueListener<T> {
    fn value_changed(&mut self, gui: &mut Gui, new_value: &T);
}

pub struct GuiValue<T> where T: Clone + PartialEq {
    value: Option<T>,
    listeners: Vec<Box<dyn GuiValueListener<T>>>,
}

impl<T> GuiValue<T> where T: Clone + PartialEq {
    pub fn new() -> GuiValue<T> {
        GuiValue { value: None, listeners: Vec::new() }
    }
    pub fn get(&self) -> T { self.get_ref().clone() }
    pub fn get_ref(&self) -> &T { self.value.as_ref().unwrap() }
    pub fn set(&mut self, gui: &mut Gui, value: T) {
        if self.value.is_some() && self.get_ref().eq(&value) { return; }
        self.value = Some(value);
        for listener in self.listeners.iter_mut() {
            listener.value_changed(gui, self.value.as_ref().unwrap());
        }
    }

    pub fn add_listener<L>(&mut self, listener: L) where L: GuiValueListener<T> + 'static {
        self.listeners.push(Box::new(listener))
    }
}


pub struct SetText(pub WidgetNode<Text>);
impl GuiValueListener<String> for SetText {
    fn value_changed(&mut self, gui: &mut Gui, new_value: &String) {
        gui.get_mut(self.0).unwrap().set_text(new_value.clone());
    }
}

pub struct EnableButton(pub WidgetNode<Button>);
impl GuiValueListener<bool> for EnableButton {
    fn value_changed(&mut self, gui: &mut Gui, new_value: &bool) {
        gui.get_mut(self.0).unwrap().set_enabled(*new_value);
    }
}

pub struct ConvertString<T>(pub T) where T: GuiValueListener<String>;
impl<T, I> GuiValueListener<I> for ConvertString<T> where T: GuiValueListener<String>, I: ToString {
    fn value_changed(&mut self, gui: &mut Gui, new_value: &I) {
        let value_str = new_value.to_string();
        self.0.value_changed(gui, &value_str);
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Comparison {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

impl Comparison {
    fn matches(self, ordering: Ordering) -> bool {
        match self {
            Comparison::Equal => ordering == Ordering::Equal,
            Comparison::NotEqual => ordering != Ordering::Equal,
            Comparison::Greater => ordering == Ordering::Greater,
            Comparison::GreaterEqual => ordering != Ordering::Less,
            Comparison::Less => ordering == Ordering::Less,
            Comparison::LessEqual => ordering != Ordering::Greater,
        }
    }
}

pub struct Compare<V, T>(pub Comparison, pub V, pub T) where V: PartialOrd, T: GuiValueListener<bool>;
impl<V, T> GuiValueListener<V> for Compare<V, T> where V: PartialOrd, T: GuiValueListener<bool> {
    fn value_changed(&mut self, gui: &mut Gui, new_value: &V) {
        let value_bool = new_value.partial_cmp(&self.1).map(|ord| self.0.matches(ord)).unwrap_or_default();
        self.2.value_changed(gui, &value_bool);
    }
}
