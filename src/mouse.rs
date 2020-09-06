// MIT/Apache2 License

/// Mouse buttons.
#[derive(Copy, Clone, Debug)]
pub enum MouseButton {
    Button1,
    Button2,
    Button3,
    Button4,
    Button5,
}

/// The direction of a scroll button.
#[derive(Copy, Clone, Debug)]
pub enum ScrollWheelDirection {
    Up,
    Down,
}
