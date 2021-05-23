// MIT/Apache2 License

use crate::{
    event::Event,
    screen::{Screen, ScreenIter},
    window::{Visibility, Window, WindowProps},
};

/// The event handler for a display, called whenever the display has an event to handle.
pub type EventHandler = Box<dyn Fn(Event<'_>) -> crate::Result + Send + 'static>;

/// A connection to the server that governs GUI interactions. Dropping is assumed to close the connection.
pub trait Display {
    // screen functions
    /// Get an iterator over the screens provided by this display.
    fn screens(&mut self) -> crate::Result<ScreenIter<'_>>;
    /// Get the default screen.
    fn default_screen(&mut self) -> crate::Result<Screen>;
    /// Get the dimensions for one of the screens.
    #[inline]
    fn screen_dimensions(&mut self, screen: Screen) -> crate::Result<(u32, u32)> {
        let w = self.toplevel_window(screen)?;
        self.window_size(w)
    }

    // window functions
    /// Get the top-level window for a given screen.
    fn toplevel_window(&mut self, screen: Screen) -> crate::Result<Window>;
    /// Get the default top-level window.
    #[inline]
    fn default_toplevel_window(&mut self) -> crate::Result<Window> {
        let s = self.default_screen()?;
        self.toplevel_window(s)
    }

    /// Create a new window, based on the window's properties. Returns an ID that uniquely identifies that
    /// window.
    fn create_window(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        parent: Window,
        props: WindowProps,
    ) -> crate::Result<Window>;
    /// Destroy or delete a window. The window represented by the given ID will no longer be able to be used
    /// in order to preform window actions.
    fn destroy_window(&mut self, window: Window) -> crate::Result;

    /// Get the X coordinate, Y coordinate, width, and height of a given window.
    fn window_geometry(&mut self, window: Window) -> crate::Result<(i32, i32, u32, u32)>;
    /// Get the parent of a given window, or None if it is the top level window.
    fn window_parent(&mut self, window: Window) -> crate::Result<Option<Window>>;

    /// Get the size of the window.
    #[inline]
    fn window_size(&mut self, window: Window) -> crate::Result<(u32, u32)> {
        let (_, _, w, h) = self.window_geometry(window)?;
        Ok((w, h))
    }
    /// Get the coordinates of the window.
    #[inline]
    fn window_coordinates(&mut self, window: Window) -> crate::Result<(i32, i32)> {
        let (x, y, _, _) = self.window_geometry(window)?;
        Ok((x, y))
    }

    /// Set whether or not this window is visible.
    fn window_set_visibility(&mut self, window: Window, visibility: Visibility) -> crate::Result;
    /// Set the geometry (x, y, width, height) for this window.
    fn window_set_geometry(
        &mut self,
        window: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> crate::Result;

    /// Set the size of this window.
    #[inline]
    fn window_set_size(&mut self, window: Window, width: u32, height: u32) -> crate::Result {
        let (x, y) = self.window_coordinates(window)?;
        self.window_set_geometry(window, x, y, width, height)
    }
    /// Set the height of this window.
    #[inline]
    fn window_set_coordinates(&mut self, window: Window, x: i32, y: i32) -> crate::Result {
        let (width, height) = self.window_size(window)?;
        self.window_set_geometry(window, x, y, width, height)
    }

    // events
    fn run_with_boxed_event_handler(&mut self, f: EventHandler) -> crate::Result;
}

/// Display extension trait, containing generic items that can't be put in a "dyn Display".
pub trait DisplayExt {
    /// Enter this display's main loop with the following event handler function.
    fn run<F: Fn(Event<'_>) -> crate::Result + 'static>(self, f: F) -> crate::Result;
}

impl<D: Display> DisplayExt for D {
    #[inline]
    fn run<F: Fn(Event<'_>) -> crate::Result + 'static>(mut self, f: F) -> crate::Result {
        self.run_with_boxed_event_handler(Box::new(f))
    }
}

/// An async version of the Display trait.
#[cfg(feature = "async")]
pub trait AsyncDisplay {}

/// An enum containing several common members of the Display trait, since enum dispatch is faster. Also provides
/// a generic "new" function that instantiates the best Display for the given OS.
pub enum DisplaySum {
    #[cfg(feature = "breadx")]
    Breadx(crate::breadx::BreadxDisplayConnection),
    #[cfg(windows)]
    Yaww(crate::yaww::YawwDisplay),
    Dynamic(Box<dyn Display + Send>),
}

macro_rules! impl_fn_body {
    ($fname: ident, $self: expr, $($aname: ident),*) => {{
        match $self {
            #[cfg(feature = "breadx")]
            Self::Breadx(b) => b.$fname($($aname),*),
            #[cfg(windows)]
            Self::Yaww(y) => y.$fname($($aname),*),
            Self::Dynamic(d) => d.$fname($($aname),*),
        }
    }}
}

impl Display for DisplaySum {
    #[inline]
    fn screens(&mut self) -> crate::Result<ScreenIter<'_>> {
        impl_fn_body!(screens, self,)
    }

    #[inline]
    fn default_screen(&mut self) -> crate::Result<Screen> {
        impl_fn_body!(default_screen, self,)
    }

    #[inline]
    fn screen_dimensions(&mut self, screen: Screen) -> crate::Result<(u32, u32)> {
        impl_fn_body!(screen_dimensions, self, screen)
    }

    #[inline]
    fn toplevel_window(&mut self, screen: Screen) -> crate::Result<Window> {
        impl_fn_body!(toplevel_window, self, screen)
    }

    #[inline]
    fn default_toplevel_window(&mut self) -> crate::Result<Window> {
        impl_fn_body!(default_toplevel_window, self,)
    }

    #[inline]
    fn create_window(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        parent: Window,
        props: WindowProps,
    ) -> crate::Result<Window> {
        impl_fn_body!(create_window, self, x, y, width, height, parent, props)
    }
    #[inline]
    fn destroy_window(&mut self, window: Window) -> crate::Result {
        impl_fn_body!(destroy_window, self, window)
    }
    #[inline]
    fn window_geometry(&mut self, window: Window) -> crate::Result<(i32, i32, u32, u32)> {
        impl_fn_body!(window_geometry, self, window)
    }
    #[inline]
    fn window_parent(&mut self, window: Window) -> crate::Result<Option<Window>> {
        impl_fn_body!(window_parent, self, window)
    }
    #[inline]
    fn window_size(&mut self, window: Window) -> crate::Result<(u32, u32)> {
        impl_fn_body!(window_size, self, window)
    }
    #[inline]
    fn window_coordinates(&mut self, window: Window) -> crate::Result<(i32, i32)> {
        impl_fn_body!(window_coordinates, self, window)
    }
    #[inline]
    fn window_set_visibility(&mut self, window: Window, visibility: Visibility) -> crate::Result {
        impl_fn_body!(window_set_visibility, self, window, visibility)
    }
    #[inline]
    fn window_set_geometry(
        &mut self,
        window: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> crate::Result {
        impl_fn_body!(window_set_geometry, self, window, x, y, width, height)
    }
    #[inline]
    fn window_set_size(&mut self, window: Window, width: u32, height: u32) -> crate::Result {
        impl_fn_body!(window_set_size, self, window, width, height)
    }
    #[inline]
    fn window_set_coordinates(&mut self, window: Window, x: i32, y: i32) -> crate::Result {
        impl_fn_body!(window_set_coordinates, self, window, x, y)
    }
    #[inline]
    fn run_with_boxed_event_handler(&mut self, f: EventHandler) -> crate::Result {
        impl_fn_body!(run_with_boxed_event_handler, self, f)
    }
}
