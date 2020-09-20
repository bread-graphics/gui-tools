// MIT/Apache2 License

//! The runtime for the GUI engine.
//!
//! The runtime handles several details necessary to `gui-tools`, including:
//!
//! * Loading the backend.
//! * The creation and management of surfaces.
//! * Polling for events and storing them in the event queue.
//! * Dispatching events in the event queue.
//! * Abstracting over runtime-like constructs (e.g. the display connection in X11).
//!
//! `gui-tools` programs will likely start by instantiating a runtime, and end with calling
//! the runtime's main loop function.
//!
//! # Creating a Runtime
//!
//! Runtimes are often created with the [`Runtime::new`] function.
//!
//! ```no_run
//! use gui_tools::runtime::Runtime;
//!
//! let runtime = Runtime::new().unwrap();
//! ```
//!
//! However, the inner details of creation vary based upon which features are enabled. The `Runtime` struct
//! is implemented as a pointer to a `RuntimeInternal` struct, which contains all of the real details of the
//! runtime.
//!
//! * If the `alloc` feature is enabled (i.e. an allocator is present), `Runtime` is implemented as a
//!   transparent wrapper around an `Arc<RuntimeInternal>`. The `RuntimeInternal` is stored in the heap.
//!   Therefore, one can create as many runtimes as they want.
//! * If the `alloc` feature is not enabled (i.e. an allocator is not present), `Runtime` is implemented
//!   as a pointer to a static memory location containing the `RuntimeInternal`. Because of this, only
//!   one runtime can be created.
//!
//! In either case, the `Runtime` struct is cheaply clonable, and its clone will refer to the same runtime.
//!
//! Note that the [`Runtime::from_backend`] function can also be used to create runtimes with a specific
//! backend.
//!
//! # Event Management
//!
//! In the course of the main loop, the Runtime polls for events and dispatches them. However, if you want
//! to operate on these events before they are dispatched, add a "peeker".
//!
//! A peeker is a function that takes the runtime and the event being "peeked" at as parameters. It is used
//! to add functionality to the application. There are two ways to add peekers:
//!
//! * Add a static reference to a peeker function via the `Runtime::add_peeker` method.
//!
//! ```no_run
//! # use gui_tools::runtime::Runtime;
//! # let runtime = Runtime::new().unwrap();
//! use gui_tools::{error::Result, event::{EventLoopAction, Event}};
//!
//! fn peeker(_runtime: &Runtime, event: &Event) -> Result<EventLoopAction> {
//!      println!("Processing event: {:?}", event);
//!      Ok(EventLoopAction::Continue)
//! }
//!
//! runtime.add_peeker(&peeker);
//!
//! runtime.run().unwrap();
//! ```
//!
//! * If the `alloc` feature is enabled, use a closure as a peeker via the `Runtime::add_peeker_owned` method.
//!
//! ```no_run
//! # use gui_tools::runtime::Runtime;
//! # let runtime = Runtime::new().unwrap();
//! use gui_tools::event::EventLoopAction;
//!
//! runtime.add_peeker_owned(|_r, event| {
//!     println!("Processing event: {:?}", event);
//!     Ok(EventLoopAction::Continue)
//! });
//!
//! runtime.run().unwrap();
//! ```
//!
//! The `Runtime::run` function begins the event loop.

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

#[cfg(not(feature = "std"))]
use conquer_once::spin::OnceCell;
#[cfg(feature = "std")]
use conquer_once::OnceCell;

#[cfg(feature = "async")]
use core::future::Future;

// on no_std system, store the global runtime in a once cell
#[cfg(not(feature = "alloc"))]
static GLOBAL_RUNTIME: OnceCell<RuntimeInternal> = OnceCell::uninit();

#[cfg(not(feature = "alloc"))]
/// The runtime for the GUI engine.
///
/// See the module-level documentation for more information.
pub struct Runtime {
    _private: (),
}

#[cfg(feature = "alloc")]
#[repr(transparent)]
/// The runtime for the GUI engine.
///
/// See the module-level documentation for more information.
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

/// The internal runtime that the `Runtime` struct points to. You will probably not need to interact
/// with this.
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
        assert!(GLOBAL_RUNTIME.is_initialized());
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
        GLOBAL_RUNTIME.init_once(|| new_runtime_inner(backend).unwrap());

        Ok(Self { _private: () })
    }

    /// Create a new runtime. This automatically selects a backend based upon system resources available. If this
    /// is not desirable for you, consider using [`Runtime::from_backend`] instead.
    ///
    /// # Errors
    ///
    /// If no backend was able to be loaded, this function will return `Error::NoBackendFound`.
    ///
    /// If the `alloc` feature is not enabled and the user attempts to create a second runtime,
    /// this function will return `Error::RuntimeDuplication`.
    ///
    /// If an error occurs in the backend, it will be propogated to this function.
    #[inline]
    pub fn new() -> crate::Result<Self> {
        let backend = match select_backend() {
            Some(backend) => backend,
            None => return Err(crate::Error::NoBackendFound),
        };
        Self::from_backend(backend)
    }

    /// Create a new runtime using a specific backend. If you have already decided on a backend for yourself,
    /// use this function to create the runtime.
    ///
    /// # Errors
    ///
    /// If the `alloc` feature is not enabled and the user attempts to create a second runtime,
    /// this function will return `Error::RuntimeDuplication`.
    ///
    /// If an error occurs in the backend, it will be propogated to this function.
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
        GLOBAL_RUNTIME.get().unwrap()
    }

    #[inline]
    #[cfg(feature = "alloc")]
    pub(crate) unsafe fn from_ptr(ptr: *const RuntimeInternal) -> Self {
        Self(Arc::from_raw(ptr))
    }

    #[inline]
    #[cfg(not(feature = "alloc"))]
    pub(crate) unsafe fn from_ptr(ptr: *const RuntimeInternal) -> Self {
        assert!(ptr as *const _ == GLOBAL_RUNTIME.get().unwrap() as *const _);
        Self { private: () }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn inner(&self) -> &RuntimeInternal {
        &*self.0
    }

    #[cfg(not(feature = "alloc"))]
    #[inline]
    fn inner(&self) -> &RuntimeInternal {
        GLOBAL_RUNTIME.get().unwrap()
    }

    #[inline]
    pub(crate) fn as_x11(&self) -> Option<&X11Runtime> {
        self.inner().sys.as_x11()
    }

    #[inline]
    pub(crate) fn as_win32(&self) -> Option<&Win32Runtime> {
        self.inner().sys.as_win32()
    }

    /// Get the index of the default monitor.
    #[inline]
    pub fn default_monitor_index(&self) -> usize {
        self.inner().default_monitor
    }

    /// Get a reference to the default monitor.
    #[inline]
    pub fn default_monitor(&self) -> Option<&Monitor> {
        self.inner().sys.monitor_at(self.inner().default_monitor)
    }

    /// Given a unique surface ID, returns a reference to the surface. Note that this immutably locks the mutex
    /// holding the list of surfaces.
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

    /// The type of backend used with this runtime.
    #[inline]
    pub fn ty(&self) -> BackendType {
        self.inner().backend.ty()
    }

    /// Create a new surface using a set of window properties.
    ///
    /// This creates a surface using the specified properties, and inserts it into the surface list.
    ///
    /// # Errors
    ///
    /// Any errors created by the backend are propogated to this function.
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
    pub(crate) fn peekers(&self) -> RwLockReadGuard<'_, StorageVec<Peeker, 5>> {
        self.inner().peekers.read()
    }

    #[inline]
    pub(crate) fn peeker_loop(&self, peekers: &[Peeker], event: &Event) -> crate::Result<bool> {
        if event.skip_peeker_loop() {
            return Ok(true);
        }

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

    /// Run an iteration of the event loop. If the user wants more control of the event loop, they can call
    /// this function in a loop. Note that one iteration is not guaranteed to correspond to one event. Returns
    /// Ok(false) if the loop should be stopped. In addition, the preferred way of handling events is to use
    /// peekers.
    ///
    /// # Errors
    ///
    /// Any errors created by the backend or the peekers are propogated to this function.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gui_tools::runtime::Runtime;
    ///
    /// let runtime = Runtime::new().unwrap();
    ///
    /// while runtime.run_iteration().unwrap() {
    ///     println!("One event loop iteration has passed...");
    /// }
    /// ```
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

    /// Add a peeker to this runtime. This function accepts a static reference to a peeker. If it is
    /// easier to pass a closure to this runtime, consider using `Runtime::add_peeker_owned`.
    #[inline]
    pub fn add_peeker(
        &self,
        peeker: &'static dyn Fn(&Runtime, &Event) -> crate::Result<EventLoopAction>,
    ) {
        self.inner().peekers.write().push(Peeker::Unowned(peeker))
    }

    /// Add an owned peeker to this runtime. This accepts closures, unlike `Runtime::add_peeker`.
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

    /// Run this event loop. This loop polls for events, takes an event from the event queue, runs the peeker
    /// functions on that event, and then dispatches the event.
    #[inline]
    pub fn run(&self) -> crate::Result<()> {
        log::info!("Beginning the event loop...");

        while self.run_iteration()? {}

        Ok(())
    }
}

/// The backend of the runtime. Backend implementors should implement this trait for the runtime.
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
