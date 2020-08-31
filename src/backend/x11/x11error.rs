// MIT/Apache2 License

use super::{x11displaymanager, X11Runtime};
use crate::X11Error;
use core::ptr::NonNull;
use cty::{c_char, c_int, c_uchar, c_ulong};
use x11nas::xlib::{self, Display, XErrorEvent};

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

// type of the X11 error handler callback
type X11ErrorCallback = Option<unsafe extern "C" fn(_: *mut Display, _: *mut XErrorEvent) -> c_int>;

/// Error data
#[derive(Copy, Clone)]
pub struct X11ErrorData {
    pub serial: c_ulong,
    pub error_code: c_uchar,
    pub request_code: c_uchar,
    pub minor_code: c_uchar,
}

#[cfg(not(feature = "alloc"))]
fn error_data_to_error(ed: X11ErrorData, display: NonNull<Display>) -> X11Error {
    X11Error::ErrorEventThrown {
        serial: ed.serial,
        error_code: ed.error_code,
        request_code: ed.request_code,
        minor_code: ed.minor_code,
    }
}

#[cfg(feature = "alloc")]
fn error_data_to_error(ed: X11ErrorData, display: NonNull<Display>) -> X11Error {
    // create a buffer for the error text
    let mut buffer = Vec::with_capacity(64);

    // get the x11 error code
    log::trace!(
        "C function call: XGetErrorText({:p}, {}, [buffer], 63)",
        display.as_ptr(),
        ed.error_code
    );
    let res = unsafe {
        xlib::XGetErrorText(
            display.as_ptr(),
            ed.error_code as c_int,
            buffer.as_mut_ptr() as *mut c_char,
            63,
        )
    };
    log::trace!("Result of C function call: {}", res);

    // convert to a string
    let error_description =
        String::from_utf8(buffer).expect("Unable to convert error text to UTF-8");

    X11Error::ErrorEventThrown {
        serial: ed.serial,
        error_code: ed.error_code.into(),
        request_code: ed.request_code,
        minor_code: ed.minor_code,
        error_description,
    }
}

impl X11ErrorData {
    fn to_error(self, display: NonNull<Display>) -> crate::Error {
        error_data_to_error(self, display).into()
    }
}

/// error trapper
#[derive(Copy, Clone)]
pub struct X11ErrorTrap {
    // the last error callback
    last_error: X11ErrorCallback,
    // the current depth of this error trap
    depth: usize,
    // the last error to occur, if any
    error: Option<X11ErrorData>,
}

impl X11ErrorTrap {
    /// Create a new X11ErrorTrap.
    #[inline]
    pub const fn new() -> Self {
        Self {
            last_error: None,
            depth: 0,
            error: None,
        }
    }

    /// Push another frame onto the error trap stack.
    #[inline]
    pub fn push(&mut self) {
        if self.last_error.is_some() || self.depth > 0 {
            log::warn!(
                "Attempted to add an error trap to the stack when there is already one there."
            );
        }

        let last_error = unsafe { xlib::XSetErrorHandler(Some(x11_error_handler)) };
        if self.depth == 0 {
            self.last_error = last_error;
        }
        self.depth += 1;
    }

    /// Pop a frame from the error trap stack.
    #[inline]
    pub fn pop(&mut self, dpy: NonNull<Display>) -> crate::Result<()> {
        self.depth -= 1;

        if self.depth == 0 {
            unsafe { xlib::XSetErrorHandler(self.last_error.take()) };
        }

        match self.error.take() {
            Some(e) => Err(e.to_error(dpy)),
            None => Ok(()),
        }
    }
}

unsafe extern "C" fn x11_error_handler(dpy: *mut Display, error: *mut XErrorEvent) -> c_int {
    // get the runtime by its display
    let dpy = match NonNull::new(dpy) {
        Some(dpy) => dpy,
        None => {
            log::error!("x11_error_handler was passed a null display");
            return 0;
        }
    };

    let runtime = match x11displaymanager::get_runtime(dpy) {
        Some(runtime) => runtime,
        None => {
            log::warn!("A non-gui-tools display was reported to the X11 error handler");
            return 0;
        }
    };

    // get a mutable reference to the error trap
    let xruntime = runtime.as_x11().unwrap();
    let mut error_trap = xruntime.error_trap_mut();

    // convert the XErrorEvent to the X11ErrorData
    // we turn the mut pointer to a reference
    let error_event = match NonNull::new(error) {
        Some(error_event) => error_event,
        None => {
            log::error!("x11_error_handler was passed a null error event");
            return 0;
        }
    };
    let err = unsafe { error_event.as_ref() };

    let error_data = X11ErrorData {
        serial: err.serial,
        error_code: err.error_code,
        request_code: err.request_code,
        minor_code: err.minor_code,
    };

    error_trap.error = Some(error_data);

    0
}
