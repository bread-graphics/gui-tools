// MIT/Apache2 License

use crate::monitor::Monitor;
use core::{convert::TryInto, ops::Deref};
use winapi::um::winuser::{self, MONITORINFO};

#[derive(Default)]
pub struct Win32Monitor {
    monitor: Monitor,
    primary: bool,
}

impl Win32Monitor {
    #[inline]
    pub fn new(info: MONITORINFO) -> crate::Result<Self> {
        let (width, height) = (
            info.rcWork.right - info.rcWork.left,
            info.rcWork.bottom - info.rcWork.top,
        );
        let primary = (info.dwFlags & winuser::MONITORINFOF_PRIMARY) != 0;
        Ok(Self {
            monitor: Monitor::new(width.try_into()?, height.try_into()?),
            primary,
        })
    }

    #[inline]
    pub(crate) fn from_raw(width: u32, height: u32, primary: bool) -> Self {
        Self {
            monitor: Monitor::new(width, height),
            primary,
        }
    }

    #[inline]
    pub fn primary(&self) -> bool {
        self.primary
    }
}

impl Deref for Win32Monitor {
    type Target = Monitor;

    #[inline]
    fn deref(&self) -> &Monitor {
        &self.monitor
    }
}
