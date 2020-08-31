// MIT/Apache2 License

use super::X11Runtime;
use crate::{
    event::{Event, EventType},
    runtime::Runtime,
};
use core::convert::TryInto;
use cty::c_int;
use euclid::{point2, size2};
use storagevec::StorageVec;
use x11nas::xlib::{self, Window, XConfigureEvent, XEvent};

// figures out the window of an event
// necessary because the sending window of an event might not be
// in the same field every time
fn get_event_window_xid(xev: &XEvent, ty: c_int) -> Window {
    macro_rules! found_evty {
        ($xev: expr, $t: ident, $fname: ident) => {{
            log::trace!(
                "Unsafe code: Confirmed window type to be {}, getting union field {}",
                stringify!($t),
                stringify!($fname),
            );

            unsafe { $xev.$fname }.window
        }};
    }

    match ty {
        xlib::DestroyNotify => found_evty!(xev, DestroyNotify, destroy_window),
        xlib::UnmapNotify => found_evty!(xev, UnmapNotify, unmap),
        xlib::MapNotify => found_evty!(xev, MapNotify, map),
        xlib::ConfigureNotify => found_evty!(xev, ConfigureNotify, configure),
        xlib::ReparentNotify => found_evty!(xev, ReparentNotify, reparent),
        xlib::GravityNotify => found_evty!(xev, GravityNotify, gravity),
        xlib::CirculateNotify => found_evty!(xev, CirculateNotify, circulate),
        _ => {
            log::trace!("Unsafe code: Window is not a special type, getting xany field window");
            unsafe { xev.any }.window
        }
    }
}

/// Translate an X11 event to a Gui-Tools event.
pub(crate) fn translate_x11_event(
    runtime: &X11Runtime,
    real_runtime: &Runtime,
    xev: XEvent,
) -> crate::Result<StorageVec<Event, 5>> {
    // SAFETY: the type field of this union is guaranteed
    //         by X11 protocol to be filled
    log::trace!("Unsafe code: Pulling type value out of X11 event union");
    let ty = unsafe { xev.type_ };
    log::debug!("Beginning translate of X11 event of type {}", ty);

    // figure out the window for this event
    let window = get_event_window_xid(&xev, ty);

    // get the surface that matches this event
    let wid: usize = window.try_into().unwrap();
    let surface = real_runtime.surface_at(wid);

    let mut events = StorageVec::new();

    match ty {
        xlib::ConfigureNotify => {
            // the event is a configure event, cast it to that
            log::debug!("Found the ConfigureNotify event");
            log::trace!("Unsafe code: Casting the XEvent to an XConfigureEvent");
            let xce = unsafe { xev.configure };
            let surface = surface.unwrap();

            let (current_width, current_height) = surface.size();
            let (new_width, new_height) = (xce.width as u32, xce.height as u32);
            if current_width != new_width || current_height != new_height {
                // emit a resize event
                events.push(Event::new(
                    EventType::Resized {
                        old: size2(current_width, current_height),
                        new: size2(new_width, new_height),
                    },
                    Some(wid),
                ));
            }
        }
        _ => (),
    }

    Ok(events)
}
