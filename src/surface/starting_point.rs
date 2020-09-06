// MIT/Apache2 License

/// The starting point for a window.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StartingPoint {
    XY(i32, i32),
    Center,
}

impl StartingPoint {
    #[inline]
    pub fn to_x_y(
        self,
        my_width: u32,
        my_height: u32,
        parent_width: u32,
        parent_height: u32,
    ) -> (i32, i32) {
        match self {
            Self::XY(x, y) => (x, y),
            Self::Center => (
                (parent_width - my_width) as i32 / 2,
                (parent_height - my_height) as i32 / 2,
            ),
        }
    }
}

impl Default for StartingPoint {
    #[inline]
    fn default() -> Self {
        Self::XY(0, 0)
    }
}
