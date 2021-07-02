// MIT/Apache2 License

//! Implementation of a mutex. This is backed by `std::sync::Mutex` when the `pl` feature is not enabled, and the
//! `parking_lot::Mutex` when the `pl` feature is enabled.

// At the moment, this is only used on the "yaww" and "itaos" platforms. breadx has no need for mutexes.
#![cfg(any(windows, target_os = "macos"))]

#[cfg(not(feature = "pl"))]
use std::sync;

pub(crate) struct Mutex<T: ?Sized> {
    #[cfg(feature = "pl")]
    inner: parking_lot::Mutex<T>,
    #[cfg(not(feature = "pl"))]
    inner: sync::Mutex<T>,
}

impl<T> Mutex<T> {
    #[inline]
    pub(crate) fn new(data: T) -> Mutex<T> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "pl")] {
                Mutex { inner: parking_lot::Mutex::new(data) }
            } else {
                Mutex { inner: sync::Mutex::new(data) }
            }
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    #[cfg(feature = "pl")]
    #[inline]
    pub(crate) fn lock(&self) -> parking_lot::MutexGuard<'_, T> {
        self.inner.lock()
    }

    #[cfg(not(feature = "pl"))]
    #[inline]
    pub(crate) fn lock(&self) -> sync::MutexGuard<'_, T> {
        self.inner.lock().expect("Unable to lock mutex")
    }
}
