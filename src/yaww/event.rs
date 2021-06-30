// MIT/Apache2 License

use super::YawwDisplay;
use crate::event::Event;
use yaww::Event as YawwEvent;

#[inline]
pub(crate) fn cvt_event<'evh>(
    yaww: &YawwDisplay<'evh>,
    event: YawwEvent,
) -> crate::Result<Option<Event>> {
    match event {
        _ => Ok(None),
    }
}
