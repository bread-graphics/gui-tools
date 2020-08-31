// MIT/Apache2 License

use super::{X11Atom, X11Runtime};
use crate::{
    event::EventTypeMask,
    runtime::Runtime,
    surface::{Surface, SurfaceBackend, SurfaceInitialization, SurfaceProperties},
};
use core::{
    convert::TryInto,
    mem::MaybeUninit,
    ops,
    ptr::{self, NonNull},
};
use cty::{c_int, c_long, c_uint, c_ulong};
use x11nas::xlib::{self, Atom, Display, Window, XSetWindowAttributes, XWindowAttributes};

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
        log::trace!("Result of C function call: {}", white_pixel);

        log::trace!("Unsafe code: MaybeUninit for partial initialization of XSetWindowAttributes");
        let mut window_attrs: XSetWindowAttributes = XSetWindowAttributes {
            background_pixel: white_pixel,
            background_pixmap: 0,
            border_pixel: white_pixel,
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

        log::trace!("C function call: XCreateWindow({:p}, {}, {}, {}, {}, {}, 1, {}, xlib::InputOutput, {:p}, {}, {:?})", xruntime.display().as_ptr(), xparent, x, y, props.width, props.height, visual.depth(), visual.visual().as_ptr(), attrs_mask, &window_attrs);
        let window = unsafe {
            xlib::XCreateWindow(
                xruntime.display().as_ptr(),
                xparent,
                x,
                y,
                props.width,
                props.height,
                1,
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

        log::debug!("Mapping the window to ensure it's visible");
        log::trace!(
            "C function call: XMapRaised({:p}, {})",
            xruntime.display().as_ptr(),
            window
        );
        unsafe { xlib::XMapRaised(xruntime.display().as_ptr(), window) };

        let surface = X11Surface {
            screen: screen.screen_id(),
            window,
            display: screen.display().clone(),
        };

        log::debug!("Finished surface initialization, returning...");

        Ok(surface)
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
            EventTypeMask::Clicked => xlib::ButtonPressMask,
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
    unsafe { xlib::XSelectInput(dpy.as_ptr(), window, mask as _) };

    Ok(())
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
}
