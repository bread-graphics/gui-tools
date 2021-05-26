// MIT/Apache2 License

use crate::event::Event as GtEvent;
use yaww::{event::Event, server::GuiThread};

#[inline]
pub(crate) fn convert_yaww_event(ev: Event, gt: &GuiThread) -> crate::Result<Event> {
    unimplemented!()
}
