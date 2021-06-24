// MIT/Apache2 License

#![cfg(unix)]

mod event;

use breadx::{
    auth_info::AuthInfo,
    display::{prelude::*, Display, DisplayBase, DisplayConnection},
    ColorAllocation, Colormap, ConfigureWindowParameters, Gcontext, Window as BreadxWindow,
};
use chalkboard::Color;
use std::{
    borrow::Cow,
    collections::hash_map::{Entry, HashMap},
    mem,
};

#[cfg(feature = "xrender")]
use breadx::render::{Picture, RenderDisplay};
#[cfg(feature = "xrender")]
use chalkboard::breadx::{RenderBreadxSurface, RenderResidual};

#[derive(Debug)]
pub struct BreadxDisplay<Dpy> {
    manager: Manager<Dpy>,
    // note: these hash maps exist largely as an optimization. we could poll the X11 server for these, but it
    //       is faster to just keep them in the client
    colormaps: HashMap<Visualid, Colormap>,
    window_visuals: HashMap<Window, Visualid>,
}

#[derive(Debug)]
enum Manager<Dpy> {
    Placeholder,
    Basic {
        dpy: Dpy,
        colors: HashMap<BreadxWindow, (Gcontext, ColorHashMap)>,
    },
    #[cfg(feature = "xrender")]
    Xrender {
        dpy: RenderDisplay<Dpy>,
        residual: HashMap<BreadxWindow, (Picture, RenderResidual)>,
    },
}

type ColorHashMap = HashMap<Color, u32>;

pub type BreadxDisplayConnection = BreadxDisplay<DisplayConnection>;

impl<Dpy: DisplayBase> BreadxDisplay<Dpy> {
    /// Create a new `BreadxDisplay` from an existing `Display`.
    #[inline]
    pub fn from_display(dpy: Dpy) -> Self {
        assert!(
            mem::size_of::<NonZeroUsize>() >= mem::size_of::<breadx::XID>(),
            "Xid cannot fit into a NonZeroUsize"
        );
        Self {
            window_screens: dpy
                .setup()
                .roots
                .iter()
                .map(|(i, root)| (root.root, root.root_visual))
                .collect(),
            manager: Manager::Basic {
                dpy,
                colors: HashMap::new(),
            },
            colormaps: HashMap::new(),
        }
    }

    /// Get a mutable reference to the inner display.
    #[inline]
    pub(crate) fn display_mut(&mut self) -> &mut Dpy {
        match &mut self.manager {
            Manager::Placeholder => unreachable!(),
            Manager::Basic { dpy, .. } => dpy,
            #[cfg(feature = "xrender")]
            Manager::Xrender { dpy, .. } => dpy.inner_mut(),
        }
    }
}

impl BreadxDisplayConnection {
    /// Create a new `BreadxDisplayConnection` by creating a `DisplayConnection` and then wrapping it.
    #[inline]
    pub fn create<'a>(
        name: Option<Cow<'a, str>>,
        auth_info: Option<AuthInfo>,
    ) -> crate::Result<Self> {
        Ok(Self::from_display(DisplayConnection::create(
            name, auth_info,
        )?))
    }
}

impl<Dpy: Display> BreadxDisplay<Dpy> {
    #[inline]
    pub(crate) fn colormap_for_visual(
        &mut self,
        visual: Visualid,
        subject_window: BreadxWindow,
    ) -> crate::Result<Colormap> {
        match self.colormaps.entry(visual) {
            Entry::Occupied(cmap) => Ok(*cmap.get()),
            Entry::Vacant(v) => {
                // allocate a new colormap
                let cmap = self.display_mut().create_colormap(
                    subject_window,
                    visual,
                    ColormapAlloc::All,
                )?;
                Ok(*v.insert(cmap))
            }
        }
    }
}

#[cfg(feature = "xrender")]
impl<Dpy: Display> BreadxDisplay<Dpy> {
    /// Convert this into an equivalent "xrender" display.
    #[inline]
    pub fn enable_xrender(&mut self) -> crate::Result {
        match &mut self.manager {
            Manager::Placeholder => unreachable!(),
            Manager::Xrender { .. } => {
                log::warn!("Attempted to enable xrender on an already-xrender-enabled display");
                Ok(())
            }
            manager @ Manager::Basic { .. } => {
                let Manager::Basic { dpy, .. } = mem::replace(manager, Manager::Placeholder);

                // try to create a RenderDisplay
                let dpy = match RenderDisplay::new(dpy, 1, 1) {
                    Ok(dpy) => dpy,
                    Err((dpy, err)) => {
                        *manager = Manager::Basic {
                            dpy,
                            colors: HashMap::new(),
                        };
                        return Err(err.into());
                    }
                };

                *manager = Manager::Xrender {
                    dpy,
                    residual: HashMap::new(),
                };
                Ok(())
            }
        }
    }
}

impl<'evh, Dpy: breadx::Display> Display<'evh> for BreadxDisplay<Dpy> {
    #[inline]
    fn screens(&mut self) -> crate::Result<ScreenIter<'_>> {
        Ok(ScreenIter::range(0, self.display_mut().setup().roots.len()))
    }

    #[inline]
    fn default_screen(&mut self) -> crate::Result<Screen> {
        Ok(Screen::from_raw(self.display_mut().default_screen_index()))
    }

    #[inline]
    fn screen_dimensions(&mut self, screen: Screen) -> crate::Result<(u32, u32)> {
        let root = &self
            .display_mut()
            .setup()
            .roots
            .get(screen.into_raw())
            .ok_or(crate::Error::NoScreen(screen.into_raw()))?;
        Ok((root.width_in_pixels.into(), root.height_in_pixels.into()))
    }

    #[inline]
    fn toplevel_window(&mut self, screen: Screen) -> crate::Result<Window> {
        Ok(cvt_window_r(
            self.display_mut()
                .setup()
                .roots
                .get(screen.into_raw())
                .ok_or(crate::Error::NoScreen(screen.into_raw()))?
                .root,
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
        mut props: WindowProps,
    ) -> crate::Result<Window> {
        // figure out what visual our parent and thus, our child will use
        // TODO: this will be different if we do OpenGL
        let visual = self
            .window_visuals
            .get(&parent)
            .copied()
            .unwrap_or(crate::Error::NotOurWindow(parent.into_raw()))?;
        let cmap = self.colormap_for_visual(visual, cvt_window(parent))?;

        let border_width: u16 = props
            .border_width
            .take()
            .unwrap_or(0)
            .try_into()
            .expect("Border width is greater than the max size u16, this is likely a mistake");

        // figure out which parameters to give our window
        let ParameterizeAndAtomSetting {
            parameterizer,
            strings_to_set,
        } = window_props_to_adjustor(self.display_mut(), cmap, props)?;

        // create the window
        let window = self.display_mut().create_window(
            cvt_window(parent),
            WindowClass::CopyFromParent,
            None,
            Some(visual),
            x as i16,
            y as i16,
            width as u16,
            height as u16,
            border_width,
            parameterizer,
        )?;
        let window = cvt_window_r(window);

        // install the window into our window_screens map
        self.window_visuals.insert(window, visual);

        Ok(window)
    }

    #[inline]
    fn delete_window(&mut self, window: Window) -> crate::Result {
        if self.window_visuals.remove(&window).is_none() {
            return Err(crate::Error::NotOurWindow(window.into_raw())?);
        }

        let window = cvt_window(window);
        window.free(self.display_mut())?;
        Ok(())
    }

    #[inline]
    fn window_dimensions(&mut self, window: Window) -> crate::Result<Dimensions> {
        let window = cvt_window(window);
        let Geometry {
            x,
            y,
            width,
            height,
            ..
        } = window.geometry_immediate(self.display_mut())?;
        Ok(Dimensions {
            x,
            y,
            width,
            height,
        })
    }

    // window_size and window_coordinates are automatically implemented and use window_dimensions automatically

    #[inline]
    fn window_set_dimensions(
        &mut self,
        window: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> crate::Result {
        let window = cvt_window(window);
        let params = ConfigureWindowParameters {
            x: Some(x),
            y: Some(y),
            width: Some(width),
            height: Some(height),
            ..Default::default()
        };
        window.configure(self.display_mut(), params)?;
        Ok(())
    }

    #[inline]
    fn window_set_coordinates(&mut self, window: Window, x: i32, y: i32) -> crate::Result {
        let window = cvt_window(window);
        let params = ConfigureWindowParameters {
            x: Some(x),
            y: Some(y),
            ..Default::default()
        };
        window.configure(self.display_mut(), params)?;
        Ok(())
    }

    #[inline]
    fn window_set_size(&mut self, window: Window, width: u32, height: u32) -> crate::Result {
        let window = cvt_window(window);
        let params = ConfigureWindowParameters {
            width: Some(width),
            height: Some(height),
            ..Default::default()
        };
        window.configure(self.display_mut(), params)?;
        Ok(())
    }

    #[inline]
    fn draw_with_boxed_draw_handler(
        &mut self,
        window: Window,
        handler: DrawHandler<'_>,
    ) -> crate::Result {
        let window = cvt_window(window);
        #[cfg(feature = "xrender")]
        let visual = self.window_visuals.get(&window).copied().unwrap();

        // call the handler with a draw handle that we construct
        match &mut self.manager {
            Manager::Placeholder => unreachable!(),
            Manager::Basic { dpy, colors } => {
                let (gc, color) = match colors.remove(&window) {
                    Some(gccolor) => gccolor,
                    None => (
                        dpy.create_gc(
                            window,
                            GcParameters {
                                graphics_exposures: Some(0),
                                ..Default::default()
                            },
                        )?,
                        HashMap::new(),
                    ),
                };

                let mut surface =
                    FallbackBreadxSurface::with_cached_colormap(dpy, window, gc, color);
                let res = handler(&mut surface);

                // insert gc and color back into surface
                colors.insert((gc, surface.into_colormap()));

                res
            }
            #[cfg(feature = "xrender")]
            Manager::Xrender { dpy, residual } => {
                let Geometry { width, height, .. } = window.geometry_immediate(dpy)?;
                let (picture, mut surface) = match residual.remove(&window) {
                    Some((picture, residual)) => (
                        picture,
                        RenderBreadxSurface::from_residual(
                            dpy, picture, window, width, height, residual,
                        ),
                    ),
                    None => {
                        // create a picture
                        let pictformat = dpy
                            .find_visual_format(dpy.inner().visual_id_to_visual(visual).unwrap())
                            .unwrap();
                        let picture = dpy.create_picture(
                            window,
                            pictformat,
                            PictureParameters {
                                graphics_exposure: Some(0),
                                ..Default::default()
                            },
                        )?;
                        (
                            picture,
                            RenderBreadxSurface::new(dpy, picture, window, width, height)?,
                        )
                    }
                };

                let res = handler(&mut surface);

                residual.insert((picture, surface.into_residual()));

                res
            }
        }
    }

    #[inline]
    fn window_parent(&mut self, window: Window) -> crate::Result<Option<Window>> {
        let Geometry { parent, .. } = cvt_window(window).geometry_immediate(self.display_mut())?;
        Ok(NonZeroUsize::new(parent.xid as usize).map(Window::from_raw))
    }

    #[inline]
    fn run_with_boxed_event_handler(&mut self, handler: EventHandler<'evh, Self>) -> crate::Result {
        loop {
            // note: return on error, since the display is probably not sane if we error out
            let ev = match self.display_mut().wait_for_event() {
                Ok(ev) => ev,
                Err(breadx::BreadError::ClosedConnection) => return Ok(()),
                Err(e) => return Err(e.into()),
            };

            if let Some(ev) = self.convert_event(ev)? {
                handler(self, ev)?;
            }
        }

        // free resources
        match &mut self.manager {
            Manager::Placeholder => unimplemented!(),
            Manager::Basic { dpy, colors } => {
                let colors = mem::take(colors);
                colors
                    .into_iter()
                    .try_for_each::<_, breadx::Result>(|(_, (gc, _))| gc.free(dpy))?;
            }
            #[cfg(feature = "xrender")]
            Manager::Xrender { dpy, residual } => {
                let rr = mem::take(residual);
                rr.into_iter()
                    .try_for_each::<_, crate::Result>(|(_, (picture, residual))| {
                        picture.free(dpy)?;
                        residual.free(dpy)
                    })?;
            }
        }

        Ok(())
    }
}

#[derive(Default)]
struct ParameterizerAndAtomSetting {
    parameterizer: WindowParameters,
    strings_to_set: Vec<(&'static str, String)>,
}

#[inline]
fn cvt_window(win: Window) -> BreadxWindow {
    BreadxWindow::const_from_xid(win.into_raw().get().try_into().expect("Not our window"))
}

#[inline]
fn cvt_window_r(win: BreadxWindow) -> Window {
    let n = NonZeroUsize::new(win.xid as usize)
        .expect("Window should never be 0 unless there is an error");
    Window::from_raw(n)
}

#[inline]
fn window_props_to_adjustor<Dpy: Display>(
    dpy: &mut Dpy,
    cmap: Colormap,
    window_props: WindowProps,
) -> crate::Result<ParameterizerAndAtomSetting> {
    let mut paas = Default::default();
    let WindowProps {
        title,
        background,
        border_color,
    } = window_props;

    if let Some(title) = title {
        paas.strings_to_set.push(("WM_TITLE", title));
    }

    let background_pixel_alloc_token = match background {
        None => None,
        Some(FillRule::SolidColor(clr)) => {
            let (r, g, b) = clr.clamp_u16();
            Some(cmap.alloc_color(dpy, r, g, b)?.pixel());
        }
        _ => {
            return Err(crate::Error::StaticMsg(
                "X11 does not support gradients as backgrounds",
            ))
        }
    };

    let border_pixel_alloc_token = match border_color {
        Some(clr) => {
            let (r, g, b) = clr.clamp_u16();
            Some(cmap.alloc_color(dpy, r, g, b)?.pixel())
        }
        None => None,
    };

    // resolve for any tokens we sent
    if let Some(bpat) = background_pixel_alloc_token {
        paas.parameterize.background_pixel = Some(dpy.resolve_request(bpat)?.pixel);
    }

    if let Some(bpat) = border_pixel_alloc_token {
        paas.parameterize.border_pixel = Some(dpy.resolve_request(bpat)?.pixel);
    }

    Ok(paas)
}
