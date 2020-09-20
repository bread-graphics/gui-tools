// MIT/Apache2 License

use core::{fmt, ptr::NonNull};
use cty::{c_char, c_int, c_uchar, c_ulong};

#[cfg(target_os = "linux")]
use x11nas::xlib;

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

/// A container type for X11-related errors.
#[derive(Debug)]
pub enum X11Error {
    DisplayDidntOpen,
    BadAtoms,
    BadScreenId(c_int),
    BadVisualDepth(c_int),
    BadVisualPointer,
    BadVisualColorType(c_int),
    BadGetVisualInfo,
    BadDefaultVisual,
    BadInputMethod,
    BadInputContext,
    BadGraphicsContext,
    NoKeysymFound,
    BadImage,
    ErrorEventThrown {
        serial: c_ulong,
        error_code: c_ulong,
        request_code: c_uchar,
        minor_code: c_uchar,

        #[cfg(feature = "alloc")]
        error_description: String,
    },
    Status {
        code: c_int,
        #[cfg(feature = "alloc")]
        error_description: String,
    },
}

impl fmt::Display for X11Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DisplayDidntOpen => f.write_str("Unable to open connection to X11 server"),
            Self::BadAtoms => f.write_str("Unable to cache X11 internal atoms"),
            Self::BadScreenId(id) => write!(
                f,
                "The monitor ID \"{}\" did not correspond to an actual monitor",
                id
            ),
            Self::BadVisualDepth(depth) => write!(
                f,
                "The visual type loaded had a depth of {}, which is less than 1",
                depth
            ),
            Self::BadVisualPointer => f.write_str("The visual type's pointer was null"),
            Self::BadVisualColorType(class) => {
                write!(f, "The visual type's color class, {}, was not known", class)
            }
            Self::BadGetVisualInfo => {
                f.write_str("The XGetVisualInfo() function returned a null pointer")
            }
            Self::BadDefaultVisual => f.write_str("The default visual type pointer was null"),
            Self::BadInputMethod => f.write_str("The input method was null"),
            Self::BadInputContext => f.write_str("The input context was null"),
            Self::BadGraphicsContext => f.write_str("The graphics context for a window was null"),
            Self::BadImage => f.write_str("The image format was invalid for X11"),
            Self::NoKeysymFound => f.write_str("The input event did not contain a key symbol"),
            #[cfg(not(feature = "alloc"))]
            Self::Status { code } => write!(f, "An X11 function failed with status {}", code),
            #[cfg(feature = "alloc")]
            Self::Status { code, ref error_description } => write!(f, "An X11 function failed with status {}: {}", code, error_description),
            #[cfg(not(feature = "alloc"))]
            Self::ErrorEventThrown { serial, error_code, request_code, minor_code } => write!(f, "An X11 internal error occurred (serial: {} - error code: {} - request code: {} - minor code: {})", serial, error_code, request_code, minor_code),
            #[cfg(feature = "alloc")]
            Self::ErrorEventThrown { serial, error_code, request_code, minor_code, ref error_description } => write!(f, "An X11 internal error occurred: {} (serial: {} - error code: {} - request code: {} - minor code: {})", error_description, serial, error_code, request_code, minor_code),            
        }
    }
}

// convert a status to an x11 error
#[cfg(all(target_os = "linux", not(feature = "alloc")))]
#[inline]
fn x11_status_to_err(_dpy: NonNull<xlib::Display>, status: c_int) -> crate::Error {
    X11Error::Status { code: status }.into()
}

#[cfg(all(target_os = "linux", feature = "alloc"))]
#[inline]
fn x11_status_to_err(dpy: NonNull<xlib::Display>, status: c_int) -> crate::Error {
    const BUFFER_SIZE: usize = 100;
    let mut buffer: [c_char; BUFFER_SIZE] = [0; BUFFER_SIZE];

    let len = unsafe {
        xlib::XGetErrorText(
            dpy.as_ptr(),
            status,
            buffer.as_mut_ptr(),
            BUFFER_SIZE as c_int - 1,
        )
    } as usize;

    X11Error::Status {
        code: status,
        error_description: String::from_utf8(buffer[0..len].iter().map(|i| *i as _).collect())
            .unwrap(),
    }
    .into()
}

#[cfg(target_os = "linux")]
#[inline]
pub(crate) fn x11_status_to_res(dpy: NonNull<xlib::Display>, status: c_int) -> crate::Result<()> {
    if status == 1 || status == 0 {
        Ok(())
    } else {
        Err(x11_status_to_err(dpy, status))
    }
}
