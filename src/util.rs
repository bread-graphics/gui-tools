// MIT/Apache2 License

use std::{any, fmt, ops};

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct DebugContainer<T>(T);

impl<T> fmt::Debug for DebugContainer<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(any::type_name::<T>())
    }
}

impl<T> DebugContainer<T> {
    #[inline]
    pub(crate) fn new(t: T) -> Self {
        Self(t)
    }

    #[inline]
    pub(crate) fn into_inner(self) -> T {
        self.0
    }
}

impl<T> ops::Deref for DebugContainer<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> ops::DerefMut for DebugContainer<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
