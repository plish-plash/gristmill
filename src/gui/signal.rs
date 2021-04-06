use std::any::Any;
use std::sync::{Arc, RwLock};

pub struct SignalTarget {
    received_signals: RwLock<Vec<(SignalIdentifier, Option<Box<dyn Any>>)>>,
}

impl SignalTarget {
    pub fn new() -> Arc<SignalTarget> {
        Arc::new(SignalTarget { received_signals: RwLock::default() })
    }

    fn receive(&self, identifier: SignalIdentifier, value: Option<Box<dyn Any>>) {
        self.received_signals.write().unwrap().push((identifier, value));
    }
    pub fn process<F>(&self, mut f: F) where F: FnMut(SignalIdentifier, Option<Box<dyn Any>>) {
        for (identifier, value) in self.received_signals.write().unwrap().drain(..) {
            f(identifier, value);
        }
    }
}

#[derive(Clone)]
pub enum SignalIdentifier {
    Index(usize),
    String(&'static str),
    OwnedString(String),
}

impl SignalIdentifier {
    pub fn index(&self) -> usize {
        match self {
            SignalIdentifier::Index(i) => *i,
            _ => panic!("SignalIdentifier is not an index")
        }
    }
    pub fn string(&self) -> &str {
        match self {
            SignalIdentifier::Index(_) => panic!("SignalIdentifier is not a string"),
            SignalIdentifier::String(string) => string,
            SignalIdentifier::OwnedString(string) => string,
        }
    }
}

pub struct Signal {
    target: Arc<SignalTarget>,
    identifier: SignalIdentifier,
}

impl Signal {
    pub fn new(target: Arc<SignalTarget>, identifier: SignalIdentifier) -> Signal {
        Signal { target, identifier }
    }
    pub fn new_index(target: Arc<SignalTarget>, index: usize) -> Signal {
        Signal { target, identifier: SignalIdentifier::Index(index) }
    }
    pub fn new_named(target: Arc<SignalTarget>, name: &'static str) -> Signal {
        Signal { target, identifier: SignalIdentifier::String(name) }
    }
    pub fn send(&self) {
        self.target.receive(self.identifier.clone(), None);
    }
    pub fn send_value<T>(&self, value: T) where T: 'static {
        self.target.receive(self.identifier.clone(), Some(Box::new(value)));
    }
}

