// MIT/Apache2 License

use crate::{
    display::{Display as GtDisplay, EventHandler},
    event::Event,
    screen::{Screen, ScreenIter},
    util::DebugContainer,
    window::{Visibility, Window, WindowProps},
};
use breadx::{
    display::{
        name::NameConnection, ConfigureWindowParameters, Connection, Display, Geometry,
        WindowParameters,
    },
    Window as XWindow, WindowClass,
};
use std::{convert::TryInto, num::NonZeroU64};

#[derive(Debug)]
pub struct BreadxDisplay<Conn> {
    display: Display<Conn>,
    closed: bool,
}

pub type BreadxDisplayConnection = BreadxDisplay<NameConnection>;

impl BreadxDisplayConnection {
    #[inline]
    pub fn new() -> crate::Result<Self> {
        Ok(Self::from_display(Display::create(None, None)?))
    }
}

impl<Conn> BreadxDisplay<Conn> {
    #[inline]
    pub fn from_display(display: Display<Conn>) -> Self {
        Self {
            display,
            closed: false,
        }
    }
}

impl<Conn: Connection> GtDisplay for BreadxDisplay<Conn> {
    #[inline]
    fn screens(&mut self) -> crate::Result<ScreenIter<'_>> {
        Ok(ScreenIter::from_range(0, self.display.screens().len()))
    }

    #[inline]
    fn default_screen(&mut self) -> crate::Result<Screen> {
        Ok(Screen::from_raw(self.display.default_screen_index()))
    }

    #[inline]
    fn screen_dimensions(&mut self, screen: Screen) -> crate::Result<(u32, u32)> {
        let l = self.display.screens();
        let s = &l[screen.into_raw()];
        Ok((s.width_in_pixels.into(), s.height_in_pixels.into()))
    }

    #[inline]
    fn toplevel_window(&mut self, screen: Screen) -> crate::Result<Window> {
        let l = self.display.screens();
        let s = &l[screen.into_raw()];
        Ok(Window::from_raw(
            NonZeroU64::new(s.root.xid.into()).expect("root shouldnt be zero?"),
        ))
    }

    #[inline]
    fn default_toplevel_window(&mut self) -> crate::Result<Window> {
        Ok(Window::from_raw(
            NonZeroU64::new(self.display.default_root().xid.into()).expect("dr shouldn't be zero?"),
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
        window_props: WindowProps,
    ) -> crate::Result<Window> {
        let WindowProps {
            background_color,
            border_color,
            border_width,
        } = window_props;

        let cmap = self.display.default_colormap();
        let params = WindowParameters {
            background_pixel: Some(match background_color {
                Some(color) => {
                    let (r, g, b, _) = color.clamp_u16();
                    cmap.alloc_color_immediate(&mut self.display, r, g, b)?
                        .pixel()
                }
                None => self.display.default_white_pixel(),
            }),
            border_pixel: Some(match border_color {
                Some(color) => {
                    let (r, g, b, _) = color.clamp_u16();
                    cmap.alloc_color_immediate(&mut self.display, r, g, b)?
                        .pixel()
                }
                None => self.display.default_black_pixel(),
            }),
            ..Default::default()
        };
        let window = self.display.create_window(
            XWindow::const_from_xid(parent.into_raw().get() as u32),
            WindowClass::InputOutput,
            None,
            None,
            x.try_into().expect("x"),
            y.try_into().expect("y"),
            width.try_into().expect("width"),
            height.try_into().expect("height"),
            border_width.try_into().expect("border_width"),
            params,
        )?;

        Ok(Window::from_raw(
            NonZeroU64::new(window.xid.into()).unwrap(),
        ))
    }

    #[inline]
    fn destroy_window(&mut self, window: Window) -> crate::Result {
        let window = XWindow::const_from_xid(window.into_raw().get() as u32);
        window.free(&mut self.display)?;
        Ok(())
    }

    #[inline]
    fn window_geometry(&mut self, window: Window) -> crate::Result<(i32, i32, u32, u32)> {
        let window = XWindow::const_from_xid(window.into_raw().get() as u32);
        let Geometry {
            x,
            y,
            width,
            height,
            ..
        } = window.geometry_immediate(&mut self.display)?;
        Ok((x.into(), y.into(), width.into(), height.into()))
    }

    #[inline]
    fn window_parent(&mut self, window: Window) -> crate::Result<Option<Window>> {
        let window = XWindow::const_from_xid(window.into_raw().get() as u32);
        let Geometry { root, .. } = window.geometry_immediate(&mut self.display)?;
        Ok(NonZeroU64::new(root.xid.into()).map(|root| Window::from_raw(root)))
    }

    #[inline]
    fn window_set_visibility(&mut self, window: Window, visibility: Visibility) -> crate::Result {
        let window = XWindow::const_from_xid(window.into_raw().get() as u32);

        match visibility {
            Visibility::Visible => window.map(&mut self.display)?,
            Visibility::Hidden => window.unmap(&mut self.display)?,
        }

        Ok(())
    }

    #[inline]
    fn window_set_geometry(
        &mut self,
        window: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> crate::Result {
        let window = XWindow::const_from_xid(window.into_raw().get() as u32);
        window.move_resize(&mut self.display, x, y, width, height)?;
        Ok(())
    }

    #[inline]
    fn window_set_size(&mut self, window: Window, width: u32, height: u32) -> crate::Result {
        let window = XWindow::const_from_xid(window.into_raw().get() as u32);
        window.resize(&mut self.display, width, height)?;
        Ok(())
    }

    #[inline]
    fn window_set_coordinates(&mut self, window: Window, x: i32, y: i32) -> crate::Result {
        let window = XWindow::const_from_xid(window.into_raw().get() as u32);
        window.configure(
            &mut self.display,
            ConfigureWindowParameters {
                x: Some(x),
                y: Some(y),
                ..Default::default()
            },
        )?;
        Ok(())
    }

    #[inline]
    fn run_with_boxed_event_handler(&mut self, f: EventHandler) -> crate::Result {
        if self.closed {
            return Err(crate::Error::RunAfterClose);
        }

        loop {
            let ev = match self.display.wait_for_event() {
                Ok(ev) => ev,
                Err(breadx::BreadError::ClosedConnection) => return Ok(()),
                Err(e) => return Err(e.into()),
            };

            unimplemented!()
        }

        self.closed = true;
        Ok(())
    }
}
