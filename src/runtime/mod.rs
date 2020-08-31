// MIT/Apache2 License

//! The runtime for the GUI engine.

use crate::{
    backend::{
        x11::{X11Runtime, X11_BACKEND},
        Backend, BackendType, RuntimeInner, SurfaceInner,
    },
    event::{
        delivery::{DefaultEventDelivery, EventDelivery},
        Event, EventLoopAction,
    },
    monitor::Monitor,
    mutex::{RwLockReadGuard, RwLockWriteGuard, ShimRwLock},
    surface::{Surface, SurfaceInitialization, SurfaceProperties},
};
use core::{cell::UnsafeCell, mem, ptr::NonNull};
use owning_ref::{OwningRef, OwningRefMut};
use spinning_top::Spinlock as Mutex;
use storagevec::{StorageMap, StorageVec};

#[cfg(feature = "alloc")]
use alloc::sync::Arc;

#[cfg(feature = "async")]
use alloc::boxed::Box;
#[cfg(feature = "async")]
use core::future::Future;

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
        delivery: DefaultEventDelivery::new(),
        peekers: StorageVec::new(),
        default_monitor,
        surfaces: StorageMap::new(),
        events: Mutex::new(StorageVec::new()),
        still_running: None,
        #[cfg(feature = "async")]
        joiner: None,
        #[cfg(feature = "async")]
        joiner_depth: 0,
    })
}

impl Clone for Runtime {
    #[cfg(feature = "alloc")]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    #[cfg(not(feature = "alloc"))]
    fn clone(&self) -> Self {
        Self
    }
}

// TODO: verify this
unsafe impl Send for Runtime {}
unsafe impl Sync for Runtime {}

#[doc(hidden)]
pub struct RuntimeInternal {
    // the system-specific display object
    sys: RuntimeInner,
    backend: Backend,
    delivery: DefaultEventDelivery,

    // people to inform of new events
    peekers: StorageVec<&'static dyn Fn(&Runtime, &Event) -> crate::Result<EventLoopAction>, 5>,

    // the default monitor that windows are initially spawned on
    default_monitor: usize,

    // list of surfaces contained in the SysRuntime
    surfaces: StorageMap<usize, Surface, 25>,

    // a list containing all of the current events
    // TODO: make a stack-based queue structure at some point
    events: Mutex<StorageVec<Event, 5>>,

    // should the event loop continue running?
    // None if the event loop hasn't started yet
    still_running: Option<bool>,

    // the currently joinable future that contains every
    // current event handler in a joiner structure
    #[cfg(feature = "async")]
    joiner: Option<Box<dyn Future<Output = crate::Result<()>>>>,

    // joiner has something similar to a tree-like structure,
    // it is useful to know its depth
    #[cfg(feature = "async")]
    joiner_depth: usize,
}

impl Runtime {
    #[cfg(not(feature = "alloc"))]
    #[inline]
    pub(crate) unsafe fn global() -> Self {
        assert!(global_runtime().is_some());
        Self
    }

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
        let this = Self::new_impl(X11_BACKEND)?;

        if let BackendType::X11 = this.ty() {
            crate::backend::x11::x11displaymanager::set_runtime(
                this.as_x11().unwrap().display().clone(),
                this.clone(),
            );
        }
        Ok(this)
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
    pub(crate) fn as_x11_mut(
        &self,
    ) -> Option<OwningRefMut<RwLockWriteGuard<'_, RuntimeInternal>, X11Runtime>> {
        let inner = self.inner_locked();
        match inner.sys.as_x11() {
            Some(_) => Some(
                OwningRefMut::new(inner).map_mut(|ri| match ri.sys.as_x11_mut() {
                    Some(x) => x,
                    None => unreachable!(),
                }),
            ),
            None => None,
        }
    }

    #[inline]
    pub fn default_monitor_index(&self) -> usize {
        self.inner().default_monitor
    }

    #[inline]
    pub fn default_monitor(&self) -> OwningRef<RwLockReadGuard<'_, RuntimeInternal>, Monitor> {
        let inner = self.inner();
        OwningRef::new(inner).map(|inner| inner.sys.monitor_at(inner.default_monitor).unwrap())
    }

    #[inline]
    pub(crate) fn surface_at(
        &self,
        id: usize,
    ) -> Option<OwningRef<RwLockReadGuard<'_, RuntimeInternal>, Surface>> {
        let inner = self.inner();
        if inner.surfaces.contains_key(&id) {
            Some(OwningRef::new(self.inner()).map(|ri| ri.surfaces.get(&id).unwrap()))
        } else {
            None
        }
    }

    /// The current graphics framework associated with this runtime.
    #[inline]
    pub fn ty(&self) -> BackendType {
        self.inner().backend.ty()
    }

    /// Create a new surface using a set of window properties.
    #[inline]
    pub fn create_surface(&self, properties: SurfaceInitialization) -> crate::Result<usize> {
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

    /// Serve events.
    #[inline]
    pub(crate) fn serve_event(&self) -> crate::Result<StorageVec<Event, 5>> {
        self.inner().sys.serve_event(self)
    }

    /// Dispatch events.
    #[inline]
    pub(crate) fn dispatch_event(&self, ev: Event) -> crate::Result<bool> {
        if ev.is_terminator() {
            Ok(false)
        } else {
            self.inner().sys.dispatch_event(ev)?;
            Ok(true)
        }
    }

    /// Run an iteration of the event loop.
    #[inline]
    pub fn run_iteration(&self) -> crate::Result<bool> {
        log::debug!("Running an iteration of the event loop.");

        let mut inner = Some(self.inner_locked());
        let ev = match inner.as_mut().unwrap().delivery.pop_event() {
            Some(ev) => ev,
            None => {
                mem::drop(inner.take()); // drop the write lock so we don't contest the process

                
                loop {
                    let inner_read = self.inner(); 
                    if inner_read.delivery.pending() { break; }

                    log::debug!("Querying system for new events...");
                    let served = inner_read.sys.serve_event(self)?;
                    mem::drop(inner_read);
                    self.inner_locked().delivery.add_events(served);
                }

                inner = Some(self.inner_locked());
                inner.as_mut().unwrap().delivery.pop_event().unwrap()
            }
        };

        // clone the peekers out while we have the lock
        let peekers = inner.as_mut().unwrap().peekers.clone();

        mem::drop(inner);

        log::trace!("There is {} peekers", peekers.len());
        let mut peekers_iter = peekers.into_iter();
        while let Some(peek) = peekers_iter.next() {
            match peek(self, &ev) {
                Err(e) => return Err(e),
                Ok(EventLoopAction::Break) => return Ok(false),
                Ok(EventLoopAction::Continue) => (),
            }
        }

        self.dispatch_event(ev)
    }

    /// Add a peeker to this runtime.
    #[inline]
    pub fn add_peeker(&self, peeker: &'static dyn Fn(&Runtime, &Event) -> crate::Result<EventLoopAction>) {
        self.inner_locked().peekers.push(peeker)
    }

    /// Run this event loop.
    #[inline]
    pub fn run(&self) -> crate::Result<()> {
        log::info!("Beginning the event loop...");

        while self.run_iteration()? {}

        Ok(())
    }
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait RuntimeBackend {
    /// Serve an event (or list of events), blocking the current execution thread.
    fn serve_event(&self, real: &Runtime) -> crate::Result<StorageVec<Event, 5>>;
    #[cfg(feature = "async")]
    /// Serve an event (or list of events), not blocking the current thread.
    async fn serve_event_async(&self) -> crate::Result<StorageVec<Event, 5>>;
    /// Dispatch an event.
    fn dispatch_event(&self, ev: Event) -> crate::Result<()>;

    /// Get a monitor at a certain index.
    fn monitor_at(&self, monitor: usize) -> Option<&Monitor>;
}
