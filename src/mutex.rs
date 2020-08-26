// MIT/Apache2 License

//! Shim for mutexes and rwlocks.

#[cfg(not(feature = "std"))]
use spinning_top::Spinlock as Mutex;
#[cfg(not(feature = "std"))]
pub use spinning_top::SpinlockGuard as MutexGuard;
#[cfg(not(feature = "std"))]
use spinny::RwLock;
#[cfg(not(feature = "std"))]
pub use spinny::{RwLockReadGuard, RwLockWriteGuard};

#[cfg(all(feature = "std", not(feature = "pl")))]
use std::sync::{Mutex, RwLock};
#[cfg(all(feature = "std", not(feature = "pl")))]
pub use std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};

#[cfg(all(feature = "std", feature = "pl"))]
use parking_lot::{Mutex, RwLock};
#[cfg(all(feature = "std", feature = "pl"))]
pub use parking_lot::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};

#[repr(transparent)]
pub struct ShimMutex<T>(Mutex<T>);

impl<T> ShimMutex<T> {
    #[inline]
    pub fn new(item: T) -> Self {
        Self(Mutex::new(item))
    }

    #[cfg(any(not(feature = "std"), feature = "pl"))]
    #[inline]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.0.lock()
    }

    #[cfg(all(feature = "std", not(feature = "pl")))]
    #[inline]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.0.lock().unwrap()
    }

    #[cfg(any(not(feature = "std"), feature = "pl"))]
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.0.get_mut()
    }

    #[cfg(all(feature = "std", not(feature = "pl")))]
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.0.get_mut().unwrap()
    }
}

#[repr(transparent)]
pub struct ShimRwLock<T>(RwLock<T>);

impl<T> ShimRwLock<T> {
    #[inline]
    pub fn new(item: T) -> Self {
        Self(RwLock::new(item))
    }

    #[cfg(any(not(feature = "std"), feature = "pl"))]
    #[inline]
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        self.0.read()
    }

    #[cfg(all(feature = "std", not(feature = "pl")))]
    #[inline]
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        self.0.read().unwrap()
    }

    #[cfg(any(not(feature = "std"), feature = "pl"))]
    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.0.write()
    }

    #[cfg(all(feature = "std", not(feature = "pl")))]
    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.0.write().unwrap()
    }

    #[cfg(any(not(feature = "std"), feature = "pl"))]
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.0.get_mut()
    }

    #[cfg(all(feature = "std", not(feature = "pl")))]
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.0.get_mut().unwrap()
    }
}
