// MIT/Apache2 License

use crate::{
    display::{Display, DrawHandler, EventHandler},
    screen::{Screen, ScreenIter},
    surface::SurfaceSum,
    window::{Visibility, Window, WindowProps},
};

/// A version of the `DisplaySum` type that instead takes the form of a reference to a `Display` rather than
/// an owned one.
pub enum DisplaySumRef<'r> {
    #[cfg(feature = "breadx")]
    Breadx(&'r mut crate::breadx::BreadxDisplayConnection),
    #[cfg(windows)]
    Yaww(&'r mut crate::yaww::YawwDisplay),
    Dynamic(&'r mut (dyn Display + Send + 'r)),
}

macro_rules! impl_fn_body {
    ($fname: ident, $self: expr, $($arg: expr),*) => {{
        match $self {
            #[cfg(feature = "breadx")]
            DisplaySumRef::Breadx(b) => b.$fname($($arg),*),
            #[cfg(windows)]
            DisplaySumRef::Yaww(y) => y.$fname($($arg),*),
            DisplaySumRef::Dynamic(d) => d.$fname($($arg),*),
        }
    }}
}

impl<'r> Display for DisplaySumRef<'r> {
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
    fn window_draw_with_boxed_drawer(
        &mut self,
        window: Window,
        draw: DrawHandler<'_>,
    ) -> crate::Result {
        impl_fn_body!(window_draw_with_boxed_drawer, self, window, draw)
    }
    #[inline]
    fn run_with_boxed_event_handler(&mut self, f: EventHandler) -> crate::Result {
        impl_fn_body!(run_with_boxed_event_handler, self, f)
    }
}
