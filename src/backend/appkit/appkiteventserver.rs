// MIT/Apache2 License

use crate::event::Event;
use core::ptr::NonNull;
use objc::{msg_send, sel, class, object::{id, Object}};
use storagevec::StorageVec;

pub struct AppkitEventServer {
    main_loop: NonNull<Object>,
    events: StorageVec<Event, 5>,
}

impl AppkitEventServer {
    #[inline]
    pub fn load() -> crate::Result<Self> {

    }
}
