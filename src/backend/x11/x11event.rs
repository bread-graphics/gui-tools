// MIT/Apache2 License

use super::{X11Atom, X11Runtime, X11Surface};
use crate::{
    event::{Event, EventType},
    graphics::Graphics,
    keyboard::{KeyInfo, KeyType},
    mouse::MouseButton,
    runtime::Runtime,
    surface::SurfaceBackend,
};
use core::convert::TryInto;
use cty::{c_char, c_int};
use euclid::{point2, size2};
use storagevec::StorageVec;
use x11nas::xlib::{self, Atom, KeySym, Status, Window, XEvent, XKeyEvent};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

// figures out the window of an event
// necessary because the sending window of an event might not be
// in the same field every time
#[inline]
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

// helper function to pull the keysym out of the keyevent
// assumes the key modifier masks have already been handled
#[inline]
fn translate_key(xkey: &mut XKeyEvent, x11s: &X11Surface) -> crate::Result<KeySym> {
    enum StrBuffer {
        Array([c_char; 32]),
        #[cfg(feature = "alloc")]
        Vector(Vec<c_char>),
    }

    impl StrBuffer {
        #[inline]
        fn as_mut_ptr(&mut self) -> *mut c_char {
            match self {
                Self::Array(ref mut a) => a.as_mut_ptr(),
                #[cfg(feature = "alloc")]
                Self::Vector(ref mut v) => v.as_mut_ptr(),
            }
        }

        #[cfg(not(feature = "alloc"))]
        #[inline]
        fn grow_to(&mut self, size: usize) -> crate::Result<()> {
            Err(crate::Error::CoreUnsupported(
                "Unable to take IME input without the allocator library",
            ))
        }

        #[cfg(feature = "alloc")]
        #[inline]
        fn grow_to(&mut self, size: usize) -> crate::Result<()> {
            match self {
                Self::Array(_) => {
                    *self = Self::Vector(Vec::with_capacity(size));
                }
                Self::Vector(ref mut v) => v.reserve(size.checked_sub(v.capacity()).unwrap()),
            }

            Ok(())
        }
    }

    let mut buffer = StrBuffer::Array([0; 32]);
    let mut buffer_len = 32;
    let mut keysym: KeySym = xlib::NoSymbol as KeySym;
    let mut status: Status = 0;

    xkey.state &= !xlib::ControlMask; // make sure we don't get any control codes

    // lookup the string
    loop {
        unsafe {
            xlib::Xutf8LookupString(
                x11s.input_context().as_ptr(),
                xkey,
                buffer.as_mut_ptr(),
                buffer_len as c_int - 1,
                &mut keysym,
                &mut status,
            )
        };

        // if the buffer overflowed, redo this operation but with a more flexible buffer
        match status {
            xlib::XBufferOverflow => {
                buffer_len *= 2;
                buffer.grow_to(buffer_len)?;
            }
            xlib::XLookupChars | xlib::XLookupNone => {
                // we don't want chars
                // keysym should be NoSymbol, just use that
                break Ok(keysym);
            }
            xlib::XLookupKeySym | xlib::XLookupBoth => {
                break Ok(keysym);
            }
            _ => panic!("Unexpected Xutf8LookupString status"),
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
                surface.set_size_no_backend(new_width, new_height);
                // emit a resize event
                events.push(Event::new(
                    EventType::Resized {
                        old: size2(current_width, current_height),
                        new: size2(new_width, new_height),
                    },
                    Some(wid),
                ));
            }

            let (current_x, current_y) = surface.location();
            let (new_x, new_y) = (xce.x, xce.y);
            if current_x != new_x || current_y != new_y {
                surface.set_location_no_backend(new_x, new_y);
                // emit a move event
                events.push(Event::new(
                    EventType::Moved {
                        old: point2(current_x, current_y),
                        new: point2(new_x, new_y),
                    },
                    Some(wid),
                ));
            }
        }
        xlib::ButtonPress | xlib::ButtonRelease => {
            log::debug!("Found either a ButtonPress or ButtonRelease event");
            log::trace!("Unsafe code: Casting the XEvent to an XButtonEvent");
            let xbe = unsafe { xev.button };

            let point_of_click = point2(xbe.x, xbe.y);
            let button = match xbe.button {
                xlib::Button1 => MouseButton::Button1,
                xlib::Button2 => MouseButton::Button2,
                xlib::Button3 => MouseButton::Button3,
                xlib::Button4 | xlib::Button5 => {
                    // this is actually the scroll wheel, I think
                    unimplemented!()
                }
                _ => {
                    log::error!("Invalid button: {}", xbe.button);
                    return Ok(events);
                }
            };

            match ty {
                xlib::ButtonPress => events.push(Event::new(
                    EventType::MouseDown(point_of_click, button),
                    Some(wid),
                )),
                xlib::ButtonRelease => events.push(Event::new(
                    EventType::MouseUp(point_of_click, button),
                    Some(wid),
                )),
                _ => (),
            }
        }
        xlib::EnterNotify | xlib::LeaveNotify => {
            log::debug!("Found either a EnterNotify or LeaveNotify event");
            log::trace!("Unsafe code: Casting the XEvent to an XCrossingEvent");
            let xce = unsafe { xev.crossing };

            let crossing_point = point2(xce.x, xce.y);
            match ty {
                xlib::EnterNotify => events.push(Event::new(
                    EventType::MouseEnterWindow(crossing_point),
                    Some(wid),
                )),
                xlib::LeaveNotify => events.push(Event::new(
                    EventType::MouseExitWindow(crossing_point),
                    Some(wid),
                )),
                _ => unreachable!(),
            }
        }
        xlib::MotionNotify => {
            log::debug!("Found a MotionNotify event");
            log::trace!("Unsafe code: Casting the XEvent to the XMotionEvent");
            let xme = unsafe { xev.motion };

            let motion_point = point2(xme.x, xme.y);
            events.push(Event::new(EventType::MouseMove(motion_point), Some(wid)));
        }
        xlib::KeyPress | xlib::KeyRelease => {
            log::debug!("Found a KeyPress or KeyRelease event");
            log::trace!("Unsafe code: Casting the XEvent to the XKeyEvent");
            let mut xke = unsafe { xev.key };
            let surface = surface.unwrap();

            let ctrl = xke.state | xlib::ControlMask != 0;
            let shift = xke.state | xlib::ShiftMask != 0;

            let keysym = match translate_key(&mut xke, surface.as_x11().unwrap()) {
                Ok(keysym) => keysym,
                Err(err) => {
                    log::error!("Unable to translate key event: {}", err);
                    return Ok(events);
                }
            };
            let keytype = KeyType::from_x11(keysym);

            let mut keyinfo = KeyInfo::new(keytype);
            keyinfo.set_ctrl(ctrl);
            keyinfo.set_shift(shift);

            let mouse_location = point2(xke.x, xke.y);
            events.push(Event::new(
                match ty {
                    xlib::KeyPress => EventType::KeyDown(keyinfo, mouse_location),
                    xlib::KeyRelease => EventType::KeyUp(keyinfo, mouse_location),
                    _ => unreachable!(),
                },
                Some(wid),
            ));
        }
        xlib::Expose => {
            log::debug!("Found an Expose event");

            let surface = surface.unwrap();
            events.push(Event::new(
                EventType::Paint(Graphics::new(
                    surface.as_x11().unwrap().graphics_internal()?,
                )),
                Some(wid),
            ));
        }
        xlib::ClientMessage => {
            log::debug!("Found a ClientMessage event");
            log::trace!("Unsafe code: Casting the XEvent to the XClientMessageEvent");
            let xcme = unsafe { xev.client_message };

            if AsRef::<[Atom]>::as_ref(&xcme.data)[0]
                == runtime.internal_atom(X11Atom::WmDeleteWindow)
            {
                let mut ev = Event::new(EventType::Quit, None);
                ev.set_is_terminator(true);
                events.push(ev);
            }
        }
        _ => (),
    }

    Ok(events)
}
