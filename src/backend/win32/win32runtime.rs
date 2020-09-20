// MIT/Apache2 License

use super::{window_proc::window_procedure, Win32ImageStorage, Win32Monitor};
use crate::{
    event::{Event, EventType},
    monitor::Monitor,
    mutex::ShimMutex as Mutex,
    runtime::{Runtime, RuntimeBackend},
    string::CoolString,
};
use core::{
    cell::UnsafeCell,
    mem::{self, MaybeUninit},
    ptr::{self, NonNull},
};
use storagevec::StorageVec;
use winapi::{
    shared::{
        basetsd::ULONG_PTR,
        minwindef::{BOOL, FALSE, LPARAM, TRUE, UINT},
        ntdef::CHAR,
        windef::{HDC, HMONITOR, LPRECT},
    },
    um::{
        gdiplusinit::{self, GdiplusStartupInput},
        libloaderapi, wingdi,
        winuser::{self, MONITORINFO, MSG, WNDCLASSEXA},
    },
};

const WINDOW_CLASS_NAME: *const CHAR = b"GuiToolsWindowClass\0".as_ptr() as *const CHAR;

pub struct Win32Runtime {
    monitors: StorageVec<Win32Monitor, 12>,
    stored_events: Mutex<StorageVec<Event, 5>>,
    stored_error: Mutex<Option<crate::Error>>,
    window_class_init: Mutex<bool>,
    startup_token: ULONG_PTR,
    images: Win32ImageStorage,
}

// a function used to help get the list of monitors
#[allow(unused_unsafe)]
unsafe extern "system" fn enum_monitor_proc(
    monitor: HMONITOR,
    dc: HDC,
    rectangle: LPRECT,
    storage_ptr: LPARAM,
) -> BOOL {
    // get the monitor information
    let mut monitor_info = MONITORINFO {
        cbSize: mem::size_of::<MONITORINFO>() as _,
        ..unsafe { MaybeUninit::zeroed().assume_init() }
    };

    if unsafe { winuser::GetMonitorInfoA(monitor, &mut monitor_info) } == 0 {
        log::error!(
            "Unable to retrieve monitor: {}",
            crate::win32error("GetMonitorInfoA")
        );
        return FALSE;
    }

    // create a win32 monitor structure
    let mon = match Win32Monitor::new(monitor_info) {
        Ok(mon) => mon,
        Err(e) => {
            log::error!("Unable to process monitor info: {}", e);
            return FALSE;
        }
    };

    // transmute the LPARAM value to the pointer to the monitor storage we passed in
    let storage_ptr =
        unsafe { mem::transmute::<LPARAM, *mut StorageVec<Win32Monitor, 12>>(storage_ptr) };
    // reborrow it
    let storage = &mut *storage_ptr;

    // note: we want to avoid a panic while we're in the C side of things
    if let Err(_) = storage.try_push(mon) {
        log::error!("Storage vector for monitors is full. Unable to push to it.");
        return FALSE;
    }

    TRUE
}

impl Win32Runtime {
    #[inline]
    pub fn new(_commctrl: bool) -> crate::Result<(usize, Self)> {
        // get all of the monitors
        // this is an unsafe cell, just in case the compiler tries to optimize this as immutable
        let monitors: UnsafeCell<StorageVec<Win32Monitor, 12>> = UnsafeCell::new(StorageVec::new());
        if unsafe {
            winuser::EnumDisplayMonitors(
                ptr::null_mut(),
                ptr::null_mut(),
                Some(enum_monitor_proc),
                monitors.get() as *const () as LPARAM,
            )
        } == FALSE
        {
            return Err(crate::win32error("EnumDisplayMonitors"));
        }

        // initialize GDI+
        let startup_input: MaybeUninit<GdiplusStartupInput> = MaybeUninit::zeroed();
        let mut startup_token: MaybeUninit<ULONG_PTR> = MaybeUninit::uninit();
        unsafe {
            gdiplusinit::GdiplusStartup(
                startup_token.as_mut_ptr(),
                startup_input.as_ptr(),
                ptr::null_mut(),
            )
        };

        // TODO: commctrl
        let monitors = monitors.into_inner();
        Ok((
            monitors.iter().position(|m| m.primary()).unwrap(),
            Self {
                monitors,
                stored_events: Mutex::new(StorageVec::new()),
                stored_error: Mutex::new(None),
                window_class_init: Mutex::new(false),
                startup_token: unsafe { startup_token.assume_init() },
                images: Win32ImageStorage::new(),
            },
        ))
    }

    #[inline]
    pub fn image_storage(&self) -> &Win32ImageStorage {
        &self.images
    }

    #[inline]
    pub fn store_events(&self, events: StorageVec<Event, 5>) {
        self.stored_events.lock().extend(events);
    }

    #[inline]
    pub fn store_error(&self, error: crate::Error) {
        let mut old_error = Some(error);
        let mut lock = self.stored_error.lock();
        mem::swap(&mut old_error, &mut *lock);

        if let Some(e) = old_error {
            log::error!("An additional error occurred: {}", e);
        }
    }

    // create the window class that gets used for every window
    #[inline]
    pub fn create_window_class(&self) -> crate::Result<NonNull<CHAR>> {
        let mut wc_lock = self.window_class_init.lock();

        if *wc_lock {
            return Ok(unsafe { NonNull::new_unchecked(WINDOW_CLASS_NAME as *mut CHAR) });
        }

        // create the window class
        let icon =
            unsafe { winuser::LoadIconA(ptr::null_mut(), winuser::IDI_APPLICATION as *const _) };
        let cursor =
            unsafe { winuser::LoadCursorA(ptr::null_mut(), winuser::IDC_ARROW as *const _) };
        let window_class = WNDCLASSEXA {
            cbSize: mem::size_of::<WNDCLASSEXA>() as UINT,
            style: 0,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hbrBackground: unsafe { wingdi::GetStockObject(wingdi::WHITE_BRUSH as _) } as *mut _,
            lpszMenuName: ptr::null(),
            lpszClassName: WINDOW_CLASS_NAME,
            lpfnWndProc: Some(window_procedure),
            hInstance: unsafe { libloaderapi::GetModuleHandleA(ptr::null()) },
            hIcon: icon,
            hIconSm: icon,
            hCursor: cursor,
        };

        // register the window class
        if unsafe { winuser::RegisterClassExA(&window_class) } == 0 {
            return Err(crate::win32error("RegisterClassExA"));
        }

        *wc_lock = true;
        Ok(unsafe { NonNull::new_unchecked(WINDOW_CLASS_NAME as *mut CHAR) })
    }
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl RuntimeBackend for Win32Runtime {
    #[inline]
    fn serve_event(&self, _real: &Runtime) -> crate::Result<StorageVec<Event, 5>> {
        // run an iteration of the event loop.
        let mut msg: MaybeUninit<MSG> = MaybeUninit::uninit();
        match unsafe { winuser::GetMessageA(msg.as_mut_ptr(), ptr::null_mut(), 0, 0) } {
            // -1 means an error
            -1 => return Err(crate::win32error("GetMessage")),
            0 => {
                // end the event loop
                let mut quit_event = Event::new(EventType::Quit, None);
                quit_event.set_is_terminator(true);
                let mut sv = StorageVec::new();
                sv.push(quit_event);
                return Ok(sv);
            }
            _ => {
                // translate and dispatch the message
                unsafe {
                    winuser::TranslateMessage(msg.as_ptr());
                    winuser::DispatchMessageA(msg.as_ptr());
                }
            }
        }

        // if the stored error is anything, throw it
        if let Some(e) = self.stored_error.lock().take() {
            return Err(e);
        }

        // take the storage vector from the runtime; the window proc should do the work
        Ok(self
            .stored_events
            .lock()
            .drain(..)
            .collect::<StorageVec<Event, 5>>())
    }

    #[inline]
    fn monitor_at(&self, index: usize) -> Option<&Monitor> {
        use core::ops::Deref;
        self.monitors.get(index).map(|i| i.deref())
    }

    #[cfg(feature = "async")]
    async fn serve_event_async(&self) -> crate::Result<()> {
        Ok(())
    }

    #[inline]
    fn dispatch_event(&self, _ev: Event) -> crate::Result<()> {
        // we already dispatched the event
        Ok(())
    }
}
