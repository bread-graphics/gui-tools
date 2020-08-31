// MIT/Apache2 License

#![cfg(feature = "async")]

use super::X11Runtime;
use crate::mutex::Mutex;
use core::task::Waker;
use std::{thread, sync::mpsc};
use storagevec::StorageVec;

struct X11EventServer<'a> {
    xrs: &'a X11Runtime,
    waker: Mutex<Option<Waker>>,
    receive_events: Option<mpsc::Receiver<StorageVec<Event, 5>>>,
    send_kill_signal: Option<mpsc::Sender<()>>,
    joiner: Option<thread::JoinHandle<()>>,
    current_coll: StorageVec<Event, 5>,
}

impl<'a> X11EventServer<'a> {
    fn new(xrs: &'a X11Runtime) -> Self {
        Self { xrs, waker: Mutex::new(None), receive_events: None, send_kill_signal: None, joiner: None, current_coll: StorageVec::new() }
    }
}

impl<'a> Stream for X11EventServer<'a> {
    type Item = Event;
}
