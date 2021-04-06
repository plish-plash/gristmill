use std::sync::mpsc;

// -------------------------------------------------------------------------------------------------
// Signals are for any number of senders to communicate with a known receiver.

// TODO when a signal is sent, the target should respond directly, instead of having to read the message from a queue.
pub struct SignalTarget<Data> {
    sender: mpsc::Sender<Data>,
    receiver: mpsc::Receiver<Data>,
}

impl<Data> SignalTarget<Data> {
    pub fn new() -> SignalTarget<Data> {
        let (sender, receiver) = mpsc::channel();
        SignalTarget { sender, receiver }
    }
    pub fn create_signal(&self) -> Signal<Data> {
        Signal { sender: self.sender.clone() }
    }
}

pub struct Signal<Data> {
    sender: mpsc::Sender<Data>,
}

impl<Data> Signal<Data> {
    pub fn send(&self, data: Data) {
        match self.sender.send(data) {
            Ok(()) => (),
            Err(_) => (), // TODO log the error
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Events are for a known sender to communicate with any number of receivers.

// TODO
pub struct EventBroadcaster {

}

pub struct EventListener;
