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

use core::num::NonZeroUsize;

/// A window that belongs to a runtime.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Window(NonZeroUsize);

impl Window {
    /// Create a new window from a `NonZeroUsize`.
    pub fn new(id: NonZeroUsize) -> Self {
        Window(id)
    }

    /// Get the ID of the window.
    pub fn id(self) -> NonZeroUsize {
        self.0
    }
}
