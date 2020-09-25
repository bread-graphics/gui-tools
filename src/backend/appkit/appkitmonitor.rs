// MIT/Apache2 License

use super::CGFloat;
use crate::{geometry::Pixel, monitor::Monitor};
use core::ptr::NonNull;
use cty::c_ulong;
use euclid::Rect;
use objc::{class, msg_send, sel, object::{Object, id}};
use storagevec::StorageVec;

pub struct AppkitMonitor {
    monitor: Monitor,
    handle: NonNull<Object>,
}

impl AppkitMonitor {
    pub fn glob() -> crate::Result<(usize, StorageVec<AppkitMonitor, 5>)> {
        // get the monitor array and its lenght
        let monitor_class = class!(NSScreen);
        let monitors: id = msg_send![monitor_class screens];
        let monitors_len: c_ulong = msg_send![monitors count];
         
        // get the pointer to the main monitor
        let primary_monitor: id = msg_send![monitor_class mainScreen];
        
        // iterate over and glob stats
        let mut primary = 0;
        let m = (0..monitors_len).iter().map(|i| {
            // get the monitor pointer
            let monitor: id = msg_send![monitors objectAtIndex:i];
            if monitor == primary_monitor { primary = i; }
            
            // query size
            // note: euclid::Rect<CGFloat> is repr(C) and thus has the same layout as an NSRect
            let size: Rect<CGFloat, Pixel> = msg_send![monitor visibleFrame];
            let size = size.size;

            AppkitMonitor { handle: NonNull::new(monitor).unwrap(), handle: Monitor::new(size.width as _, size.height as _) }
        }).collect::<StorageVec<AppkitMonitor, 5>>();

        Ok((primary, m))
    }
}

impl Deref for AppkitMonitor {
    type Target = Monitor;

    #[inline]
    fn deref(&self) -> &Monitor { &self.monitor }
}
