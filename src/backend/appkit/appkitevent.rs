// MIT/Apache2 License

use crate::{event::Event, runtime::Runtime};
use objc::runtime::{Object, id};
use storagevec::StorageVec;

pub fn appkit_translate_event(runtime: &Runtime, event: id) -> crate::Result<StorageVec<Event, 5>> {
    let mut events = StorageVec::new();
    let event_type = unsafe { msg_send![event type] }; 

    Ok(events)
}
