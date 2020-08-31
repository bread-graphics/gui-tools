// MIT/Apache2 License

use super::{visual::X11Visual, X11Runtime};
use crate::monitor::Monitor;
use core::{convert::TryInto, ops::Deref, ptr::NonNull};
use cty::c_int;
use storagevec::StorageVec;
use x11nas::xlib::{self, Display, Screen};

pub struct X11Monitor {
    parent_object: Monitor,

    screen: NonNull<Screen>,
    // a reference to the display
    // SAFETY: X11Monitor is stored in the X11Runtime. It will be dropped before the display.
    display: NonNull<Display>,

    // list of visuals, this is initially blank
    // Note: This is unused initially, so it's an option. We can rely on the Option
    //       being a Some() variant
    pub(crate) visuals: Option<StorageVec<X11Visual, 6>>,
    pub(crate) default_visual: Option<usize>,

    // this is actually an optional value
    pub(crate) rgba_visual: Option<usize>,

    screen_id: c_int,
    root_window: xlib::Window,
}

impl X11Monitor {
    pub fn new(
        runtime: &mut X11Runtime,
        screen_id: c_int,
        setup_runtime_vis: bool,
    ) -> crate::Result<Self> {
        log::info!("Initializing X11 monitor #{}", screen_id);

        // get pointer to screen
        // SAFETY: we verify the results of the operation after we call it
        //         with the NonNull constructor
        log::debug!("Getting pointer to screen of ID {}", screen_id);
        log::trace!(
            "C function call: XScreenOfDisplay({:p}, {})",
            runtime.display().as_ptr(),
            screen_id
        );
        let screen = match NonNull::new(unsafe {
            xlib::XScreenOfDisplay(runtime.display().as_ptr(), screen_id)
        }) {
            Some(screen) => {
                log::trace!("Result of C function call: {:p}", screen);
                screen
            }
            None => {
                log::trace!("Result of C function call: null");
                return Err(crate::X11Error::BadScreenId(screen_id).into());
            }
        };

        // get properties of the screen
        // SAFETY: the data returned here should be A-OK
        log::debug!("Getting the width and height of monitor {}", screen_id);
        log::trace!("C function call: XWidthOfScreen({:p})", screen.as_ptr());
        let width = unsafe { xlib::XWidthOfScreen(screen.as_ptr()) }
            .try_into()
            .unwrap();
        log::trace!("Result of C function call: {}", width);

        log::trace!("C function call: XHeightOfScreen({:p})", screen.as_ptr());
        let height = unsafe { xlib::XHeightOfScreen(screen.as_ptr()) }
            .try_into()
            .unwrap();
        log::trace!("Result of C function call: {}", height);

        log::debug!("Getting the root window of monitor {}", screen_id);
        log::trace!(
            "C function call: XRootWindowOfScreen({:p})",
            screen.as_ptr()
        );
        let root_window = unsafe { xlib::XRootWindowOfScreen(screen.as_ptr()) };
        log::trace!("Result of C function call: {}", root_window);

        log::debug!("Initializing X11Monitor struct");
        let mut monitor = Self {
            parent_object: Monitor::new(width, height),
            display: runtime.display().clone(),
            screen,
            screen_id,
            root_window,
            visuals: None,
            default_visual: None,
            rgba_visual: None,
        };

        log::debug!("Setting up X11Monitor visual members");
        X11Visual::setup_monitor(&mut monitor)?;

        // if this is the default visual, set up the runtime's defaults
        if setup_runtime_vis {
            log::debug!("Setting up runtime visual defaults");

            match monitor.rgba_visual {
                Some(rgba_visual) => {
                    log::debug!("Found an RGBA visual... Creating new colormap...");
                    let rgba_visual = &monitor.visuals.as_ref().unwrap()[rgba_visual];

                    // alloc a new colormap
                    // note: this seems to be the source of an error, let's not
                    //       do it for now
                    /*
                    log::trace!(
                        "C function call: XCreateColormap({:p}, {}, {:p}, xlib::AllocNone)",
                        runtime.display().as_ptr(),
                        root_window,
                        rgba_visual.visual().as_ptr()
                    );
                    let colormap = unsafe {
                        xlib::XCreateColormap(
                            runtime.display().as_ptr(),
                            root_window,
                            rgba_visual.visual().as_ptr(),
                            xlib::AllocNone,
                        )
                    };
                    log::trace!("Result of C function call: {}", colormap);
                    */

                    log::trace!(
                        "C function call: XDefaultColormap({:p}, {})",
                        runtime.display().as_ptr(),
                        monitor.screen_id
                    );
                    let colormap = unsafe {
                        xlib::XDefaultColormap(runtime.display().as_ptr(), monitor.screen_id)
                    };
                    log::trace!("Result of C function call: {}", colormap);

                    runtime.default_colormap = Some(colormap);
                    runtime.default_visual = Some(rgba_visual.visual().clone());
                    runtime.default_depth = Some(rgba_visual.depth());
                }
                _ => {
                    runtime.default_colormap = Some(unsafe {
                        xlib::XDefaultColormap(runtime.display().as_ptr(), screen_id)
                    });
                    runtime.default_visual = Some(
                        match NonNull::new(unsafe {
                            xlib::XDefaultVisual(runtime.display().as_ptr(), screen_id)
                        }) {
                            Some(def) => def,
                            None => return Err(crate::X11Error::BadDefaultVisual.into()),
                        },
                    );
                    runtime.default_depth =
                        Some(unsafe { xlib::XDefaultDepth(runtime.display().as_ptr(), screen_id) });
                }
            }
        }

        log::debug!("Monitor is initialized, returning...");
        Ok(monitor)
    }

    #[inline]
    pub(crate) fn display(&self) -> &NonNull<Display> {
        &self.display
    }

    #[inline]
    pub(crate) fn screen(&self) -> &NonNull<Screen> {
        &self.screen
    }

    #[inline]
    pub(crate) fn screen_id(&self) -> c_int {
        self.screen_id
    }

    #[inline]
    pub(crate) fn root_window(&self) -> xlib::Window {
        self.root_window
    }

    #[inline]
    pub(crate) fn visuals(&self) -> &[X11Visual] {
        self.visuals.as_deref().unwrap()
    }

    #[inline]
    pub(crate) fn default_visual(&self) -> &X11Visual {
        &self.visuals()[self.default_visual.unwrap()]
    }
}

impl Deref for X11Monitor {
    type Target = Monitor;

    fn deref(&self) -> &Self::Target {
        &self.parent_object
    }
}
