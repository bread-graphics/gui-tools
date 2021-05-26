// MIT/Apache2 License

use crate::{
    event::Event,
    screen::{Screen, ScreenIter},
    surface::SurfaceSum,
    window::{Visibility, Window, WindowProps},
};

mod sum;
pub use sum::*;

/// The event handler for a display, called whenever the display has an event to handle.
pub type EventHandler = Box<dyn FnMut(DisplaySumRef<'_>, Event) -> crate::Result + Send + 'static>;

/// Handles drawing for a specific window. Requirements are less strict because it should be display-local.
pub type DrawHandler<'a> = Box<dyn FnOnce(&mut SurfaceSum<'_>) -> crate::Result + 'a>;

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
    /// Set the properties associated with this window.
    fn window_set_properties(&mut self, window: Window, props: WindowProps) -> crate::Result;

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

    /// You shouldn't call this directly, instead use the DisplayExt trait's "window_draw" function.
    fn window_draw_with_boxed_drawer(
        &mut self,
        window: Window,
        draw: DrawHandler<'_>,
    ) -> crate::Result;

    // events
    /// You shouldn't call this directly, instead use the DisplayExtOwned trait's "run" function.
    fn run_with_boxed_event_handler(&mut self, f: EventHandler) -> crate::Result;
}

// Display can be implemented on &mut Display as well
impl<'r, D: Display + ?Sized> Display for &'r mut D {
    #[inline]
    fn screens(&mut self) -> crate::Result<ScreenIter<'_>> {
        (**self).screens()
    }
    #[inline]
    fn default_screen(&mut self) -> crate::Result<Screen> {
        (**self).default_screen()
    }
    #[inline]
    fn screen_dimensions(&mut self, screen: Screen) -> crate::Result<(u32, u32)> {
        (**self).screen_dimensions(screen)
    }
    #[inline]
    fn toplevel_window(&mut self, screen: Screen) -> crate::Result<Window> {
        (**self).toplevel_window(screen)
    }
    #[inline]
    fn default_toplevel_window(&mut self) -> crate::Result<Window> {
        (**self).default_toplevel_window()
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
        (**self).create_window(x, y, width, height, parent, props)
    }
    #[inline]
    fn destroy_window(&mut self, window: Window) -> crate::Result {
        (**self).destroy_window(window)
    }
    #[inline]
    fn window_geometry(&mut self, window: Window) -> crate::Result<(i32, i32, u32, u32)> {
        (**self).window_geometry(window)
    }
    #[inline]
    fn window_parent(&mut self, window: Window) -> crate::Result<Option<Window>> {
        (**self).window_parent(window)
    }
    #[inline]
    fn window_size(&mut self, window: Window) -> crate::Result<(u32, u32)> {
        (**self).window_size(window)
    }
    #[inline]
    fn window_coordinates(&mut self, window: Window) -> crate::Result<(i32, i32)> {
        (**self).window_coordinates(window)
    }
    #[inline]
    fn window_set_visibility(&mut self, window: Window, vis: Visibility) -> crate::Result {
        (**self).window_set_visibility(window, vis)
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
        (**self).window_set_geometry(window, x, y, width, height)
    }
    #[inline]
    fn window_set_properties(&mut self, window: Window, props: WindowProps) -> crate::Result {
        (**self).window_set_properties(window, props)
    }
    #[inline]
    fn window_set_size(&mut self, window: Window, width: u32, height: u32) -> crate::Result {
        (**self).window_set_size(window, width, height)
    }
    #[inline]
    fn window_set_coordinates(&mut self, window: Window, x: i32, y: i32) -> crate::Result {
        (**self).window_set_coordinates(window, x, y)
    }
    #[inline]
    fn window_draw_with_boxed_drawer(
        &mut self,
        window: Window,
        draw: DrawHandler<'_>,
    ) -> crate::Result {
        (**self).window_draw_with_boxed_drawer(window, draw)
    }
    #[inline]
    fn run_with_boxed_event_handler(&mut self, f: EventHandler) -> crate::Result {
        (**self).run_with_boxed_event_handler(f)
    }
}

/// Display extension trait, containing generic items that can't be used in dynamic dispatch.
pub trait DisplayExt {
    /// Draw on a window using the given function, which will be provided with a `Surface`. Calling this
    /// outside of a "paint" event handler may have unpredictable results.
    fn window_draw<F: FnOnce(&mut SurfaceSum<'_>) -> crate::Result>(
        &mut self,
        window: Window,
        f: F,
    ) -> crate::Result;
}

impl<D: Display + ?Sized> DisplayExt for D {
    #[inline]
    fn window_draw<F: FnOnce(&mut SurfaceSum<'_>) -> crate::Result>(
        &mut self,
        window: Window,
        f: F,
    ) -> crate::Result {
        self.window_draw_with_boxed_drawer(window, Box::new(f))
    }
}

/// Display extension trait, containing generic items that can't be put in a sized instantiation.
pub trait DisplayExtOwned {
    /// Enter this display's main loop with the following event handler function.
    fn run<F: FnMut(DisplaySumRef<'_>, Event) -> crate::Result + Send + 'static>(
        self,
        f: F,
    ) -> crate::Result;
}

impl<D: Display> DisplayExtOwned for D {
    #[inline]
    fn run<F: FnMut(DisplaySumRef<'_>, Event) -> crate::Result + Send + 'static>(
        mut self,
        f: F,
    ) -> crate::Result {
        self.run_with_boxed_event_handler(Box::new(f))
    }
}

/// An async version of the Display trait.
#[cfg(feature = "async")]
pub trait AsyncDisplay {}
