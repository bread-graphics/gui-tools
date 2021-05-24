// MIT/Apache2 License

use crate::{
    display::Display,
    screen::{Screen, ScreenIter},
};
use once_cell::sync::OnceCell;
use std::{collections::BTreeMap, sync::Arc};
use yaww::{
    monitor::{Monitor, MonitorInfo},
    server::GuiThread,
};

/// A wrapper around the YAWW GuiThread that provides a Display.
#[derive(Debug, Clone)]
pub struct YawwDisplay {
    gui_thread: GuiThread,
    monitors: Arc<OnceCell<BTreeMap<usize, MonitorCache>>>,
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
        let gui_thread = GuiThread::new();

        Ok(Self {
            gui_thread: GuiThread::new(),
            monitors: Arc::new(OnceCell::new()),
        })
    }

    #[inline]
    fn gui_thread(&self) -> &GuiThread {
        &self.gui_thread
    }

    #[inline]
    fn monitors(&mut self) -> crate::Result<&BTreeMap<usize, Size>> {
        self.monitors.get_or_try_init(|| {
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
                                monitor.into_raw(),
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
        Ok(self.monitors()?.contains_key(&window))
    }
}

impl Display for YawwDisplay {
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
        // this function just converts from one to the other
        Ok(Window::from_raw(
            NonZeroU64::new(screen.into_raw() as u64).expect("Screen pointer should not be null"),
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
    }
}

#[inline]
fn monitor_to_screen(monitor: Monitor) -> Screen {
    Screen::from_raw(monitor.into_raw())
}
