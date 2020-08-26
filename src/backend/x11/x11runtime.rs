// MIT/Apache2 License

use super::{X11Monitor, X11Surface};
use crate::{
    backend::SurfaceInner,
    error::X11Error,
    runtime::RuntimeBackend,
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

#[derive(Copy, Clone)]
#[repr(usize)]
enum X11Atom {
    WmDeleteWindow = 0,
}

const X11_ATOMS_LEN: usize = 1;
const X11_ATOMS: [X11Atom; X11_ATOMS_LEN] = [X11Atom::WmDeleteWindow];
const X11_ATOMS_NAMES: [&'static CStr; X11_ATOMS_LEN] = unsafe {
    [
        // SAFETY: all of these are valid CStrings
        CStr::from_bytes_with_nul_unchecked(b"WM_DELETE_WINDOW\0"),
    ]
};

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
}

impl X11Runtime {
    // create a new display
    pub fn new() -> crate::Result<(usize, Self)> {
        unsafe { xlib::XInitThreads() };

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
        let mut internal_atoms: [MaybeUninit<Atom>; X11_ATOMS_LEN] = MaybeUninit::uninit_array();
        let names = X11_ATOMS_NAMES.as_ptr() as *const *const CStr as *const *const [c_char]
            as *const *const c_char;
        let status = unsafe {
            xlib::XInternAtoms(
                display.as_ptr(),
                names as *mut *mut c_char,
                X11_ATOMS_LEN as c_int,
                1,
                internal_atoms.as_mut_ptr() as *mut Atom,
            )
        };
        if status == 0 {
            return Err(X11Error::BadAtoms.into());
        }

        // SAFETY: same as below
        let default_screen: c_int = unsafe { xlib::XDefaultScreen(display.as_ptr()) };

        // create the runtime to put items into
        let mut runtime = X11Runtime {
            display,
            // SAFETY: internal_atoms are guaranteed to be valid atoms if
            //         XInternAtoms did not fail
            internal_atoms: unsafe { mem::transmute::<_, [Atom; X11_ATOMS_LEN]>(internal_atoms) },
            monitors: StorageVec::new(),
            default_colormap: None,
            default_visual: None,
            default_depth: None,
        };

        // SAFETY: C function that returns an integer, we check it for validity
        let monitor_count = unsafe { xlib::XScreenCount(runtime.display.as_ptr()) };
        if monitor_count <= 0 {
            // a display should still have monitors if it initialized properly
            panic!("Unexpected monitor count: {}", monitor_count);
        }

        // create a new monitor for every screen
        let i = (0..monitor_count)
            .into_iter()
            .map(|id| X11Monitor::new(&mut runtime, id, id == default_screen))
            .collect::<crate::Result<StorageVec<X11Monitor, 2>>>()?;
        runtime.monitors.extend(i);

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
}

impl RuntimeBackend for X11Runtime {}

impl Drop for X11Runtime {
    fn drop(&mut self) {
        // SAFETY: even if this somehow goes awry, we're disposing of the display anyways
        unsafe { xlib::XCloseDisplay(self.display.as_ptr()) };
    }
}
