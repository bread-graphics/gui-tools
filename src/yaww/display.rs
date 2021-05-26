// MIT/Apache2 License

use super::EventHandlerUpdate;
use crate::{
    display::Display,
    screen::{Screen, ScreenIter},
};
use flume::Sender;
use once_cell::sync::OnceCell;
use std::{collections::BTreeMap, ffi::CStr, sync::Arc};
use yaww::{
    monitor::{Monitor, MonitorInfo},
    server::GuiThread,
    window::YWindow,
    Rectangle,
};

/// A wrapper around the YAWW GuiThread that provides a Display.
#[derive(Debug, Clone)]
pub struct YawwDisplay {
    gui_thread: GuiThread,
    data: Arc<YawwInnerData>,
    evh_communicator: Sender<EventHandlerUpdate>,
}

#[derive(Debug)]
struct YawwInnerData {
    monitors: OnceCell<BTreeMap<u64, MonitorCache>>,
    class: OnceCell<&'static CStr>,
}

#[derive(Debug, Clone)]
struct MonitorCache {
    width: u32,
    height: u32,
    x: i32,
    y: i32,
}

impl YawwDisplay {
    #[inline]
    pub fn new() -> crate::Result {
        if std::mem::size_of::<usize>() > 64 {
            panic!("This program assumes that it will never be run on a 128-bit computer");
        }

        let gui_thread = GuiThread::new();
        let (evh_communicator, rx) = flume::unbounded();

        let this = Self {
            gui_thread: GuiThread::new(),
            evh_communicator,
            data: Arc::new(YawwInnerData {
                monitors: OnceCell::new(),
                class: OnceCell::new(),
            }),
        };

        let evh = super::event_handler(this.clone(), rx);
        this.gui_thread.set_event_handler(evh)?.wait();

        Ok(this)
    }

    #[inline]
    fn gui_thread(&self) -> &GuiThread {
        &self.gui_thread
    }

    #[inline]
    fn monitors(&self) -> crate::Result<&BTreeMap<usize, Size>> {
        self.data.monitors.get_or_try_init(|| {
            crate::Result::Ok(
                self.gui_thread()
                    .monitors()?
                    .wait()?
                    .into_iter()
                    .map(
                        |MonitorInfo {
                             monitor,
                             width,
                             height,
                             x,
                             y,
                         }| {
                            (
                                monitor.into_raw() as u64,
                                MonitorCache {
                                    width: width.into(),
                                    height: height.into(),
                                    x: x.into(),
                                    y: y.into(),
                                },
                            )
                        },
                    )
                    .collect(),
            )
        })
    }

    #[inline]
    fn is_monitor(&mut self, window: Window) -> crate::Result<bool> {
        Ok(self.monitors()?.contains_key(&window.into_raw().get()))
    }
}

impl<'a> Display for &'a YawwDisplay {
    #[inline]
    fn screens(&mut self) -> crate::Result<ScreenIter<'_>> {
        Ok(ScreenIter::from_iterator(
            self.monitors()?.keys().copied().map(Screen::from_raw),
        ))
    }

    #[inline]
    fn default_screen(&mut self) -> crate::Result<Screen> {
        Ok(monitor_to_screen(
            self.gui_thread().default_monitor()?.wait()?,
        ))
    }

    #[inline]
    fn screen_dimensions(&mut self, screen: Screen) -> crate::Result<(u32, u32)> {
        match self.monitors()?.get(&screen.into_raw()) {
            Some(MonitorCache { width, height, .. }) => Ok((*width, *height)),
            None => Err(crate::Error::StaticMsg("Monitor not found")),
        }
    }

    #[inline]
    fn toplevel_window(&mut self, screen: Screen) -> crate::Result<Window> {
        // for Yaww, we represent the "Window" struct as either a pointer to a monitor or a pointer to a window
        // this function just converts a known monitor pointer to the window pointer
        Ok(Window::from_raw(
            NonZeroU64::new(screen.into_raw() as u64).expect("Screen pointer should not be null"),
        ))
    }

    #[inline]
    fn create_window(
        &mut self,
        mut x: i32,
        mut y: i32,
        width: u32,
        height: u32,
        parent: Window,
        mut props: WindowProps,
    ) -> crate::Result<Window> {
        const CLASS_NAME: &[u8] = b"yaww-class\0";

        // register our default window class if we haven't yet
        let window_class = self.data.window_class.get_or_try_init(|| {
            self.gui_thread
                .register_class(
                    CStr::from_bytes(CLASS_NAME.as_ref()).unwrap(),
                    None,
                    ClassStyle::empty(),
                    None,
                    None,
                    None,
                    Some(DEFAULT_BRUSH),
                )?
                .wait()
        })?;

        let parent = match self.monitors()?.get(&parent) {
            Some(MonitorCache { x: mx, y: my, .. }) => {
                x += *mx;
                y += *my;
                None
            }
            None => Some(YWindow::from_raw(
                NonZeroUsize::new(parent.into_raw().get() as usize).expect("Literally impossible"),
            )),
        };

        let window_name = mem::take(&mut props.title);
        let window_name =
            window_name.map(|window_name| CString::new(window_name).expect("Invalid title!"));
        let window = self
            .gui_thread
            .create_window(
                CStr::from_bytes(CLASS_NAME.as_ref()).unwrap(),
                None,
                window_name.into(),
                WindowStyle::OVERLAPPED_WINDOW,
                ExtendedWIndowStyle::CLIENT_EDGE,
                x,
                y,
                width,
                height,
                parent,
                None,
            )?
            .wait()?;

        self.evh_communicator
            .send(EventHandlerUpdate::WindowProps(window, window_props))
            .expect("Event handler closed");

        Ok(Window::from_raw(
            NonZeroU64::new(window.into_raw() as u64).expect("Impossible"),
        ))
    }

    #[inline]
    fn destroy_window(&mut self, window: Window) -> crate::Result {
        let window = YWindow::from_raw(NonZeroUsize::new(window.into_raw().get()).expect("NPP"));
        let _task = window.close(&self.gui_thread)?;
        #[cfg(debug_assertions)]
        _task.wait()?;
        Ok(())
    }

    #[inline]
    fn window_geometry(&mut self, window: Window) -> crate::Result<(i32, i32, u32, u32)> {
        if let Some(info) = self.monitors()?.get(&window.into_raw().get()) {
            Ok((
                info.x.into(),
                info.y.into(),
                info.width.try_into().expect("width"),
                info.height.try_into().expect("height"),
            ))
        }

        let window = YWindow::from_raw(NonZeroUsize::new(window.into_raw().get()).expect("NPP"));
        let Rectangle {
            left,
            top,
            right,
            bottom,
        } = window.get_window_rect(self.gui_thread())?.wait()?;
        Ok((
            left,
            top,
            (right - left).abs() as u32,
            (bottom - top).abs() as u32,
        ))
    }

    #[inline]
    fn window_parent(&mut self, window: Window) -> crate::Result<Option<Window>> {
        if self.is_monitor(window)? {
            return Ok(None);
        }
        let window = YWindow::from_raw(NonZeroUsize::new(window.into_raw().get()).expect("NPP"));
        let parent = window.get_parent()?.wait()?;
        Ok(Window::from_raw(
            NonZeroU64::new(parent.into_raw()).expect("NPP"),
        ))
    }
}

#[inline]
fn monitor_to_screen(monitor: Monitor) -> Screen {
    Screen::from_raw(monitor.into_raw())
}
