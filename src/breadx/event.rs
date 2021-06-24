// MIT/Apache2 License

use super::BreadxDisplay;
use crate::event::Event;
use breadx::{display::{prelude::*, Display}, event::Event as BEvent};

impl<Dpy: Display> BreadxDisplay<Dpy> {
    #[inline]
    pub fn convert_event(&mut self, event: BEvent) -> crate::Result<Option<Event>> {
        match event {
            _ => Ok(None),
        }
    }
}
