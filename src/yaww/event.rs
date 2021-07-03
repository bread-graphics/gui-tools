// MIT/Apache2 License

use super::{cvt_window_r, YawwDisplay};
use crate::event::Event;
use yaww::Event as YawwEvent;

#[inline]
pub(crate) fn cvt_event<'evh>(
    yaww: &YawwDisplay<'evh>,
    event: YawwEvent,
) -> crate::Result<Option<Event>> {
    Ok(Some(match event {
        YawwEvent::Quit => Event::Quit,
        YawwEvent::Move { window, x, y } => Event::Move { window: cvt_window_r(window), x, y },
        _ => return Ok(None),
    }))
}
