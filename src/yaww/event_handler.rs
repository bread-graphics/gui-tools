// MIT/Apache2 License

use super::{convert_yaww_event, YawwDisplay};
use crate::{
    display::{DisplaySumRef, EventHandler},
    window::WindowProps,
};
use flume::{Receiver, TryRecvError};
use std::collections::hash_map::{Entry, HashMap};
use yaww::{event::Event, server::GuiThread, window::Window};

pub(crate) enum EventHandlerUpdate {
    WindowProps(Window, WindowProps),
    SetupEventHandler(EventHandler),
}

/// Event handler.
pub(crate) fn event_handler(
    mut yaww_display: YawwDisplay,
    rx: Receiver<EventHandlerUpdate>,
) -> impl FnMut(GuiThread, Event) -> yaww::Result + 'static {
    // function state
    let mut window_properties = HashMap::new();
    let mut event_handler: Option<EventHandler> = None;

    move |gt, ev| {
        // before we run, check the rx for incoming messages
        match rx.try_recv() {
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                return Ok(());
            }
            Ok(EventHandlerUpdate::WindowProps(win, props)) => match window_properties.entry(win) {
                Entry::Vacant(v) => {
                    v.insert(props);
                }
                Entry::Occupied(o) => {
                    unimplemented!("TODO: OR props together")
                }
            },
            Ok(EventHandlerUpdate::SetupEventHandler(evh)) => {
                event_handler = Some(evh);
            }
        }

        // TODO: run basic processing on the events, i.e. for background drawing

        let dpy = DisplaySumRef::Yaww(&mut yaww_display);

        // if possible, run the user-defined event handler
        if let Some(ref mut event_handler) = event_handler {
            let ev = convert_yaww_event(ev)?;
            if let Err(e) = (event_handler)(dpy, ev) {
                return Err(yaww::Error::Dynamic(Box::new(e)));
            }
        }

        Ok(())
    }
}
