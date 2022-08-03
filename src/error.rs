// This file is part of gui-tools.
//
// gui-tools is free software: you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option)
// any later version.
//
// gui-tools is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty
// of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General
// Public License along with gui-tools. If not, see
// <https://www.gnu.org/licenses/>.

use alloc::boxed::Box;
use chalkboard::Error as ChalkError;
use core::fmt;

/// The error type used in `gui-tools`.
pub struct Error {
    /// The inner internal error.
    error: Internal,
    /// A chain to a previous error, if this error exists in a list
    /// of consequential errors.
    link: Option<Box<Error>>,
}

enum Internal {
    /// A chalkboard error.
    ///
    /// This type is also used to store string-based message errors.
    Chalk(ChalkError),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dt = f.debug_tuple("Error");
        dt.field(&self.error);

        if let Some(ref link) = self.link {
            dt.field(link);
        }

        dt.finish()
    }
}

impl fmt::Debug for Internal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chalk(c) => fmt::Debug::fmt(c, f),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write the initial error message
        match self.error {
            Internal::Chalk(ref e) => fmt::Display::fmt(e, f)?,
        }

        // if necessary, write the link to the previous error
        if let Some(ref link) = self.link {
            f.write_str("\n\nNext error in linked list: ")?;
            fmt::Display::fmt(link, f)?;
        }

        Ok(())
    }
}

pub type Result<T = ()> = core::result::Result<T, Error>;
