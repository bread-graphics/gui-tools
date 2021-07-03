// MIT/Apache2 License

use crate::window::Window;
use gluten_keyboard::Key;

/// Sum type representing a possible event emitted by the underlying GUI library.
#[derive(Debug)]
pub enum Event {
    /// No-op event.
    NoOp,
    /// The display is being closed; good time to deallocate resources.
    Destroy,
    /// Special event that tells the `Display` to quit.
    Quit,
    /// A window is being closed.
    Close(Window),
    /// A window is being moved.
    Move { window: Window, x: i32, y: i32 },
    /// A window is being resized.
    Resize {
        window: Window,
        width: u32,
        height: u32,
    },
    /// The window is being activated.
    Activate(Window),
    /// The window is being deactivated.
    Deactivate(Window),
    /// The window is ready to be painted. Use the `draw` function on the `Display` in order to actually
    /// begin painting.
    Paint(Window),
    /// A key is being depressed.
    KeyDown { window: Window, key: Option<Key> },
    /// A key is being released.
    KeyUp { window: Window, key: Option<Key> },
    /// A mouse button is being depressed.
    ButtonDown {
        window: Window,
        button: MouseButton,
        x: i32,
        y: i32,
    },
    /// A mouse button is being released.
    ButtonUp {
        window: Window,
        button: MouseButton,
        x: i32,
        y: i32,
    },
    /// The mouse is being moved.
    MouseMove { window: Window, x: i32, y: i32 },
}

impl Default for Event {
    #[inline]
    fn default() -> Event {
        Event::NoOp
    }
}

/// The mouse button being depressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

impl Event {
    #[inline]
    pub fn is_quit_event(&self) -> bool {
        matches!(self, Event::Quit)
    }
}
