// MIT/Apache2 License

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

/// A rectangle of pixels on the screen.
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

    /// The universal ID of the window.
    #[inline]
    pub fn id(&self) -> usize {
        self.internal.id()
    }

    /// Set the size of the window.
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

    /// The width and height of the window.
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        let p = self.properties.read();
        (p.width, p.height)
    }

    /// Set the event mask.
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

    /// Set the location of this surface.
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

    #[inline]
    pub fn set_background_color(&self, clr: Rgba) -> crate::Result<()> {
        let mut p = self.properties.write();
        p.background_color = clr;
        mem::drop(p);

        self.internal.set_background_color(clr)?;
        self.internal.invalidate(None)
    }

    #[inline]
    pub fn set_border_color(&self, clr: Rgba) -> crate::Result<()> {
        let mut p = self.properties.write();
        p.border_color = clr;
        mem::drop(p);

        self.internal.set_border_color(clr)?;
        self.internal.invalidate(None)
    }

    #[inline]
    pub fn set_border_width(&self, width: u32) -> crate::Result<()> {
        let mut p = self.properties.write();
        p.border_width = width;
        mem::drop(p);

        self.internal.set_border_width(width)?;
        self.internal.invalidate(None)
    }

    #[inline]
    pub fn invalidate(&self, rect: Option<Rectangle>) -> crate::Result<()> {
        self.internal.invalidate(rect)
    }
}

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
