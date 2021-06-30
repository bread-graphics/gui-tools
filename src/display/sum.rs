// MIT/Apache2 License

use super::{Display, DrawHandler, EventHandler};
use crate::{Dimensions, Screen, ScreenIter, Window, WindowProps};

#[cfg(unix)]
use crate::breadx::BreadxDisplayConnection;

#[cfg(windows)]
use crate::yaww::YawwDisplay;

/// A sum of the most commonly used implementors of `Display`, arranged in such a way so that it benefits
/// from enum dispatch.
pub enum DisplaySum<'evh> {
    /// We are using `breadx`.
    #[cfg(unix)]
    Breadx(BreadxDisplayConnection),
    /// We are using `yaww`.
    #[cfg(windows)]
    Yaww(YawwDisplay<'evh>),
    /// We are using a generic `Display`.
    Generic(Box<dyn Display<'evh> + Send + Sync + 'evh>),
}

impl<'evh> DisplaySum<'evh> {
    /// Create a new `DisplaySum` using the system's resources.
    #[inline]
    pub fn create() -> crate::Result<Self> {
        crate::init::init()
    }
}

macro_rules! impl_display_sum {
    (fn $fname: ident ( &mut self, $($aname: ident: $aty: ty),* ) -> $retty: ty) => {
        #[inline]
        fn $fname(&mut self, $($aname: $aty),*) -> $retty {
            match self {
                #[cfg(unix)]
                DisplaySum::Breadx(b) => b.$fname($($aname),*),
                #[cfg(windows)]
                DisplaySum::Yaww(y) => y.$fname($($aname),*),
                DisplaySum::Generic(g) => g.$fname($($aname),*),
            }
        }
    };
    (fn $fname: ident ( &mut self, $($aname: ident: $aty: ty),* ) -> $retty: ty, $($tt:tt)*) => {
        impl_display_sum! { fn $fname(&mut self, $($aname: $aty),*) -> $retty }
        impl_display_sum! { $($tt)* }
    };
}

/*
macro_rules! impl_display_sum {
    ($(fn $fname: ident ( &mut self, $($aname: ident: $aty: ty),* ) -> $retty: ty),*) => {
        impl<'evh> Display<'evh> for DisplaySum<'evh> {
            $(
                impl_display_sum_method! {  }
            )*
        }
    }
}
*/

impl<'evh> Display<'evh> for DisplaySum<'evh> {
    impl_display_sum! {
        fn screens(&mut self,) -> crate::Result<ScreenIter<'_>>,
        fn default_screen(&mut self,) -> crate::Result<Screen>,
        fn screen_dimensions(&mut self, screen: Screen) -> crate::Result<(u32, u32)>,
        fn toplevel_window(&mut self, screen: Screen) -> crate::Result<Window>,
        fn create_window(
            &mut self,
            x: i32,
            y: i32,
            width: u32,
            height: u32,
            parent: Window,
            props: WindowProps
        ) -> crate::Result<Window>,
        fn delete_window(&mut self, window: Window) -> crate::Result,
        fn set_window_visibility(&mut self, window: Window, visible: bool) -> crate::Result,
        fn window_dimensions(&mut self, window: Window) -> crate::Result<Dimensions>,
        fn window_coordinates(&mut self, window: Window) -> crate::Result<(i32, i32)>,
        fn window_size(&mut self, window: Window) -> crate::Result<(u32, u32)>,
        fn window_set_dimensions(
            &mut self,
            window: Window,
            x: i32,
            y: i32,
            width: u32,
            height: u32
        ) -> crate::Result,
        fn window_set_coordinates(&mut self, window: Window, x: i32, y: i32) -> crate::Result,
        fn window_set_size(&mut self, window: Window, width: u32, height: u32) -> crate::Result,
        fn draw_with_boxed_draw_handler(&mut self, window: Window, handler: DrawHandler<'_>) -> crate::Result,
        fn window_parent(&mut self, window: Window) -> crate::Result<Option<Window>>,
        fn run_with_boxed_event_handler(&mut self, handler: EventHandler<'evh>) -> crate::Result
    }
}
