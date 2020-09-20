// MIT/Apache2 License

//! Surfaces, in the most technical definition, are rectangles of pixels that can contain other rectangles
//! of pixels, and be drawn into. Surfaces are the building blocks of `gui-tools`.
//!
//! # What is a surface?
//!
//! The `Surface` in `gui-tools` is analagous to the `GdkSurface` from GDK, the `Window` from Xlib, and the
//! `HWND` from Win32. However, these terms tend to be confusing. We grew up with *Windows*, a GUI-based OS famous
//! for letting us open *windows* full of *widgets*. When you're browsing the web and you right click a link,
//! you could "Open Link in New *Window*". I find it easier to learn with a demonstration.
//!
//! Imagine a computer monitor.
//!
//! <div>
//! <img src="https://github.com/not-a-seagull/gui-tools/raw/master/images/surface/monitor.jpg"
//!      alt="Computer Monitor" width="300px" />
//! <p><sub>
//!    <a href="https://upload.wikimedia.org/wikipedia/commons/a/a4/Computer_monitor.jpg">Source</a>,
//!    public domain
//! </sub></p>
//! </div>
//!
//! Now imagine it's time for work. You boot up your computer, and you're greeted with your desktop background.
//! After you're done admiring your desktop background, you open some applications: a web browser and a
//! calculator.
//!
//! <div>
//! <img src="https://github.com/not-a-seagull/gui-tools/raw/master/images/surface/surface1.jpg"
//!      width="300px" />
//! </div>
//!
//! Ask yourself: how many surfaces, or windows, have you opened? You might say two; the calculator and the web
//! browser. However, the real answer, in a broad sense, is three. There's the calculator, the web browser,
//! and the root window for the monirot.
//!
//! <div>
//! <img src="https://github.com/not-a-seagull/gui-tools/raw/master/images/surface/surface2.jpg"
//!      width="300px" />
//! </div>
//!
//! (Note: This doesn't include whatever surfaces are created as a result of any kind of "Start" menu
//! or toolbar.)
//!
//! The answer isn't really three, however. Let's take a look at that calculator. Let's assume it's a relatively
//! pritimitive calculator. It has sixteen buttons: ten for the digits (0-9), and six for the basic functions
//! (addition, subtraction, multiplication, division, evaluation, and clearing). In addition to this, you have
//! the label that contains the current operand or the result.
//!
//! <div>
//! <img src="https://github.com/not-a-seagull/gui-tools/raw/master/images/surface/surface3.jpg"
//!     width="300px" />
//! </div>
//!
//! It turns out, these widgets (the buttons and the label) are all usually implemented as surfaces themselves.
//! Rather than being children of the monitor root window, they are children of the calculator surface.
//!
//! <div>
//! <img src="https://github.com/not-a-seagull/gui-tools/raw/master/images/surface/surface3.jpg"
//!      width="300px" />
//! </div>
//!
//! Therefore, the calculator app by itself consists of 17 surfaces. Let's not even count the number of surfaces
//! in the web browser; the implementation of that varies from engine to engine.
//!
//! What most people call windows are surfaces. In addition to this, what most people call widgets are surfaces
//! as well. This differentation happens at the toolkit level more often than not.
//!
//! **TL;DR:** Your windows are sufaces. Your widgets are surfaces. Even your monitors are surfaces.
//!
//! # Instatiation
//!
//! The [`SurfaceInitialization`] structure is used to create surfaces. It contains all of the properties that a
//! surface will need at initialiation. This ensures that the API's for surface creation won't need an
//! increasingly long number of arguments.
//!
//! Once you create the surface initialization structure, it should be passed into [`Runtime::create_surface`].
//!
//! ## Example
//!
//! ```no_run
//! use gui_tools::{color::Rgba, runtime::Runtime, surface::{SurfaceInitialization, StartingPoint}};
//!
//! // create the runtime
//! let runtime = Runtime::new().unwrap();
//!
//! // build the surface initialization
//! let mut surface_init = SurfaceInitialization::new(
//!    None, // Parent, or none if this is a top-level window
//!    StartingPoint::Center, // where the window starts
//!    300, 200, // width and height
//!    "Calculator", // window title
//! );
//!
//! // set auxillary properties
//! surface_init.background_color = Rgba::new(0.0, 0.0, 1.0, 1.0).unwrap();
//!
//! // now, create the surface
//! let surface: usize = runtime.create_surface(surface_init).unwrap();
//! ```
//!
//! Note that the above function returns a `usize`, rather than any kind of surface. This is the surface ID.
//! The documentation for [`Surface::id`] function has more information on this, but know that it is unique
//! for every surface. If you'd like to access the surface proper, use it as an argument to
//! [`Runtime::surface_at`].
//!
//! ```no_run
//! # use gui_tools::{runtime::Runtime, surface::{SurfaceInitializaiton, StartingPoint}};
//! # let runtime = Runtime::new().unwrap();
//! # let surface = runtime.create_surface(SurfaceInitialization::new(
//! #                 None, StartingPoint::Center, 1, 1, "t"));
//! let size = runtime.surface_at(surface).unwrap().size();
//! ```
//!
//! [`Runtime::surface_at`] returns an `Option`, so it must be unwrapped. In addition, note that these functions
//! use mutex locks in the current implementation. A more atomic way of doing this is coming soon.

use crate::{
    backend::{win32::Win32Surface, x11::X11Surface, SurfaceInner},
    color::{colors, Rgba},
    event::EventTypeMask,
    geometry::Rectangle,
    graphics::GraphicsInternal,
    mutex::ShimRwLock,
    runtime::Runtime,
    string::CoolString,
};
use core::{mem, ptr::NonNull};
use storagevec::StorageVec;

mod starting_point;

pub use starting_point::StartingPoint;

/// The properties that a surface can hold.
#[derive(Debug)]
pub struct SurfaceProperties {
    pub parent: Option<usize>,
    pub children: StorageVec<usize, 12>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub title: CoolString,
    pub event_mask: StorageVec<EventTypeMask, 20>,

    pub background_color: Rgba,
    pub border_color: Rgba,
    pub border_width: u32,
}

/// The properties that a surface is initialized with.
#[derive(Debug)]
pub struct SurfaceInitialization {
    pub parent: Option<usize>,
    pub children: StorageVec<usize, 12>,
    pub starting_point: StartingPoint,
    pub width: u32,
    pub height: u32,
    pub title: CoolString,
    pub event_mask: StorageVec<EventTypeMask, 20>,

    pub background_color: Rgba,
    pub border_color: Rgba,
    pub border_width: u32,
}

impl SurfaceProperties {
    #[inline]
    pub fn new(
        parent: Option<usize>,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        title: CoolString,
    ) -> Self {
        Self {
            parent,
            x,
            y,
            width,
            height,
            title,
            children: StorageVec::new(),
            event_mask: StorageVec::new(),
            background_color: colors::WHITE,
            border_color: colors::BLACK,
            border_width: 0,
        }
    }
}

impl SurfaceInitialization {
    /// Create a new SurfaceInitialization with some defaults.
    #[inline]
    pub fn new<Title: Into<CoolString>>(
        parent: Option<usize>,
        starting_point: StartingPoint,
        width: u32,
        height: u32,
        title: Title,
    ) -> Self {
        Self {
            parent,
            starting_point,
            width,
            height,
            title: title.into(),
            children: StorageVec::new(),
            event_mask: StorageVec::new(),
            background_color: colors::WHITE,
            border_color: colors::BLACK,
            border_width: 0,
        }
    }

    #[inline]
    pub fn into_properties(self, parent_width: u32, parent_height: u32) -> SurfaceProperties {
        let (x, y) =
            self.starting_point
                .to_x_y(self.width, self.height, parent_width, parent_height);
        let SurfaceInitialization {
            parent,
            width,
            height,
            children,
            title,
            event_mask,
            background_color,
            border_color,
            border_width,
            ..
        } = self;
        let mut sp = SurfaceProperties::new(parent, x, y, width, height, title);
        sp.children = children;
        sp.event_mask = event_mask;
        sp.background_color = background_color;
        sp.border_color = border_color;
        sp.border_width = border_width;
        sp
    }
}

/// A rectangle of pixels on the screen. This object is the primary building block of GUIs in `gui-tools`.
///
/// See the module-level documentation for further information.
pub struct Surface {
    properties: ShimRwLock<SurfaceProperties>,
    internal: SurfaceInner,
}

impl Surface {
    pub(crate) fn new(runtime: &Runtime, properties: SurfaceInitialization) -> crate::Result<Self> {
        let (width, height) = match properties.parent {
            Some(parent) => runtime.surface_at(parent).unwrap().size(),
            None => runtime.default_monitor().unwrap().size(),
        };

        Ok(Self {
            internal: runtime.backend().surface(runtime, &properties)?,
            properties: ShimRwLock::new(properties.into_properties(width, height)),
        })
    }

    #[inline]
    pub(crate) fn as_x11(&self) -> Option<&X11Surface> {
        self.internal.as_x11()
    }

    #[inline]
    pub(crate) fn as_win32(&self) -> Option<&Win32Surface> {
        self.internal.as_win32()
    }

    /// The universal ID of the window. This is guaranteed to be unique for every window.
    ///
    /// The implementation of `id()` varies from backend to backend. It should correspond to some
    /// internal details of a window. For instance, on the X11 backend, it corresponds to the window
    /// ID provided by the display connection. For the Win32 backend, it corresponds to the address
    /// held by the window handle.
    #[inline]
    pub fn id(&self) -> usize {
        self.internal.id()
    }

    /// Set the size of the window. Note that this sends [`EventType::Resized`](../event/enum.EventType.html)
    ///  to the runtime.
    #[inline]
    pub fn set_size(&self, width: u32, height: u32) -> crate::Result<()> {
        self.internal.set_size(width, height)?;
        // the backend should call set_size_no_backend
        Ok(())
    }

    #[inline]
    pub(crate) fn set_size_no_backend(&self, width: u32, height: u32) {
        let mut p = self.properties.write();
        p.width = width;
        p.height = height;
    }

    /// The width and height of the window in pixels.
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        let p = self.properties.read();
        (p.width, p.height)
    }

    /// Set the event mask. The event masks defines which types of events this surface
    /// is guaranteed to generate. See the documentation for [`EventTypeMask`](../event/enum.EventTypeMask.html)
    /// for more information.
    ///
    /// # Errors
    ///
    /// Any errors created by the backend are propogated to this function.
    #[inline]
    pub fn set_event_mask(&self, em: &[EventTypeMask]) -> crate::Result<()> {
        self.properties.write().event_mask = em.iter().cloned().collect();
        self.internal.set_event_mask(em)
    }

    /// Get the location of this surface.
    #[inline]
    pub fn location(&self) -> (i32, i32) {
        let p = self.properties.read();
        (p.x, p.y)
    }

    /// Set the location of this surface. Note that this sends [`EventType::Moved`](../event/enum.EventType.html)
    ///  to the runtime.
    ///
    /// # Errors
    ///
    /// Any errors created by the backend are propogated to this function.
    #[inline]
    pub fn set_location(&self, x: i32, y: i32) -> crate::Result<()> {
        self.internal.set_location(x, y)?;
        // the backend should cal set_location_no_backend
        Ok(())
    }

    #[inline]
    pub(crate) fn set_location_no_backend(&self, x: i32, y: i32) {
        let mut p = self.properties.write();
        p.x = x;
        p.y = y;
    }

    /// Set the background color of this surface. Note that this triggers a repainting.
    ///
    /// # Errors
    ///
    /// Any errors created by the backend are propogated to this function.
    #[inline]
    pub fn set_background_color(&self, clr: Rgba) -> crate::Result<()> {
        let mut p = self.properties.write();
        p.background_color = clr;
        mem::drop(p);

        self.internal.set_background_color(clr)?;
        self.internal.invalidate(None)
    }

    /// Set the border color of this surface. Note that this triggers a repainting.
    ///
    /// # Errors
    ///
    /// Any errors created by the backend are propogated to this function.
    #[inline]
    pub fn set_border_color(&self, clr: Rgba) -> crate::Result<()> {
        let mut p = self.properties.write();
        p.border_color = clr;
        mem::drop(p);

        self.internal.set_border_color(clr)?;
        self.internal.invalidate(None)
    }

    /// Set the width of the border of this surface. Note that this triggers a repainting.
    ///
    /// # Errors
    ///
    /// Any errors created by the backend are propogated to this function.
    #[inline]
    pub fn set_border_width(&self, width: u32) -> crate::Result<()> {
        let mut p = self.properties.write();
        p.border_width = width;
        mem::drop(p);

        self.internal.set_border_width(width)?;
        self.internal.invalidate(None)
    }

    /// Repaints the window by sending an [`EventType::Paint`](../event/enum.EventType.html) to the event runtime.
    ///
    /// This is the preferred method of forcing a repaint on a surface. The `rect` argument is the rectangle
    /// that the repaint targets, or `None` if the entire window ought to be repainted.
    ///
    /// # Errors
    ///
    /// Any errors created by the backend are propogated to this function.
    #[inline]
    pub fn invalidate(&self, rect: Option<Rectangle>) -> crate::Result<()> {
        self.internal.invalidate(rect)
    }
}

/// An object containing details regarding the backend of the surface. Backends should implement this
/// in order to provide surfaces.
pub trait SurfaceBackend {
    fn id(&self) -> usize;
    fn set_event_mask(&self, mask: &[EventTypeMask]) -> crate::Result<()>;
    fn set_size(&self, width: u32, height: u32) -> crate::Result<()>;
    fn set_location(&self, x: i32, y: i32) -> crate::Result<()>;
    fn set_background_color(&self, clr: Rgba) -> crate::Result<()>;
    fn set_border_color(&self, clr: Rgba) -> crate::Result<()>;
    fn set_border_width(&self, width: u32) -> crate::Result<()>;

    fn graphics_internal(&self) -> crate::Result<NonNull<dyn GraphicsInternal>>;
    fn invalidate(&self, rect: Option<Rectangle>) -> crate::Result<()>;
}
