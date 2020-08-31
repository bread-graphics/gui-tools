// MIT/Apache2 License

use core::fmt;
use cty::{c_int, c_uchar, c_ulong};

#[cfg(feature = "alloc")]
use alloc::string::String;

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
    ErrorEventThrown {
        serial: c_ulong,
        error_code: c_ulong,
        request_code: c_uchar,
        minor_code: c_uchar,

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
            #[cfg(not(feature = "alloc"))]
            Self::ErrorEventThrown { serial, error_code, request_code, minor_code } => write!(f, "An X11 internal error occurred (serial: {} - error code: {} - request code: {} - minor code: {})", serial, error_code, request_code, minor_code),
            #[cfg(feature = "alloc")]
            Self::ErrorEventThrown { serial, error_code, request_code, minor_code, ref error_description } => write!(f, "An X11 internal error occurred: {} (serial: {} - error code: {} - request code: {} - minor code: {})", error_description, serial, error_code, request_code, minor_code),
        }
    }
}
