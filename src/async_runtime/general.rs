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

use super::Runtime;
use alloc::boxed::Box;

/// A sum type containing all of the `Runtime`s used on this
/// system.
#[non_exhaustive]
pub enum GeneralRuntime {
    /// A runtime not listed here.
    Dynamic(Box<dyn Runtime + Send + 'static>),
}

impl Runtime for GeneralRuntime {}
