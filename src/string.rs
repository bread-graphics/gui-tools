// MIT/Apache2 License

use core::ops;

#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, string::String};

/// A cool string that is very cool.
///
/// On platforms that do not support allocators, this is an `&'static str`. On platforms that do, this is
/// a `Cow<'static, str>`.
#[derive(Debug)]
#[repr(transparent)]
pub struct CoolString(CoolStringInner);

#[cfg(not(feature = "alloc"))]
type CoolStringInner = &'static str;

#[cfg(feature = "alloc")]
type CoolStringInner = Cow<'static, str>;

impl ops::Deref for CoolString {
    type Target = str;

    #[cfg(feature = "alloc")]
    #[inline]
    fn deref(&self) -> &str {
        &*self.0
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn deref(&self) -> &str {
        self.0
    }
}

impl CoolString {
    /// Create a new CoolString from a static string.
    #[inline]
    pub fn from_static(s: &'static str) -> Self {
        Self::from_static_impl(s)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn from_static_impl(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn from_static_impl(s: &'static str) -> Self {
        Self(s)
    }

    /// Create a new CoolString from an allocated string.
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn from_alloc(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl From<&'static str> for CoolString {
    #[inline]
    fn from(s: &'static str) -> Self {
        Self::from_static(s)
    }
}

#[cfg(feature = "alloc")]
impl From<String> for CoolString {
    #[inline]
    fn from(s: String) -> Self {
        Self::from_alloc(s)
    }
}

impl Default for CoolString {
    #[inline]
    fn default() -> Self {
        Self::from_static("")
    }
}
