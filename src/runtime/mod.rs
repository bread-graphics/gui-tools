// MIT/Apache2 License

//! The runtime for the GUI engine.

use crate::{
    backend::{
        x11::{X11Runtime, X11_BACKEND},
        Backend, BackendType, RuntimeInner, SurfaceInner,
    },
    monitor::Monitor,
    mutex::{RwLockReadGuard, RwLockWriteGuard, ShimRwLock},
    surface::{Surface, SurfaceProperties},
};
use core::{cell::UnsafeCell, ptr::NonNull};
use owning_ref::OwningRef;
use spinning_top::Spinlock as Mutex;
use storagevec::StorageMap;

#[cfg(feature = "alloc")]
use alloc::sync::Arc;

// current internal runtime
// used as a pointer to circumvent
// allocation
#[cfg(not(feature = "alloc"))]
static GLOBAL_RUNTIME: UnsafeCell<Option<ShimRwLock<RuntimeInternal>>> = UnsafeCell::new(None);
#[cfg(not(feature = "alloc"))]
static GLOBAL_RUNTIME_MUTEX: Mutex<()> = Mutex::new();

#[cfg(not(feature = "alloc"))]
#[inline]
unsafe fn global_runtime() -> &'static Option<ShimRwLock<RuntimeInternal>> {
    &*GLOBAL_RUNTIME.get()
}

#[cfg(not(feature = "alloc"))]
#[inline]
unsafe fn global_runtime_mut() -> &'static mut Option<ShimRwLock<RuntimeInternal>> {
    &mut *GLOBAL_RUNTIME.get()
}

#[cfg(not(feature = "alloc"))]
/// The runtime for the GUI engine.
pub struct Runtime;

#[cfg(feature = "alloc")]
#[repr(transparent)]
/// The runtime for the GUI engine.
pub struct Runtime(Arc<ShimRwLock<RuntimeInternal>>);

fn new_inner_runtime(backend: Backend) -> crate::Result<RuntimeInternal> {
    let (default_monitor, sys) = backend.open()?;
    Ok(RuntimeInternal {
        backend,
        sys,
        default_monitor,
        surfaces: StorageMap::new(),
    })
}

#[doc(hidden)]
pub struct RuntimeInternal {
    // the system-specific display object
    sys: RuntimeInner,
    backend: Backend,

    // the default monitor that windows are initially spawned on
    default_monitor: usize,

    // list of surfaces contained in the SysRuntime
    surfaces: StorageMap<usize, Surface, 25>,
}

impl Runtime {
    #[cfg(feature = "alloc")]
    #[inline]
    fn new_impl(backend: Backend) -> crate::Result<Self> {
        Ok(Self(Arc::new(ShimRwLock::new(new_inner_runtime(backend)?))))
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn new_impl(backend: Backend) -> crate::Result<Self> {
        let _spinny = GLOBAL_RUNTIME_MUTEX.lock();

        // SAFETY: we have the spinny lock, we have exclusive access to the unsafe cell
        if let None = unsafe { global_runtime() } {
            *unsafe { global_runtime_mut() } = Some(new_inner_runtime(backend)?);
        } else {
            return Err(crate::Error::RuntimeDuplication);
        }

        Ok(Self)
    }

    /// Create a new runtime.
    #[inline]
    pub fn new() -> crate::Result<Self> {
        Self::new_impl(X11_BACKEND)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn inner(&self) -> RwLockReadGuard<'_, RuntimeInternal> {
        self.0.read()
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn inner(&self) -> RwLockReadGuard<'_, RuntimeInternal> {
        // SAFETY: the only time global_runtime_mut is called is during initialization
        unsafe { global_runtime() }.as_ref().unwrap().read()
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn inner_locked(&self) -> RwLockWriteGuard<'_, RuntimeInternal> {
        self.0.write()
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn inner_locked(&self) -> RwLockWriteGuard<'_, RuntimeInternal> {
        // SAFETY: same as above
        unsafe { global_runtime() }.as_ref().unwrap().write()
    }

    #[inline]
    pub(crate) fn as_x11(
        &self,
    ) -> Option<OwningRef<RwLockReadGuard<'_, RuntimeInternal>, X11Runtime>> {
        let inner = self.inner();
        match inner.sys.as_x11() {
            Some(_) => Some(OwningRef::new(inner).map(|ri| match ri.sys.as_x11() {
                Some(x) => x,
                None => unreachable!(),
            })),
            None => None,
        }
    }

    #[inline]
    pub(crate) fn default_monitor_index(&self) -> usize {
        self.inner().default_monitor
    }

    #[inline]
    pub(crate) fn surface_at(
        &self,
        id: usize,
    ) -> OwningRef<RwLockReadGuard<'_, RuntimeInternal>, Surface> {
        OwningRef::new(self.inner()).map(|ri| ri.surfaces.get(&id).unwrap())
    }

    /// The current graphics framework associated with this runtime.
    #[inline]
    pub fn ty(&self) -> BackendType {
        self.inner().backend.ty()
    }

    /// Create a new surface using a set of window properties.
    #[inline]
    pub fn create_surface(&self, properties: SurfaceProperties) -> crate::Result<usize> {
        let window = Surface::new(self, properties)?;
        let id = window.id();
        self.inner_locked().surfaces.insert(id, window);
        Ok(id)
    }

    /// The backend assocaited with this item.
    #[inline]
    pub fn backend(&self) -> OwningRef<RwLockReadGuard<'_, RuntimeInternal>, Backend> {
        OwningRef::new(self.inner()).map(|ri| &ri.backend)
    }
}

pub trait RuntimeBackend {}
