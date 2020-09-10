// MIT/Apache2 License

//! The runtime for the GUI engine.

use crate::{
    backend::{
        select_backend,
        win32::Win32Runtime,
        x11::{X11Runtime, X11_BACKEND},
        Backend, BackendType, RuntimeInner,
    },
    event::{
        delivery::{DefaultEventDelivery, EventDelivery},
        Event, EventLoopAction,
    },
    monitor::Monitor,
    mutex::{RwLockReadGuard, RwLockWriteGuard, ShimRwLock},
    surface::{Surface, SurfaceInitialization},
};
use core::{
    fmt, mem,
    sync::atomic::{AtomicBool, Ordering},
};
use owning_ref::OwningRef;
use storagevec::{StorageMap, StorageVec};

#[cfg(not(feature = "alloc"))]
use core::cell::UnsafeCell;
#[cfg(not(feature = "alloc"))]
use spinning_top::{Spinlock as Mutex, SpinlockGuard as MutexGuard};

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, sync::Arc};

#[cfg(feature = "async")]
use core::future::Future;

// current internal runtime
// used as a pointer to circumvent allocation
#[cfg(not(feature = "alloc"))]
struct GlobalRuntime(UnsafeCell<Option<RuntimeInternal>>, Mutex<()>);

#[cfg(not(feature = "alloc"))]
impl GlobalRuntime {
    #[inline]
    const fn new() -> Self {
        Self(UnsafeCell::new(None), Mutex::new(()))
    }
}

#[cfg(not(feature = "alloc"))]
unsafe impl Send for GlobalRuntime {}
#[cfg(not(feature = "alloc"))]
unsafe impl Sync for GlobalRuntime {}

#[cfg(not(feature = "alloc"))]
static GLOBAL_RUNTIME: GlobalRuntime = GlobalRuntime::new();

#[cfg(not(feature = "alloc"))]
impl GlobalRuntime {
    #[inline]
    unsafe fn inner(&self) -> &Option<RuntimeInternal> {
        &*self.0.get()
    }

    #[inline]
    unsafe fn inner_mut(&self) -> &mut Option<RuntimeInternal> {
        &mut *self.0.get()
    }

    #[inline]
    unsafe fn get_lock(&self) -> MutexGuard<'_, ()> {
        self.1.lock()
    }
}

#[cfg(not(feature = "alloc"))]
/// The runtime for the GUI engine.
pub struct Runtime {
    _private: (),
}

#[cfg(feature = "alloc")]
#[repr(transparent)]
/// The runtime for the GUI engine.
pub struct Runtime(Arc<RuntimeInternal>);

fn new_inner_runtime(backend: Backend) -> crate::Result<RuntimeInternal> {
    let (default_monitor, sys) = backend.open()?;
    Ok(RuntimeInternal {
        backend,
        sys,
        delivery: DefaultEventDelivery::new(),
        peekers: ShimRwLock::new(StorageVec::new()),
        default_monitor,
        surfaces: ShimRwLock::new(StorageMap::new()),
        suppress_peeker_loop: AtomicBool::new(backend.suppress_peeker_loop),
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
unsafe impl Send for RuntimeInternal {}
unsafe impl Sync for RuntimeInternal {}

#[derive(Clone)]
pub(crate) enum Peeker {
    Unowned(&'static dyn Fn(&Runtime, &Event) -> crate::Result<EventLoopAction>),
    #[cfg(feature = "alloc")]
    Owned(Arc<dyn Fn(&Runtime, &Event) -> crate::Result<EventLoopAction>>),
}

impl Peeker {
    fn call(&self, runtime: &Runtime, event: &Event) -> crate::Result<EventLoopAction> {
        match self {
            Self::Unowned(f) => f(runtime, event),
            #[cfg(feature = "alloc")]
            Self::Owned(ref b) => b(runtime, event),
        }
    }
}

#[doc(hidden)]
pub struct RuntimeInternal {
    // the system-specific display object
    sys: RuntimeInner,
    backend: Backend,
    delivery: DefaultEventDelivery,

    // people to inform of new events
    peekers: ShimRwLock<StorageVec<Peeker, 5>>,

    // the default monitor that windows are initially spawned on
    default_monitor: usize,

    // list of surfaces contained in the SysRuntime
    surfaces: ShimRwLock<StorageMap<usize, Surface, 25>>,

    // whether or not to suppress the peeker loop (i.e. it's already been run (i.e. win32))
    suppress_peeker_loop: AtomicBool,

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
        assert!(GLOBAL_RUNTIME.inner().is_some());
        Self { _private: () }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn new_impl(backend: Backend) -> crate::Result<Self> {
        Ok(Self(Arc::new(new_inner_runtime(backend)?)))
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn new_impl(backend: Backend) -> crate::Result<Self> {
        let _spinny = unsafe { GLOBAL_RUNTIME.get_lock() };

        // SAFETY: we have the spinny lock, we have exclusive access to the unsafe cell
        if let None = unsafe { GLOBAL_RUNTIME.inner() } {
            *unsafe { GLOBAL_RUNTIME.inner_mut() } = Some(new_inner_runtime(backend)?);
        } else {
            return Err(crate::Error::RuntimeDuplication);
        }

        Ok(Self { _private: () })
    }

    /// Create a new runtime.
    #[inline]
    pub fn new() -> crate::Result<Self> {
        let backend = match select_backend() {
            Some(backend) => backend,
            None => return Err(crate::Error::NoBackendFound),
        };
        Self::from_backend(backend)
    }

    /// Create a new runtime from a specific backend.
    #[inline]
    pub fn from_backend(backend: Backend) -> crate::Result<Self> {
        let this = Self::new_impl(backend)?;

        // register w/ the backend
        backend.register(&this);

        Ok(this)
    }

    #[inline]
    #[cfg(feature = "alloc")]
    pub(crate) fn into_ptr(self) -> *const RuntimeInternal {
        Arc::into_raw(self.0)
    }

    #[inline]
    #[cfg(not(feature = "alloc"))]
    pub(crate) fn into_ptr(self) -> *const RuntimeInernal {
        unsafe { GLOBAL_RUNTIME.inner() }.as_ref().unwrap()
    }

    #[inline]
    #[cfg(feature = "alloc")]
    pub(crate) unsafe fn from_ptr(ptr: *const RuntimeInternal) -> Self {
        Self(Arc::from_raw(ptr))
    }

    #[inline]
    #[cfg(not(feature = "alloc"))]
    pub(crate) unsafe fn from_ptr(ptr: *const RuntimeInternal) -> Self {
        assert!(ptr as *const _ == GLOBAL_RUNTIME.inner().as_ref().unwrap() as *const _);
        Self { private: () }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn inner(&self) -> &RuntimeInternal {
        log::debug!("Borrowing inner lock immutably");
        &*self.0
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn inner(&self) -> &RuntimeInternal {
        // SAFETY: the only time global_runtime_mut is called is during initialization
        unsafe { GLOBAL_RUNTIME.inner() }.as_ref().unwrap()
    }

    #[inline]
    pub(crate) fn as_x11(&self) -> Option<&X11Runtime> {
        self.inner().sys.as_x11()
    }

    #[inline]
    pub(crate) fn as_win32(&self) -> Option<&Win32Runtime> {
        self.inner().sys.as_win32()
    }

    #[inline]
    pub fn default_monitor_index(&self) -> usize {
        self.inner().default_monitor
    }

    #[inline]
    pub fn default_monitor(&self) -> Option<&Monitor> {
        self.inner().sys.monitor_at(self.inner().default_monitor)
    }

    #[inline]
    pub fn surface_at(
        &self,
        id: usize,
    ) -> Option<OwningRef<RwLockReadGuard<'_, StorageMap<usize, Surface, 25>>, Surface>> {
        let surfaces = self.inner().surfaces.read();
        match surfaces.contains_key(&id) {
            true => Some(OwningRef::new(surfaces).map(move |surfaces| surfaces.get(&id).unwrap())),
            false => None,
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
        self.inner().surfaces.write().insert(id, window);
        Ok(id)
    }

    /// The backend assocaited with this item.
    #[inline]
    pub fn backend(&self) -> &Backend {
        &self.inner().backend
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

    #[inline]
    pub(crate) fn peeker_loop(&self, peekers: &[Peeker], event: &Event) -> crate::Result<bool> {
        let mut peekers_iter = peekers.iter();
        while let Some(peek) = peekers_iter.next() {
            match peek.call(self, event) {
                Err(e) => return Err(e),
                Ok(EventLoopAction::Break) => return Ok(false),
                Ok(EventLoopAction::Continue) => (),
            }
        }

        Ok(true)
    }

    /// Run an iteration of the event loop.
    #[inline]
    pub fn run_iteration(&self) -> crate::Result<bool> {
        log::debug!("Running an iteration of the event loop.");

        let inner = self.inner();
        let ev = match inner.delivery.pop_event() {
            Some(ev) => ev,
            None => loop {
                if let Some(ev) = inner.delivery.pop_event() {
                    break ev;
                }

                log::debug!("Querying system for new events...");
                let served = inner.sys.serve_event(self)?;
                inner.delivery.add_events(served);
            },
        };
        log::debug!("Running iteration on event {:?}", &ev);

        // clone the peekers out while we have the lock
        let suppress_peeker_loop = inner.suppress_peeker_loop.load(Ordering::Acquire);

        if !suppress_peeker_loop {
            if !self.peeker_loop(&inner.peekers.read(), &ev)? {
                return Ok(false);
            }
        }

        self.dispatch_event(ev)
    }

    /// Add a peeker to this runtime.
    #[inline]
    pub fn add_peeker(
        &self,
        peeker: &'static dyn Fn(&Runtime, &Event) -> crate::Result<EventLoopAction>,
    ) {
        self.inner().peekers.write().push(Peeker::Unowned(peeker))
    }

    /// Add an owned peeker to this runtime.
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn add_peeker_owned<F>(&self, peeker: F)
    where
        F: Fn(&Runtime, &Event) -> crate::Result<EventLoopAction> + 'static,
    {
        self.inner().peekers.write().push(Peeker::Owned(
            (Box::new(peeker) as Box<dyn Fn(&Runtime, &Event) -> crate::Result<EventLoopAction>>)
                .into(),
        ))
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

impl fmt::Debug for Runtime {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Runtime")
    }
}
