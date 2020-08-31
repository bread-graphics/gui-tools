// MIT/Apache2 License

use super::{X11ErrorTrap, X11Monitor, X11Surface};
use crate::{
    backend::SurfaceInner,
    error::X11Error,
    event::Event,
    monitor::Monitor,
    mutex::{MutexGuard, ShimMutex as Mutex},
    runtime::{Runtime, RuntimeBackend},
    surface::{Surface, SurfaceProperties},
};
use core::{
    convert::TryInto,
    mem::{self, MaybeUninit},
    ptr::{self, NonNull},
};
use cstr_core::{c_char, CStr};
use cty::c_int;
use storagevec::{StorageMap, StorageVec};
use x11nas::xlib::{self, Atom, Colormap, Display, Visual, Window as WindowID};

#[cfg(feature = "async")]
use core::task::Waker;
#[cfg(feature = "async")]
use futures_lite::Stream;
#[cfg(feature = "async")]
use std::{sync::mpsc, thread};

#[derive(Copy, Clone)]
#[repr(usize)]
pub enum X11Atom {
    WmDeleteWindow = 0,
    WmName,
}

const X11_ATOMS_LEN: usize = 2;
const X11_ATOMS: [X11Atom; X11_ATOMS_LEN] = [X11Atom::WmDeleteWindow, X11Atom::WmName];
const X11_ATOMS_NAMES: [*const c_char; X11_ATOMS_LEN] = [
    // SAFETY: all of these are valid CStrings
    b"WM_DELETE_WINDOW\0" as *const _ as *const c_char,
    b"WM_NAME\0" as *const _ as *const c_char,
];

/// Runtime components necessary for the X11 portion of the program.
pub struct X11Runtime {
    // connection to the X11 server
    display: NonNull<Display>,

    // list of monitors currently associated with this display
    monitors: StorageVec<X11Monitor, 2>,

    // the color map that should be the default
    pub(crate) default_colormap: Option<Colormap>,

    // default visual
    pub(crate) default_visual: Option<NonNull<Visual>>,

    // default depth
    pub(crate) default_depth: Option<c_int>,

    // list of X11 internal atoms
    internal_atoms: [Atom; X11_ATOMS_LEN],

    // trap for errors
    error_trap: Mutex<X11ErrorTrap>,
}

impl X11Runtime {
    // create a new display
    pub fn new() -> crate::Result<(usize, Self)> {
        log::info!("Creating a new X11 runtime");

        log::trace!("C function call: XInitThreads()");
        unsafe { xlib::XInitThreads() };

        log::debug!("Opening up the X11 display connection");
        log::trace!("C function call: XOpenDisplay(null)");
        // try to load the default display connection
        // SAFETY: calling a C function whose results we check
        let display_ptr = unsafe { xlib::XOpenDisplay(ptr::null()) };
        // try to create a NonNull container
        // note that NonNull::new() fails with None if the pointer is null
        let display = match NonNull::new(display_ptr) {
            Some(dpy) => dpy,
            None => return Err(X11Error::DisplayDidntOpen.into()),
        };

        // query for internal atoms
        log::debug!("Querying for internal atoms");
        let mut internal_atoms: [MaybeUninit<Atom>; X11_ATOMS_LEN] = MaybeUninit::uninit_array();

        log::trace!(
            "C function call: XInternAtoms({:p}, {:?}, {}, 1, {:p})",
            display.as_ptr(),
            X11_ATOMS_NAMES,
            X11_ATOMS_LEN,
            internal_atoms.as_ptr()
        );
        let status = unsafe {
            xlib::XInternAtoms(
                display.as_ptr(),
                X11_ATOMS_NAMES.as_ptr() as *mut *mut c_char,
                X11_ATOMS_LEN as c_int,
                1,
                internal_atoms.as_mut_ptr() as *mut Atom,
            )
        };
        log::trace!("Result of C function call: {}", status);

        if status == 0 {
            return Err(X11Error::BadAtoms.into());
        }

        log::debug!("Getting the default screen");
        log::trace!("C function call: XDefaultScreen({:p})", display.as_ptr());
        // SAFETY: same as below
        let default_screen: c_int = unsafe { xlib::XDefaultScreen(display.as_ptr()) };
        log::trace!("Result of C function call: {}", default_screen);

        // create the runtime to put items into
        log::debug!("Initializing X11Runtime struct");
        log::trace!("Unsafe Code: Transmute MaybeUninit array into init array");
        let mut runtime = X11Runtime {
            display,
            // SAFETY: internal_atoms are guaranteed to be valid atoms if
            //         XInternAtoms did not fail, since MaybeUninit<T> has
            //         the same layout of T
            internal_atoms: unsafe { mem::transmute::<_, [Atom; X11_ATOMS_LEN]>(internal_atoms) },
            monitors: StorageVec::new(),
            default_colormap: None,
            default_visual: None,
            default_depth: None,
            error_trap: Mutex::new(X11ErrorTrap::new()),
        };

        // SAFETY: C function that returns an integer, we check it for validity
        log::debug!("Getting the monitor count");
        log::trace!(
            "C function call: XScreenCount({:p})",
            runtime.display.as_ptr()
        );
        let monitor_count = unsafe { xlib::XScreenCount(runtime.display.as_ptr()) };
        log::trace!("Result of C function call: {}", monitor_count);

        if monitor_count <= 0 {
            // a display should still have monitors if it initialized properly
            panic!("Unexpected monitor count: {}", monitor_count);
        }

        // create a new monitor for every screen
        log::debug!("Initializing monitor collection");
        let i = (0..monitor_count)
            .into_iter()
            .map(|id| X11Monitor::new(&mut runtime, id, id == default_screen))
            .collect::<crate::Result<StorageVec<X11Monitor, 2>>>()?;
        runtime.monitors.extend(i);

        log::debug!("X11Runtime initialization finished");
        Ok((default_screen.try_into().unwrap(), runtime))
    }

    #[inline]
    pub(crate) fn display(&self) -> &NonNull<Display> {
        &self.display
    }

    // list monitors
    #[inline]
    pub(crate) fn monitors(&self) -> &[X11Monitor] {
        &self.monitors
    }

    #[inline]
    pub fn error_trap_mut(&self) -> MutexGuard<'_, X11ErrorTrap> {
        self.error_trap.lock()
    }

    #[inline]
    pub fn push_error_trap(&self) {
        self.error_trap.lock().push();
    }

    #[inline]
    pub fn pop_error_trap(&self) -> crate::Result<()> {
        log::trace!(
            "C function call: XSync({:p}, xlib::False)",
            self.display.as_ptr()
        );
        unsafe { xlib::XSync(self.display.as_ptr(), xlib::False) };
        self.error_trap.lock().pop(self.display)
    }

    #[inline]
    pub fn internal_atom(&self, name: X11Atom) -> xlib::Atom {
        self.internal_atoms[name as usize]
    }
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl RuntimeBackend for X11Runtime {
    fn serve_event(&self, real: &Runtime) -> crate::Result<StorageVec<Event, 5>> {
        // this blocks the thread
        let mut event: MaybeUninit<xlib::XEvent> = MaybeUninit::uninit();
        log::trace!(
            "C function call: XNextEvent({:p}, [buffer])",
            self.display().as_ptr()
        );
        unsafe { xlib::XNextEvent(self.display().as_ptr(), event.as_mut_ptr()) };
        log::trace!("Finished C function call");

        super::translate_x11_event(self, real, unsafe { MaybeUninit::assume_init(event) })
    }

    #[inline]
    fn monitor_at(&self, index: usize) -> Option<&Monitor> {
        use core::ops::Deref;
        self.monitors.get(index).map(|i| i.deref())
    }

    #[cfg(feature = "async")]
    async fn serve_event_async(&self) -> crate::Result<StorageVec<Event, 5>> {
        Ok(())
    }

    #[inline]
    fn dispatch_event(&self, _ev: Event) -> crate::Result<()> {
        // event dispatching is handled in translation
        // so we do nothing :-)
        Ok(())
    }
}

impl Drop for X11Runtime {
    fn drop(&mut self) {
        // SAFETY: even if this somehow goes awry, we're disposing of the display anyways
        unsafe { xlib::XCloseDisplay(self.display.as_ptr()) };
    }
}
