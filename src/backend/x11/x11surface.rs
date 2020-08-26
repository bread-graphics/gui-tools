// MIT/Apache2 License

use super::X11Runtime;
use crate::{
    runtime::Runtime,
    surface::{Surface, SurfaceBackend, SurfaceProperties},
};
use core::{
    convert::TryInto,
    mem::MaybeUninit,
    ops,
    ptr::{self, NonNull},
};
use cty::{c_int, c_long, c_uint, c_ulong};
use x11nas::xlib::{self, Display, Window, XSetWindowAttributes};

pub struct X11Surface {
    // pointer to the display
    // SAFETY: X11Surface is always owned by an X11Runtime. The surface will be dropped
    //         before the display is.
    display: NonNull<Display>,
    // the screen that the surface is on
    screen: c_int,
    // the window ID
    window: Window,
}

impl X11Surface {
    #[inline]
    pub(crate) fn xid(&self) -> Window {
        self.window
    }

    // create from surface properties
    pub(crate) fn new(runtime: &Runtime, props: &SurfaceProperties) -> Self {
        let xruntime = runtime.as_x11().unwrap();
        let screen = &xruntime.monitors()[runtime.default_monitor_index()];
        let root_window = screen.root_window();

        // create the x11 attributes object
        let mut window_attrs: XSetWindowAttributes = XSetWindowAttributes {
            background_pixmap: 0,
            border_pixel: unsafe {
                xlib::XBlackPixel(xruntime.display().as_ptr(), screen.screen_id())
            },
            bit_gravity: xlib::NorthWestGravity,
            colormap: xruntime.default_colormap.unwrap(),
            override_redirect: xlib::True,
            save_under: xlib::True,
            // SAFETY: this isn't UB since only the initialized properties are used by X11,
            ..unsafe { MaybeUninit::uninit().assume_init() }
        };
        let attrs_mask: c_ulong = xlib::CWBackPixmap
            | xlib::CWBorderPixel
            | xlib::CWBitGravity
            | xlib::CWColormap
            | xlib::CWSaveUnder
            | xlib::CWOverrideRedirect;

        let visual = screen.default_visual();

        let window = unsafe {
            xlib::XCreateWindow(
                xruntime.display().as_ptr(),
                match props.parent {
                    None => root_window,
                    Some(parent) => runtime.surface_at(parent).as_x11().unwrap().xid(),
                },
                props.x,
                props.y,
                props.width,
                props.height,
                0,
                visual.depth(),
                xlib::InputOutput as c_uint,
                visual.visual().as_ptr(),
                attrs_mask,
                &mut window_attrs,
            )
        };

        unsafe { xlib::XMapWindow(xruntime.display().as_ptr(), window) };

        let surface = X11Surface {
            screen: screen.screen_id(),
            window,
            display: screen.display().clone(),
        };

        surface
    }
}

impl SurfaceBackend for X11Surface {
    #[inline]
    fn id(&self) -> usize {
        self.window.try_into().unwrap()
    }
}
