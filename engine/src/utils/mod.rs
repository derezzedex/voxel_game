pub mod timer;
pub mod debug;
pub mod camera;

use std::sync::mpsc::{channel, Receiver, Sender};
pub struct MessageChannel<T>{
    pub receiver: Receiver<T>,
    pub sender: Sender<T>,
}

impl<T> MessageChannel<T>{
    pub fn new() -> Self{
        let (sender, receiver) = channel();

        Self{
            receiver,
            sender,
        }
    }
}
