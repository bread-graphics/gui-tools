// MIT/Apache2 License

use super::X11Atom;
use crate::{
    color::Rgba,
    error::x11_status_to_res,
    event::EventTypeMask,
    geometry::Rectangle,
    graphics::GraphicsInternal,
    runtime::Runtime,
    surface::{SurfaceBackend, SurfaceInitialization},
};
use core::{
    convert::TryInto,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};
use cty::{c_int, c_long, c_uint, c_ulong};
use euclid::{Point2D, Size2D};
use x11nas::xlib::{
    self, Atom, Display, Window, XEvent, XExposeEvent, XGCValues, XSetWindowAttributes,
    XWindowAttributes, XWindowChanges, _XGC, _XIC,
};

pub struct X11Surface {
    // pointer to the display
    // SAFETY: X11Surface is always owned by an X11Runtime. The surface will be dropped
    //         before the display is.
    display: NonNull<Display>,
    // the screen that the surface is on
    screen: c_int,
    // the window ID
    window: Window,
    // pointer to the input context
    input_context: NonNull<_XIC>,
    // pointer to the graphics context
    graphics_context: NonNull<_XGC>,
    // pointer to the runtime
    runtime: Runtime,
}

// helper function to set a text-based property for the window
#[inline]
fn set_text_property(
    dpy: NonNull<Display>,
    wndw: Window,
    property: Atom,
    txt: &str,
) -> crate::Result<()> {
    // NOTE: All of this has a chance of going horribly horrible wrong. X11 has
    //       weird ways of dealing with UTF-8 strings, and Rust strings are natively
    //       UTF-8. I'm going to leave it like this and HOPE that nothing goes wrong
    //       until I can figure out how to do it right.
    const PROP_TYPE: Atom = xlib::XA_STRING;
    const PROP_FORMAT: c_int = 8;

    unsafe {
        xlib::XChangeProperty(
            dpy.as_ptr(),
            wndw,
            property,
            PROP_TYPE,
            PROP_FORMAT,
            xlib::PropModeReplace,
            txt.as_ptr(),
            txt.len() as c_int,
        )
    };

    Ok(())
}

impl X11Surface {
    #[inline]
    pub(crate) fn xid(&self) -> Window {
        self.window
    }

    #[inline]
    pub fn input_context(&self) -> NonNull<_XIC> {
        self.input_context
    }

    // create from surface properties
    pub(crate) fn new(runtime: &Runtime, props: &SurfaceInitialization) -> crate::Result<Self> {
        log::info!("Creating new X11Surface");

        let xruntime = runtime.as_x11().unwrap();
        let screen = &xruntime.monitors()[runtime.default_monitor_index()];
        let root_window = screen.root_window();

        // create the x11 attributes object
        log::debug!("Creating an XSetWindowAttributes object");

        log::trace!(
            "C function call: XWhitePixel({:p}, {})",
            xruntime.display().as_ptr(),
            screen.screen_id()
        );
        let white_pixel =
            unsafe { xlib::XWhitePixel(xruntime.display().as_ptr(), screen.screen_id()) };
        log::trace!(
            "C function call: XBlackPixel({:p}, {})",
            xruntime.display().as_ptr(),
            screen.screen_id()
        );
        let black_pixel =
            unsafe { xlib::XBlackPixel(xruntime.display().as_ptr(), screen.screen_id()) };
        log::trace!("Result of C function call: {}", white_pixel);

        log::trace!("Unsafe code: MaybeUninit for partial initialization of XSetWindowAttributes");
        let mut window_attrs: XSetWindowAttributes = XSetWindowAttributes {
            background_pixel: white_pixel,
            background_pixmap: 0,
            border_pixel: black_pixel,
            bit_gravity: xlib::NorthWestGravity,
            colormap: xruntime.default_colormap.unwrap(),
            //            override_redirect: xlib::True,
            save_under: xlib::True,
            // SAFETY: this isn't UB since only the initialized properties are used by X11,
            ..unsafe { MaybeUninit::uninit().assume_init() }
        };
        let attrs_mask: c_ulong = xlib::CWBackPixmap
            | xlib::CWBackPixel
            | xlib::CWBorderPixel
            | xlib::CWBitGravity
            | xlib::CWColormap
//            | xlib::CWOverrideRedirect
            | xlib::CWSaveUnder;

        let visual = screen.default_visual();

        log::debug!("Creating the actual X11 window");
        let xparent = match props.parent {
            None => root_window,
            Some(parent) => runtime.surface_at(parent).unwrap().as_x11().unwrap().xid(),
        };

        // get the x11 window's width and height
        let mut parent_properties: MaybeUninit<XWindowAttributes> = MaybeUninit::uninit();
        log::trace!(
            "C function call: XGetWindowAttributes({:p}, {}, [buffer])",
            xruntime.display().as_ptr(),
            xparent
        );
        unsafe {
            xlib::XGetWindowAttributes(
                xruntime.display().as_ptr(),
                xparent,
                parent_properties.as_mut_ptr(),
            )
        };

        log::trace!(
            "Unsafe code: Assuming window attributes from XGetWindowAttributes are initialized"
        );
        let XWindowAttributes {
            width: parent_width,
            height: parent_height,
            ..
        } = unsafe { MaybeUninit::assume_init(parent_properties) };

        let (x, y) = props.starting_point.to_x_y(
            props.width,
            props.height,
            parent_width.try_into().unwrap(),
            parent_height.try_into().unwrap(),
        );

        xruntime.push_error_trap();

        /*
        log::trace!(
            "C function call: XCreateWindow({:p}, {}, {}, {}, {}, {}, 1, {}, xlib::InputOutput, {:p}, {}, {:?})",
            xruntime.display().as_ptr(),
            xparent,
            x,
            y,
            props.width,
            props.height,
            visual.depth(),
            visual.visual().as_ptr(),
            attrs_mask,
            &window_attrs
        );
        */
        let window = unsafe {
            xlib::XCreateWindow(
                xruntime.display().as_ptr(),
                xparent,
                x,
                y,
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
        log::trace!("Result of C function call: {}", window);

        xruntime.pop_error_trap()?;

        set_text_property(
            *xruntime.display(),
            window,
            xruntime.internal_atom(X11Atom::WmName),
            &*props.title,
        )?;

        // if there is a mask, set it
        if !props.event_mask.is_empty() {
            set_event_mask(*xruntime.display(), window, &props.event_mask)?;
        }

        // create the input context
        let xic = unsafe {
            xlib::XCreateIC(
                xruntime.input_method().as_ptr(),
                xlib::XNInputStyle_0.as_ptr(),
                xlib::XIMPreeditNothing | xlib::XIMStatusNothing,
                xlib::XNClientWindow_0.as_ptr(),
                window,
                xlib::XNFocusWindow_0.as_ptr(),
                window,
                ptr::null_mut::<c_int>(),
            )
        };
        let input_context = match NonNull::new(xic) {
            Some(xic) => xic,
            None => return Err(crate::X11Error::BadInputContext.into()),
        };

        unsafe { xlib::XSetICFocus(input_context.as_ptr()) };

        // create the GC
        log::debug!("Creating graphics context for window");
        log::trace!(
            "C function call: XCreateGC({:p}, {}, 0, null)",
            xruntime.display().as_ptr(),
            window
        );
        let gc =
            unsafe { xlib::XCreateGC(xruntime.display().as_ptr(), window, 0, ptr::null_mut()) };
        let graphics_context = match NonNull::new(gc) {
            Some(gc) => gc,
            None => return Err(crate::X11Error::BadGraphicsContext.into()),
        };

        log::debug!("Mapping the window to ensure it's visible");
        log::trace!(
            "C function call: XMapRaised({:p}, {})",
            xruntime.display().as_ptr(),
            window
        );
        x11_status_to_res(*xruntime.display(), unsafe {
            xlib::XMapRaised(xruntime.display().as_ptr(), window)
        })?;

        let surface = X11Surface {
            screen: screen.screen_id(),
            window,
            display: screen.display().clone(),
            input_context,
            graphics_context,
            runtime: runtime.clone(),
        };

        log::debug!("Finished surface initialization, returning...");

        Ok(surface)
    }

    #[inline]
    pub fn graphics_context(&self) -> NonNull<_XGC> {
        self.graphics_context
    }

    #[inline]
    pub fn display(&self) -> NonNull<Display> {
        self.display
    }

    #[inline]
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }
}

fn set_event_mask(
    dpy: NonNull<Display>,
    window: Window,
    mask: &[EventTypeMask],
) -> crate::Result<()> {
    const DEFAULT_MASK: c_long = xlib::ExposureMask;

    // table to convert EventTypeMask to an X11 event mask
    fn etm_to_x11(etm: EventTypeMask) -> Option<c_long> {
        Some(match etm {
            EventTypeMask::Resized => xlib::StructureNotifyMask,
            EventTypeMask::MouseDown => xlib::ButtonPressMask,
            EventTypeMask::MouseUp => xlib::ButtonReleaseMask,
            EventTypeMask::MouseEnterWindow => xlib::EnterWindowMask,
            EventTypeMask::MouseExitWindow => xlib::LeaveWindowMask,
            EventTypeMask::MouseMove => xlib::PointerMotionMask,
            EventTypeMask::KeyDown => xlib::KeyPressMask,
            EventTypeMask::KeyUp => xlib::KeyReleaseMask,
            EventTypeMask::Paint => xlib::ExposureMask,
            _ => return None,
        })
    }

    let mask = mask
        .iter()
        .copied()
        .filter_map(etm_to_x11)
        .fold(DEFAULT_MASK, |sum_mask, etm| sum_mask | (etm as c_long));

    log::trace!(
        "C function call: XSelectInput({:p}, {}, {})",
        dpy.as_ptr(),
        window,
        mask
    );
    x11_status_to_res(dpy, unsafe {
        xlib::XSelectInput(dpy.as_ptr(), window, mask as _)
    })?;

    Ok(())
}

macro_rules! set_window_attr {
    ($self: expr, $field: ident, $val: expr, $mask: expr) => {{
        let mut win_attrs = XSetWindowAttributes {
            $field: ($val),
            ..unsafe { MaybeUninit::uninit().assume_init() }
        };

        x11_status_to_res($self.display, unsafe {
            xlib::XChangeWindowAttributes(
                $self.display.as_ptr(),
                $self.window,
                ($mask),
                &mut win_attrs,
            )
        })?;

        Ok(())
    }};
}

impl SurfaceBackend for X11Surface {
    #[inline]
    fn id(&self) -> usize {
        self.window.try_into().unwrap()
    }

    #[inline]
    fn set_event_mask(&self, mask: &[EventTypeMask]) -> crate::Result<()> {
        set_event_mask(self.display, self.window, mask)
    }

    #[inline]
    fn set_size(&self, width: u32, height: u32) -> crate::Result<()> {
        log::trace!(
            "C function call: XResizeWindow({:p}, {}, {}, {})",
            self.display.as_ptr(),
            self.window,
            width,
            height
        );
        x11_status_to_res(self.display, unsafe {
            xlib::XResizeWindow(self.display.as_ptr(), self.window, width, height)
        })?;
        Ok(())
    }

    #[inline]
    fn set_location(&self, x: i32, y: i32) -> crate::Result<()> {
        log::trace!(
            "C function call: XMoveWindow({:p}, {}, {}, {})",
            self.display.as_ptr(),
            self.window,
            x,
            y
        );
        x11_status_to_res(self.display, unsafe {
            xlib::XMoveWindow(self.display.as_ptr(), self.window, x, y)
        })?;
        Ok(())
    }

    #[inline]
    fn set_background_color(&self, clr: Rgba) -> crate::Result<()> {
        // get color ID from runtime
        let clr = self.runtime.as_x11().unwrap().color_id(clr)?;
        x11_status_to_res(self.display, unsafe {
            xlib::XSetWindowBackground(self.display.as_ptr(), self.window, clr)
        })
    }

    #[inline]
    fn set_border_color(&self, clr: Rgba) -> crate::Result<()> {
        let clr = self.runtime.as_x11().unwrap().color_id(clr)?;
        set_window_attr!(self, border_pixel, clr, xlib::CWBorderPixel)
    }

    #[inline]
    fn set_border_width(&self, width: u32) -> crate::Result<()> {
        let mut window_changes = XWindowChanges {
            border_width: width.try_into()?,
            ..unsafe { MaybeUninit::uninit().assume_init() }
        };

        x11_status_to_res(self.display, unsafe {
            xlib::XConfigureWindow(
                self.display.as_ptr(),
                self.window,
                xlib::CWBorderWidth.try_into().unwrap(),
                &mut window_changes,
            )
        })
    }

    #[inline]
    fn graphics_internal(&self) -> crate::Result<NonNull<dyn GraphicsInternal>> {
        // SAFETY: we know Self is non-null
        Ok(unsafe {
            NonNull::new_unchecked(
                self as *const Self as *const dyn GraphicsInternal as *mut dyn GraphicsInternal,
            )
        })
    }

    #[inline]
    fn invalidate(&self, rectangle: Option<Rectangle>) -> crate::Result<()> {
        x11_status_to_res(self.display, unsafe {
            xlib::XClearArea(self.display.as_ptr(), self.window, 0, 0, 0, 0, xlib::True)
        })
    }
}
