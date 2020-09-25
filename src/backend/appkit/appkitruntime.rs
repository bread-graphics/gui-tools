// MIT/Apache2 License

use super::AppkitMonitor;
use crate::{monitor::Monitor, runtime::RuntimeBackend};
use core::{mem::ManuallyDrop, ptr::NonNull};
use objc::{class, msg_send, object::{id, Object}};
use storagevec::StorageVec;

pub struct AppkitRuntime {
    monitors: StorageVec<AppkitMonitor, 5>,
}

impl AppkitRuntime {
    #[inline]
    pub fn new() -> crate::Result<(usize, Self)> {
        // create the application
        let nsapplication_class = class!(NSApplication);
        let _: id = msg_send![nsapplication_class, sharedApplication];

        let (primary, monitors) = AppkitMonitor::glob()?;
        Ok((primary, AppkitRuntime { monitors }))
    }
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl RuntimeBackend for AppkitRuntime {
    #[inline]
    fn monitor_at(&self, monitor: usize) -> Option<&Monitor> {
        self.monitors.get(monitor).map(|m| &*m)
    }

    #[inline] 
    fn serve_event(&self: real: &Runtime) -> crate::Result<StorageVec<Event, 5>> {
        
    }
}
