// MIT/Apache2 License

#![cfg(windows)]

mod event;

use crate::{
    display::{Display, DrawHandler, EventHandler},
    mutex::Mutex,
    screen::{Screen, ScreenIter},
    window::{Window, WindowProps},
    Dimensions,
};
use chalkboard::yaww::{YawwGdiSurface, YawwGdiSurfaceResidual};
use dashmap::{mapref::entry::Entry, DashMap};
use nanorand::RNG;
use once_cell::sync::OnceCell;
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    ffi::{CStr, CString},
    iter,
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use thread_safe::ThreadSafe;
use yaww::{
    brush::DEFAULT_BRUSH,
    dc::Dc,
    monitor::{MonitorFunctions, MonitorInfo},
    window::{ExtendedWindowStyle, ShowWindowCommand, Window as YWindow, WindowStyle},
    window_class::ClassStyle,
    GuiThread, GuiThreadHandle, Rectangle, SendsDirective, WcFunctions, WindowFunctions,
};

/// Wrapper around a `yaww` handle.
#[derive(Clone)]
pub struct YawwDisplay<'evh> {
    handle: GuiThreadHandle<'evh>,
    associated: Arc<Data<'evh>>,
}

/// Internal data that needs to be passed around.
struct Data<'evh> {
    // hash map containing window properties
    window_properties: DashMap<YWindow, WindowProps>,
    // we keep a reference to the GUI thread
    gui_thread: Mutex<Option<ThreadSafe<GuiThread<'evh>>>>,
    // cached monitors
    monitors: OnceCell<HashMap<Screen, MonitorInfo>>,
    // the window class
    window_class: OnceCell<Box<CStr>>,
    // cached DC/cached window
    // Note: This HashMap is used in two places: in the draw() function, and in the WM_PAINT handler. Although
    //       the draw() function may be called from any thread, it is often called in the wndproc, where
    //       WM_PAINT is also called. Therefore I'm fine not using a DashMap in this case since the case where
    //       any contention actually happens in extremely rare and will probably lead to an error anyways
    dcs: Mutex<HashMap<YWindow, DcOrMaybeResidual>>,
}

#[derive(Default)]
struct DcOrMaybeResidual {
    dc: Option<Dc>,
    residual: Option<Residual>,
}

enum Residual {
    Gdi(YawwGdiSurfaceResidual),
}

impl<'evh> YawwDisplay<'evh> {
    #[inline]
    pub fn new() -> crate::Result<Self> {
        let gt = match GuiThread::try_new() {
            Ok(gt) => gt,
            Err(yaww::Error::AlreadyAYawwThread) => {
                return Err(crate::Error::StaticMsg("Thread already existed"))
            }
            Err(e) => return Err(e.into()),
        };
        log::trace!("Created GuiThread");
        let handle = gt.handle();
        log::trace!("Created GuiThreadHandle");
        Ok(Self {
            handle,
            associated: Arc::new(Data {
                window_properties: DashMap::new(),
                gui_thread: Mutex::new(Some(ThreadSafe::new(gt))),
                monitors: OnceCell::new(),
                window_class: OnceCell::new(),
                dcs: Mutex::new(HashMap::new()),
            }),
        })
    }

    #[inline]
    fn monitors(&self) -> crate::Result<&HashMap<Screen, MonitorInfo>> {
        let handle = self.handle.clone();
        self.associated.monitors.get_or_try_init(move || {
            log::trace!("Initialized monitors");
            let monitors = handle.monitors()?.wait()?;
            crate::Result::Ok(
                monitors
                    .into_iter()
                    .map(|s| (Screen::from_raw(s.monitor.into_raw().get()), s))
                    .collect(),
            )
        })
    }

    #[inline]
    fn is_monitor(&self, window: Window) -> crate::Result<bool> {
        let screen = Screen::from_raw(window.into_raw().get());
        Ok(self.monitors()?.contains_key(&screen))
    }

    #[inline]
    fn check_for_monitor(&self, window: Window) -> crate::Result<Option<(Screen, &MonitorInfo)>> {
        let screen = Screen::from_raw(window.into_raw().get());
        match self.monitors()?.get(&screen) {
            Some(mon) => Ok(Some((screen, mon))),
            None => Ok(None),
        }
    }

    #[inline]
    fn common_window_class(&self) -> crate::Result<CString> {
        let handle = self.handle.clone();
        self.associated
            .window_class
            .get_or_try_init(move || {
                let mut class_name = b"GuiToolsWndClass".to_vec();
                let mut rng = nanorand::tls_rng();
                class_name.extend(
                    iter::repeat_with(move || rng.generate_range(0x48, 0x57))
                        .take(nanorand::tls_rng().generate_range(1, 4)),
                );
                let class_name = CString::new(class_name)
                    .expect("Class name shouldn't have a 0 byte")
                    .into_boxed_c_str();

                handle
                    .register_class(
                        class_name.clone().into_c_string(),
                        None,
                        ClassStyle::empty(),
                        None,
                        None,
                        None,
                        Some(DEFAULT_BRUSH),
                    )?
                    .wait()?;

                crate::Result::Ok(class_name)
            })
            .map(|c| c.clone().into_c_string())
    }

    #[inline]
    pub(crate) fn create_window_custom_class(
        &self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        parent: Window,
        mut props: WindowProps,
        base_class: Option<Cow<'static, CStr>>,
    ) -> crate::Result<Window> {
        let (x, y, parent) = match self.check_for_monitor(parent)? {
            None => (x, y, Some(YWindow::from_raw(parent.into_raw()))),
            Some((scr, mon)) => (mon.x as i32 + x, mon.y as i32 + y, None),
        };

        let title = props.title.take();
        let title = CString::new(match title {
            Some(title) => title.into_bytes(),
            None => vec![],
        })
        .map_err(|_| crate::Error::NotCompatible)?;

        let window = self
            .handle
            .create_window(
                self.common_window_class()?,
                base_class,
                Some(Cow::Owned(title)),
                WindowStyle::OVERLAPPED_WINDOW,
                ExtendedWindowStyle::CLIENT_EDGE,
                x,
                y,
                width as _,
                height as _,
                parent,
                None,
            )?
            .wait()?;
        self.associated.window_properties.insert(window, props);
        Ok(Window::from_raw(window.into_raw()))
    }
}

impl<'evh> Display<'evh> for YawwDisplay<'evh> {
    #[inline]
    fn screens(&mut self) -> crate::Result<ScreenIter<'_>> {
        let screens: Vec<usize> = self.monitors()?.keys().map(|s| s.into_raw()).collect();
        Ok(ScreenIter::from(screens))
    }

    #[inline]
    fn default_screen(&mut self) -> crate::Result<Screen> {
        let scr = self.handle.default_monitor()?.wait()?;
        Ok(Screen::from_raw(scr.into_raw().get()))
    }

    #[inline]
    fn screen_dimensions(&mut self, screen: Screen) -> crate::Result<(u32, u32)> {
        match self.monitors()?.get(&screen) {
            Some(monitor) => Ok((monitor.width as _, monitor.height as _)),
            None => Err(crate::Error::NoScreen(screen.into_raw())),
        }
    }

    #[inline]
    fn toplevel_window(&mut self, screen: Screen) -> crate::Result<Window> {
        // we represent Windows as "either a Window or a Monitor", so this is a simple
        // translation
        Ok(Window::from_raw(
            NonZeroUsize::new(screen.into_raw()).expect("Screen should not be zero"),
        ))
    }

    #[inline]
    fn create_window(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        parent: Window,
        props: WindowProps,
    ) -> crate::Result<Window> {
        self.create_window_custom_class(x, y, width, height, parent, props, None)
    }

    #[inline]
    fn delete_window(&mut self, window: Window) -> crate::Result {
        let window = YWindow::from_raw(window.into_raw());
        window.close(&self.handle)?.wait()?;
        self.associated.window_properties.remove(&window);
        Ok(())
    }

    #[inline]
    fn set_window_visibility(&mut self, window: Window, visible: bool) -> crate::Result {
        if self.is_monitor(window)? {
            return Err(crate::Error::CannotOnMonitor);
        }

        let window = YWindow::from_raw(window.into_raw());
        let _ = window
            .show(
                &self.handle,
                if visible {
                    ShowWindowCommand::SHOW
                } else {
                    ShowWindowCommand::HIDE
                },
            )?
            .wait();
        Ok(())
    }

    #[inline]
    fn window_dimensions(&mut self, window: Window) -> crate::Result<Dimensions> {
        if let Some((_, mon_info)) = self.check_for_monitor(window)? {
            return Ok(Dimensions {
                x: mon_info.x as _,
                y: mon_info.y as _,
                width: mon_info.width as _,
                height: mon_info.height as _,
            });
        }

        let Rectangle {
            left,
            top,
            right,
            bottom,
        } = cvt_window(window).get_client_rect(&self.handle)?.wait()?;

        Ok(Dimensions {
            x: left as _,
            y: top as _,
            width: (right - left) as _,
            height: (bottom - top) as _,
        })
    }

    // window_coordinates and window_size are implemented in terms of the above function

    #[inline]
    fn window_set_dimensions(
        &mut self,
        window: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> crate::Result {
        if self.is_monitor(window)? {
            return Err(crate::Error::CannotOnMonitor);
        }

        let window = cvt_window(window);
        window
            .move_resize_window(&self.handle, x, y, width as _, height as _, true)?
            .wait()?;
        Ok(())
    }

    #[inline]
    fn window_set_coordinates(&mut self, window: Window, x: i32, y: i32) -> crate::Result {
        if self.is_monitor(window)? {
            return Err(crate::Error::CannotOnMonitor);
        }

        let window = cvt_window(window);
        window.move_window(&self.handle, x, y, true)?.wait()?;
        Ok(())
    }

    #[inline]
    fn window_set_size(&mut self, window: Window, width: u32, height: u32) -> crate::Result {
        if self.is_monitor(window)? {
            return Err(crate::Error::CannotOnMonitor);
        }

        let window = cvt_window(window);
        window
            .resize_window(&self.handle, width as _, height as _, true)?
            .wait()?;
        Ok(())
    }

    #[inline]
    fn draw_with_boxed_draw_handler(
        &mut self,
        window: Window,
        handler: DrawHandler<'_>,
    ) -> crate::Result {
        if self.is_monitor(window)? {
            unimplemented!()
        }

        let window = cvt_window(window);
        let mut dcs = self.associated.dcs.lock();
        let entry = match dcs.get_mut(&window) {
            Some(entry) => entry,
            None => return Err(crate::Error::NoValidDraw(window.into_raw())),
        };

        let dc = entry
            .dc
            .take()
            .ok_or(crate::Error::NoValidDraw(window.into_raw()))?;
        // TODO: also handle direct2d and gl, when the time comes
        let mut surface = match entry.residual.take() {
            Some(Residual::Gdi(residual)) => {
                YawwGdiSurface::from_residual(&self.handle, dc, residual)
            }
            None => YawwGdiSurface::new(&self.handle, dc),
        };

        let res = handler(&mut surface);

        let residual = match surface {
            surface => Residual::Gdi(surface.into_residual()),
        };

        entry.residual = Some(residual);
        res
    }

    #[inline]
    fn window_parent(&mut self, window: Window) -> crate::Result<Option<Window>> {
        let window = cvt_window(window);
        let parent = window.get_parent(&self.handle)?.wait();
        Ok(parent.map(cvt_window_r))
    }

    #[inline]
    fn run_with_boxed_event_handler(&mut self, mut handler: EventHandler<'evh>) -> crate::Result {
        log::trace!("Running yaww main loop");

        let gt = self
            .associated
            .gui_thread
            .lock()
            .take()
            .ok_or(crate::Error::AlreadyRanMainLoop)?
            .try_into_inner()
            .map_err(|_| {
                crate::Error::StaticMsg("Main loop can only be ran in the originating thread")
            })?;

        log::trace!("Loaded main GuiThread");
        let mut this = self.clone();
        let event_handler: Box<
            dyn FnMut(&yaww::PinnedGuiThreadHandle<'evh>, yaww::Event) -> yaww::Result
                + Send
                + 'evh,
        > = Box::new(move |_, ev| {
            let ev = match event::cvt_event(&this, ev) {
                Ok(Some(ev)) => ev,
                Ok(None) => return Ok(()),
                Err(e) => return Err(yaww::Error::Dynamic(Arc::new(e))),
            };
            match handler(&mut this, ev) {
                Ok(()) => Ok(()),
                Err(e) => Err(yaww::Error::Dynamic(Arc::new(e))),
            }
        });
        gt.set_event_handler(event_handler);

        match gt.main_loop() {
            Ok(()) => Ok(()),
            Err(e) => match e {
                e => Err(e.into()),
            },
        }
    }
}

#[inline]
fn cvt_window(window: Window) -> YWindow {
    YWindow::from_raw(window.into_raw())
}

#[inline]
pub(crate) fn cvt_window_r(window: YWindow) -> Window {
    Window::from_raw(window.into_raw())
}
