// MIT/Apache2 License

/// The physical monitor.
#[derive(Default)]
pub struct Monitor {
    // dimensions of this monitor
    width: u32,
    height: u32,
}

impl Monitor {
    #[inline]
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Get the width of this monitor.
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height of this monitor.
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the size of the monitor.
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
