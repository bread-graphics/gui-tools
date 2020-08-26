// MIT/Apache2 License

use super::Event;
use crate::backend::x11::X11Runtime;
use x11nas::xlib::XEvent;

/// Translate an X11 event to a Gui-Tools event.
pub(crate) fn translate_x11_event(runtime: &X11Runtime, xev: XEvent) -> Event {
    unimplemented!()
}
