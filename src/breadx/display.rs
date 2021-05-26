// MIT/Apache2 License

use crate::{
    display::{Display as GtDisplay, DrawHandler, EventHandler},
    event::Event,
    screen::{Screen, ScreenIter},
    surface::SurfaceSum,
    util::DebugContainer,
    window::{Visibility, Window, WindowProps},
};
use breadx::{
    display::{
        name::NameConnection, ConfigureWindowParameters, Connection, Display, GcParameters,
        Geometry, WindowParameters,
    },
    Gcontext, Window as XWindow, WindowClass,
};
use chalkboard::breadx::FallbackBreadxSurface;
use std::{collections::HashMap, convert::TryInto, mem, num::NonZeroU64};

#[cfg(feature = "xrender")]
use breadx::{
    auto::render::Picture,
    display::WindowAttributes,
    render::{PictureParameters, RenderDisplay, StandardFormat},
};
#[cfg(feature = "xrender")]
use chalkboard::breadx::{RenderBreadxSurface, RenderResidual};

/// A display that internally uses a `breadx::Display` to create a GUI environment.
#[derive(Debug)]
pub struct BreadxDisplay<Conn> {
    display: InnerDisplay<Conn>,
    closed: bool,
}

#[derive(Debug)]
enum InnerDisplay<Conn> {
    Fallback {
        display: Display<Conn>,
        cached_maps: HashMap<XWindow, CachedMap>,
    },
    #[cfg(feature = "xrender")]
    Xrender {
        display: RenderDisplay<Display<Conn>>,
        residuals: HashMap<XWindow, XrenderResidual>,
    },
}

#[derive(Debug)]
struct CachedMap {
    gc: Gcontext,
    colormap: HashMap<chalkboard::Color, u32>,
}

#[cfg(feature = "xrender")]
#[derive(Debug)]
struct XrenderResidual {
    rr: RenderResidual,
    picture: Picture,
}

pub type BreadxDisplayConnection = BreadxDisplay<NameConnection>;

impl BreadxDisplayConnection {
    #[inline]
    pub fn new() -> crate::Result<Self> {
        Ok(Self::from_display(Display::create(None, None)?))
    }
}

impl<Conn: Connection> BreadxDisplay<Conn> {
    #[inline]
    pub fn from_display(display: Display<Conn>) -> Self {
        Self {
            display: InnerDisplay::divine(display),
            closed: false,
        }
    }

    #[inline]
    fn display(&self) -> &Display<Conn> {
        self.display.display()
    }

    #[inline]
    fn display_mut(&mut self) -> &mut Display<Conn> {
        self.display.display_mut()
    }

    #[inline]
    fn cvt_color(&mut self, screen: Screen) -> crate::Result<u32> {
        let cmap = self.display().default_colormap();
        let (r, g, b, _) = color.clamp_u16();
        cmap.alloc_color_immediate(self.display_mut(), r, g, b)?
            .pixel()
    }
}

impl<Conn: Connection> InnerDisplay<Conn> {
    #[inline]
    fn divine(dpy: Display<Conn>) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(feature = "xrender")] {
                match RenderDisplay::new(dpy, 1, 1) {
                    Ok(display) => InnerDisplay::Xrender { display, residuals: HashMap::new() },
                    Err((display, _)) => InnerDisplay::Fallback { display, cached_maps: HashMap::new() },
                }
            } else {
                InnerDisplay::Fallback { display: dpy, cached_maps: HashMap::new() },
            }
        }
    }

    #[inline]
    fn display(&self) -> &Display<Conn> {
        match self {
            InnerDisplay::Fallback { display, .. } => display,
            #[cfg(feature = "xrender")]
            InnerDisplay::Xrender { display, .. } => display.inner(),
        }
    }

    #[inline]
    fn display_mut(&mut self) -> &mut Display<Conn> {
        match self {
            InnerDisplay::Fallback { display, .. } => display,
            #[cfg(feature = "xrender")]
            InnerDisplay::Xrender { display, .. } => display.inner_mut(),
        }
    }
}

//impl<Conn: Connection> GtDisplay for BreadxDisplay<Conn> {
impl GtDisplay for BreadxDisplay<NameConnection> {
    #[inline]
    fn screens(&mut self) -> crate::Result<ScreenIter<'_>> {
        Ok(ScreenIter::from_range(0, self.display().screens().len()))
    }

    #[inline]
    fn default_screen(&mut self) -> crate::Result<Screen> {
        Ok(Screen::from_raw(self.display().default_screen_index()))
    }

    #[inline]
    fn screen_dimensions(&mut self, screen: Screen) -> crate::Result<(u32, u32)> {
        let l = self.display().screens();
        let s = &l[screen.into_raw()];
        Ok((s.width_in_pixels.into(), s.height_in_pixels.into()))
    }

    #[inline]
    fn toplevel_window(&mut self, screen: Screen) -> crate::Result<Window> {
        let l = self.display().screens();
        let s = &l[screen.into_raw()];
        Ok(Window::from_raw(
            NonZeroU64::new(s.root.xid.into()).expect("root shouldnt be zero?"),
        ))
    }

    #[inline]
    fn default_toplevel_window(&mut self) -> crate::Result<Window> {
        Ok(Window::from_raw(
            NonZeroU64::new(self.display().default_root().xid.into())
                .expect("dr shouldn't be zero?"),
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
            title,
            background_color,
            border_color,
            border_width,
        } = window_props;

        // todo: figure out the proper screen
        let params = WindowParameters {
            background_pixel: Some(match background_color {
                Some(color) => self.cvt_color(color)?,
                None => self.display().default_white_pixel(),
            }),
            border_pixel: Some(match border_color {
                Some(color) => self.cvt_color(color)?,
                None => self.display().default_black_pixel(),
            }),
            ..Default::default()
        };
        let window = self.display_mut().create_window(
            XWindow::const_from_xid(parent.into_raw().get() as u32),
            WindowClass::InputOutput,
            None,
            None,
            x.try_into().expect("x"),
            y.try_into().expect("y"),
            width.try_into().expect("width"),
            height.try_into().expect("height"),
            border_width.unwrap_or(0).try_into().expect("border_width"),
            params,
        )?;

        // apply window properties that we haven't used yet
        if let Some(ref title) = title {
            window.set_title(self.display_mut(), title)?;
        }

        Ok(Window::from_raw(
            NonZeroU64::new(window.xid.into()).unwrap(),
        ))
    }

    #[inline]
    fn destroy_window(&mut self, window: Window) -> crate::Result {
        let window = XWindow::const_from_xid(window.into_raw().get() as u32);
        window.free(self.display_mut())?;
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
        } = window.geometry_immediate(self.display_mut())?;

        Ok((x.into(), y.into(), width.into(), height.into()))
    }

    #[inline]
    fn window_parent(&mut self, window: Window) -> crate::Result<Option<Window>> {
        let window = XWindow::const_from_xid(window.into_raw().get() as u32);
        let Geometry { root, .. } = window.geometry_immediate(self.display_mut())?;
        Ok(NonZeroU64::new(root.xid.into()).map(|root| Window::from_raw(root)))
    }

    #[inline]
    fn window_set_visibility(&mut self, window: Window, visibility: Visibility) -> crate::Result {
        let window = XWindow::const_from_xid(window.into_raw().get() as u32);

        match visibility {
            Visibility::Visible => window.map(self.display_mut())?,
            Visibility::Hidden => window.unmap(self.display_mut())?,
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
        window.move_resize(self.display_mut(), x, y, width, height)?;
        Ok(())
    }

    #[inline]
    fn window_set_properties(&mut self, window: Window, props: WindowProps) -> crate::Result {
        let window = XWindow::const_from_xid(window.into_raw().get() as u32);
        let WindowProps {
            border_width,
            border_color,
            background_color,
            title,
        } = window;

        if let Some(border_width) = border_width {
            window.configure(
                self.display_mut(),
                ConfigureWindowParameters {
                    border_width: border_width.try_into().expect("border_width"),
                    ..Default::default()
                },
            )?;
        }

        if border_color.is_some() || background_color.is_some() {
            let mut wp = WindowParameters::default();
            if let Some(border_color) = border_color {
                wp.border_pixel = self.cvt_color(border_color)?;
            }
            if let Some(background_color) = background_color {
                wp.background_pixel = self.cvt_color(background_color)?;
            }
            window.change_attributes(self.display_mut(), wp)?;
        }

        if let Some(ref title) = title {
            window.set_title(self.dsiplay_mut(), title)?;
        }

        Ok(())
    }

    #[inline]
    fn window_set_size(&mut self, window: Window, width: u32, height: u32) -> crate::Result {
        let window = XWindow::const_from_xid(window.into_raw().get() as u32);
        window.resize(self.display_mut(), width, height)?;
        Ok(())
    }

    #[inline]
    fn window_set_coordinates(&mut self, window: Window, x: i32, y: i32) -> crate::Result {
        let window = XWindow::const_from_xid(window.into_raw().get() as u32);
        window.configure(
            self.display_mut(),
            ConfigureWindowParameters {
                x: Some(x),
                y: Some(y),
                ..Default::default()
            },
        )?;
        Ok(())
    }

    #[inline]
    fn window_draw_with_boxed_drawer(
        &mut self,
        window: Window,
        draw_handler: DrawHandler<'_>,
    ) -> crate::Result {
        let window = XWindow::const_from_xid(window.into_raw().get() as u32);

        match &mut self.display {
            InnerDisplay::Fallback {
                display,
                cached_maps,
            } => {
                // load up a colormap if we already have one
                let (gc, surface) = match cached_maps.remove(&window) {
                    Some(CachedMap { gc, colormap }) => (
                        gc,
                        FallbackBreadxSurface::with_cached_colormap(display, window, gc, colormap),
                    ),
                    None => {
                        // we need to create a gc for the window
                        let gc = display.create_gc(
                            window,
                            GcParameters {
                                graphics_exposures: Some(0),
                                ..Default::default()
                            },
                        )?;
                        (gc, FallbackBreadxSurface::new(display, window, gc))
                    }
                };

                let mut surface = SurfaceSum::FallbackBreadx(surface);

                // run the draw handler
                (draw_handler)(&mut surface)?;

                // deconstruct the surface into a colormap
                let colormap = match surface {
                    SurfaceSum::FallbackBreadx(surface) => surface.into_colormap(),
                    _ => panic!("Surface was moved out of reference and was not returned!"),
                };

                // cache the colormap and the gc
                cached_maps.insert(window, CachedMap { gc, colormap });
            }
            #[cfg(feature = "xrender")]
            InnerDisplay::Xrender { display, residuals } => {
                // load up the residual, or create one from scratch
                let Geometry { width, height, .. } =
                    window.geometry_immediate(display.inner_mut())?;
                let (picture, surface) = match residuals.remove(&window) {
                    Some(XrenderResidual { rr, picture }) => (
                        picture,
                        RenderBreadxSurface::from_residual(
                            display, picture, window, width, height, rr,
                        )?,
                    ),
                    None => {
                        // create a picture for the window
                        let WindowAttributes { visual, .. } =
                            window.window_attributes_immediate(display.inner_mut())?;
                        let pictformat = display
                            .find_visual_format(display.inner().visual_id_to_visual(visual).ok_or(
                                crate::Error::StaticMsg(
                                    "Could not find matching Visual ID for visual",
                                ),
                            )?)
                            .ok_or(crate::Error::StaticMsg(
                                "Could not find matching Pictformat for visual",
                            ))?;

                        let picture = display.create_picture(
                            window,
                            pictformat,
                            PictureParameters {
                                graphics_exposure: Some(1),
                                ..Default::default()
                            },
                        )?;
                        (
                            picture,
                            RenderBreadxSurface::new(display, picture, window, width, height)?,
                        )
                    }
                };

                let mut surface = SurfaceSum::XrenderBreadx(surface);

                (draw_handler)(&mut surface)?;

                let residual = match surface {
                    SurfaceSum::XrenderBreadx(s) => s.into_residual(),
                    _ => panic!("Surface was moved out of reference and it was not returned!"),
                };

                residuals.insert(
                    window,
                    XrenderResidual {
                        rr: residual,
                        picture,
                    },
                );
            }
        }

        Ok(())
    }

    #[inline]
    fn run_with_boxed_event_handler(&mut self, mut f: EventHandler) -> crate::Result {
        if self.closed {
            return Err(crate::Error::RunAfterClose);
        }

        self.closed = true;

        loop {
            let ev = match self.display_mut().wait_for_event() {
                Ok(ev) => ev,
                Err(breadx::BreadError::ClosedConnection) => return Ok(()),
                Err(e) => return Err(e.into()),
            };

            unimplemented!()
        }

        // destroy all of our cached residuals, if possible
        match &mut self.display {
            InnerDisplay::Fallback {
                display,
                cached_maps,
            } => {
                let maps = mem::take(cached_maps);
                maps.into_iter().try_for_each::<_, breadx::Result>(
                    |(_, CachedMap { gc, .. })| gc.free(display),
                )?;
            }
            #[cfg(feature = "xrender")]
            InnerDisplay::Xrender { display, residuals } => {
                let res = mem::take(residuals);
                res.into_iter().try_for_each::<_, chalkboard::Result>(
                    |(_, XrenderResidual { rr, picture })| {
                        picture.free(display.inner_mut())?;
                        rr.free(display.inner_mut())
                    },
                )?;
            }
        }

        Ok(())
    }
}
