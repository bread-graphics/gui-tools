// MIT/Apache2 License

use crate::{error::x11_status_to_res, image::GenericImage, mutex::{RwLockReadGuard, ShimRwLock}, runtime::Runtime};
use core::{mem, ptr::NonNull};
use cty::{c_uint, c_ulong};
use owning_ref::{OwningRef};
use storagevec::StorageMap;
use x11nas::xlib::{self, Display, _XGC, Pixmap};

pub struct X11Pixmap {
    inner: Pixmap,
    display: NonNull<Display>,
    graphics_context: NonNull<_XGC>, 
    runtime: Runtime,
}

impl Pixmap {
    #[inline]
    pub fn inner(&self) -> Pixmap { self.inner }

    #[inline]
    pub fn display(&self) -> NonNull<Display> { self.display }

    #[inline]
    pub fn gc(&self) -> NonNull<_XGC> { self.graphics_context }

    #[inline]
    pub fn runtime(&self) -> &Runtime { &self.runtime }

    #[inline]
    pub fn from_image(img: &dyn GenericImage, runtime: &Runtime, monitor: c_int) -> crate::Result<Self> {
        let xruntime = runtime.as_x11().unwrap();
        let dpy = xruntime.display();
        let xmonitor = &xruntime.monitors()[monitor as usize];
        let (width, height) = img.size();
        let depth = xmonitor.default_visual().depth();

        // TODO: support anything other than 32-bit RGBA
        assert_eq!(depth, 32);

        // create the space for the pixmap
        let pixmap = unsafe { xlib::XCreatePixmap(xruntime.display().as_ptr(), xmonitor.root_window(), width as c_uint, height as c_uint, depth) };

        // get the image as part of the pixmap
        let image = unsafe { xlib::XGetImage(dpy.as_ptr(), pixmap, 0, 0, width, as c_uint, height as c_uint, 0, xlib::ZPixmap) };
        let image = match NonNull::new(image) {
            Some(image) => image,
            None => return Err(crate::X11Error::BadImage.into()),
        };

        // call to XPutPixel to construct the image
        for x in 0..width {
            for y in 0..height {
                let (r, g, b, _) = image.color_at(x, y).convert_elements::<u8>();
                let color_repr = (65536 * (b as c_ulong)) + (256 * (g as c_ulong)) + (r as c_ulong);
                unsafe { xlib::XPutPixel(image.as_ptr(), x as c_int, y as c_int, color_repr) };
            }
        }

        // get a pointer to the monitor GC while we're at it
        let monitor_gc = unsafe { xlib::XDefaultGC(dpy.as_ptr(), monitor) };
        let monitor_gc = NonNull::new(monitor_gc).unwrap();
 
        // call to XPutImage using the monitor GC
        x11_status_to_res(unsafe { xlib::XPutImage(dpy.as_ptr(), pixmap, monitor_gc.as_ptr(), image.as_ptr(), 0, 0, 0, 0, width as c_uint, height as c_uint) })?;

        // delete the image
        unsafe { xlib::XDestroyImage(image) };   

        // finish up, construct the structure
        Ok(Self { inner: pixmap, display: dpy, graphics_context: monitor_gc, runtime: runtime.clone() })
    }
}

#[repr(transparent)]
pub struct PixmapStorage {
    // matching id's to pixmaps 
    pixmaps: ShimRwLock<StorageMap<usize, X11Pixmap, 12>>,
}

impl PixmapStorage {
    #[inline]
    pub fn register_image(&self, img: &dyn GenericImage, runtime: &Runtime, monitor: c_int) -> crate::Result<OwningRef<RwLockReadGuard<'_, StorageMap<usize, X11Pixmap, 12>>, X11Pixmap>> {
        let id = img.id();
        let pixmaps = self.pixmaps.read();
        match self.pixmaps.contains_key(&id) {
            true => Ok(OwningRef::new(pixmaps).map(move |pm| pixmaps.get(&id).unwrap())),
            false => {
                mem::drop(pixmaps); 
                let mut pixmaps = self.pixmaps.write();
                pixmaps.insert(id, Pixmap::from_image(img, runtime, monitor)?);
                mem::drop(pixmaps);
                Ok(OwningRef::new(self.pixmaps.read()).map(move |pm| pixmaps.get(&id).unwrap()))
            }
        }
    }
}
