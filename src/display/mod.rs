// MIT/Apache2 License

//! This module defines the "display", which represents either the connection to the server or the thread in
//! which the server is running. Most operations in the `gui-tools` crate take place using the `Display` trait.

use super::{Dimensions, Event, Screen, ScreenIter, Window, WindowProps};
use chalkboard::surface::Surface;

mod sum;
pub use sum::*;

/// A boxed event handler.
///
/// TODO: somehow avoid dynamic dispatch here, since the given reference is !Send
pub type EventHandler<'evh> =
    Box<dyn FnMut(&mut dyn Display<'evh>, Event) -> crate::Result + Send + 'evh>;

/// A boxed drawing handler.
pub type DrawHandler<'draw> = Box<dyn FnOnce(&mut dyn Surface) -> crate::Result + 'draw>;

/// Represents a connection to the server. Most GUI operations take place using this object.
///
/// The lifetime represents the lifetime for the event handler.
pub trait Display<'evh> {
    /* Screen Functions */

    /// Returns an iterator that provides a list of `Screen`s to the user. The `Screen`s represent the logical
    /// screens that the `Display` has access to.
    fn screens(&mut self) -> crate::Result<ScreenIter<'_>>;
    /// Returns the default screen to the user. This screen is considered the "primary" screen, where windows
    /// should be spawned.
    fn default_screen(&mut self) -> crate::Result<Screen>;
    /// Get the dimensions for a specific screen.
    #[inline]
    fn screen_dimensions(&mut self, screen: Screen) -> crate::Result<(u32, u32)> {
        let window = self.toplevel_window(screen)?;
        self.window_size(window)
    }

    /* Window Creation and Deletion Functions */

    /// Get the toplevel window for a specific `Screen`.
    fn toplevel_window(&mut self, screen: Screen) -> crate::Result<Window>;
    /// Create a new window.
    fn create_window(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        parent: Window,
        props: WindowProps,
    ) -> crate::Result<Window>;
    /// Delete a window.
    fn delete_window(&mut self, window: Window) -> crate::Result;

    /* Window Geometry Functions */

    /// Get the dimensions of a window.
    fn window_dimensions(&mut self, window: Window) -> crate::Result<Dimensions>;
    /// Get the coordinates of a window.
    #[inline]
    fn window_coordinates(&mut self, window: Window) -> crate::Result<(i32, i32)> {
        let Dimensions { x, y, .. } = self.window_dimensions(window)?;
        Ok((x, y))
    }
    /// Get the size of a window.
    #[inline]
    fn window_size(&mut self, window: Window) -> crate::Result<(u32, u32)> {
        let Dimensions { width, height, .. } = self.window_dimensions(window)?;
        Ok((width, height))
    }

    /// Set the dimensions of a window.
    fn window_set_dimensions(
        &mut self,
        window: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> crate::Result;
    /// Set the coordinates of a window.
    #[inline]
    fn window_set_coordinates(&mut self, window: Window, x: i32, y: i32) -> crate::Result {
        let (width, height) = self.window_size(window)?;
        self.window_set_dimensions(window, x, y, width, height)
    }
    /// Set the size of a window.
    #[inline]
    fn window_set_size(&mut self, window: Window, width: u32, height: u32) -> crate::Result {
        let (x, y) = self.window_coordinates(window)?;
        self.window_set_dimensions(window, x, y, width, height)
    }

    /* Window Drawing Functions */

    /// Draw on a window. It is preferred to use `DisplayExt::draw` instead.
    fn draw_with_boxed_draw_handler(
        &mut self,
        window: Window,
        handler: DrawHandler<'_>,
    ) -> crate::Result;

    /* Window Misc. Functions */

    /// Get the parent for a certain window.
    fn window_parent(&mut self, window: Window) -> crate::Result<Option<Window>>;

    /* Misc. Functions */

    /// Begin running the event handler. It is preferred to use `DisplayExtOwned::run` instead. Running this
    /// function twice has an undefined result, but will most likely error out.
    fn run_with_boxed_event_handler(&mut self, handler: EventHandler<'evh>) -> crate::Result;
}

// Since all the methods take &mut self, we can implement Display for any &mut Display
impl<'evh, D: Display<'evh> + ?Sized> Display<'evh> for &mut D {
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
    fn delete_window(&mut self, window: Window) -> crate::Result {
        (**self).delete_window(window)
    }
    #[inline]
    fn window_dimensions(&mut self, window: Window) -> crate::Result<Dimensions> {
        (**self).window_dimensions(window)
    }
    #[inline]
    fn window_coordinates(&mut self, window: Window) -> crate::Result<(i32, i32)> {
        (**self).window_coordinates(window)
    }
    #[inline]
    fn window_size(&mut self, window: Window) -> crate::Result<(u32, u32)> {
        (**self).window_size(window)
    }
    #[inline]
    fn window_set_dimensions(
        &mut self,
        window: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> crate::Result {
        (**self).window_set_dimensions(window, x, y, width, height)
    }
    #[inline]
    fn window_set_coordinates(&mut self, window: Window, x: i32, y: i32) -> crate::Result {
        (**self).window_set_coordinates(window, x, y)
    }
    #[inline]
    fn window_set_size(&mut self, window: Window, width: u32, height: u32) -> crate::Result {
        (**self).window_set_size(window, width, height)
    }
    #[inline]
    fn draw_with_boxed_draw_handler(
        &mut self,
        window: Window,
        handler: DrawHandler<'_>,
    ) -> crate::Result {
        (**self).draw_with_boxed_draw_handler(window, handler)
    }
    #[inline]
    fn window_parent(&mut self, window: Window) -> crate::Result<Option<Window>> {
        (**self).window_parent(window)
    }
    #[inline]
    fn run_with_boxed_event_handler(&mut self, handler: EventHandler<'evh>) -> crate::Result {
        (**self).run_with_boxed_event_handler(handler)
    }
}

/// Extension trait for the `Display`.
pub trait DisplayExt {
    /// Draw using the specified draw handler. This is meant to be called inside of a draw event handler; calling
    /// it outside of one may have undefined consequences.
    fn draw<F: FnOnce(&mut dyn Surface) -> crate::Result>(
        &mut self,
        window: Window,
        draw_handler: F,
    ) -> crate::Result;
}

impl<'evh, D: Display<'evh> + ?Sized> DisplayExt for D {
    #[inline]
    fn draw<F: FnOnce(&mut dyn Surface) -> crate::Result>(
        &mut self,
        window: Window,
        draw_handler: F,
    ) -> crate::Result {
        self.draw_with_boxed_draw_handler(window, Box::new(draw_handler))
    }
}

/// Extension trait for owned instances of `Display`.
pub trait DisplayExtOwned<'evh>: Sized {
    /// Run the program using the specified event handler.
    fn run<F: FnMut(&mut dyn Display<'evh>, Event) -> crate::Result + Send + 'evh>(
        self,
        run_handler: F,
    ) -> crate::Result;
}

impl<'evh, D: Display<'evh>> DisplayExtOwned<'evh> for D {
    #[inline]
    fn run<F: FnMut(&mut dyn Display<'evh>, Event) -> crate::Result + Send + 'evh>(
        mut self,
        run_handler: F,
    ) -> crate::Result {
        self.run_with_boxed_event_handler(Box::new(run_handler))
    }
}
