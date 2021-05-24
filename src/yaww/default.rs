// MIT/Apache2 License

use super::YawwDisplay;
use crate::{display::DisplaySumRef, window::WindowProps};
use flume::Receiver;
use yaww::{event::Event, server::GuiThread, window::Window};

pub(crate) enum EventHandlerUpdate {
    WindowProps(Window, WindowProps),
}

/// Event handler.
pub(crate) fn default_event_handler(
    rx: Receiver<EventHandlerUpdate>,
) -> impl FnMut(GuiThread, Event) -> yaww::Result + 'static {
    // function state
    let mut window_properties = HashMap::new();
    let mut event_handler: Option<
        Box<dyn FnMut(DisplaySumRef<'_>, Event) -> crate::Result + Send>,
    > = None;
    let mut yaww_display_ref: Option<YawwDisplay> = None;

    move |gt, ev| {
        //        let display = YawwDisplay::reconstruct(gt, data)
        Ok(())
    }
}
