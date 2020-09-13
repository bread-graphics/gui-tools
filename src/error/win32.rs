// MIT/Apache2 License

use core::fmt;
use cty::{c_char, c_ulong};

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

#[derive(Debug)]
pub enum Win32Error {
    FunctionFailure {
        function_name: &'static str,
        error_code: c_ulong,
        #[cfg(feature = "alloc")]
        error_text: String,
    },
    NoDCAvailable,
    DCIsNull,
}

impl fmt::Display for Win32Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDCAvailable => f.write_str("Unable to acquire a DC to paint with"),
            Self::DCIsNull => f.write_str("Unable to create a DC for paint operations"),
            #[cfg(not(feature = "alloc"))]
            Self::FunctionFailure {
                function_name,
                error_code,
            } => write!(
                f,
                "The Win32 function {} failed with error code {}",
                function_name, error_code
            ),
            #[cfg(feature = "alloc")]
            Self::FunctionFailure {
                function_name,
                error_code,
                ref error_text,
            } => write!(
                f,
                "The Win32 function {} failed with error code {}: {}",
                function_name, error_code, error_text
            ),
        }
    }
}

#[cfg(all(windows, not(feature = "alloc")))]
pub(crate) fn win32error(fname: &'static str) -> crate::Error {
    use winapi::um::errhandlingapi;

    // get the last error
    let last_error = unsafe { errhandlingapi::GetLastError() };

    Win32Error::FunctionFailure {
        function_name: fname,
        error_code: last_error,
    }
    .into()
}

#[cfg(all(windows, feature = "alloc"))]
pub(crate) fn win32error(fname: &'static str) -> crate::Error {
    use core::ptr;
    use winapi::{
        shared::ntdef,
        um::{errhandlingapi, winbase},
    };

    // get the last error
    let last_error = unsafe { errhandlingapi::GetLastError() };

    // get the error text
    let mut buffer: Vec<c_char> = Vec::with_capacity(1024);
    let len = unsafe {
        winbase::FormatMessageA(
            winbase::FORMAT_MESSAGE_FROM_SYSTEM | winbase::FORMAT_MESSAGE_IGNORE_INSERTS,
            ptr::null_mut(),
            last_error,
            ntdef::MAKELANGID(ntdef::LANG_NEUTRAL, ntdef::SUBLANG_DEFAULT).into(),
            buffer.as_mut_ptr(),
            0,
            ptr::null_mut(),
        )
    };
    unsafe { buffer.set_len(len as _) };

    Win32Error::FunctionFailure {
        function_name: fname,
        error_code: last_error,
        error_text: String::from_utf8(buffer.into_iter().map(|c| c as _).collect()).unwrap(),
    }
    .into()
}
