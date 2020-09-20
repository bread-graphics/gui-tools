// MIT/Apache2 License

use crate::{
    image::{ColorSpace, GenericImage},
    mutex::{RwLockReadGuard, ShimRwLock as RwLock},
};
use core::{
    mem,
    ptr::{self, NonNull},
};
use euclid::Size2D;
use owning_ref::OwningRef;
use storagevec::StorageMap;
use winapi::um::{
    gdiplusflat,
    gdiplusgpstubs::{GpBitmap, GpImage},
    gdipluspixelformats,
    gdiplustypes::Ok as StatusOk,
};

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};

// TODO: modify this when appropriate
const MAX_IMAGE_SPACE: usize = 4096;

enum ImageStoredData {
    #[cfg(feature = "alloc")]
    Boxed(Box<[u8]>),
    #[cfg(not(feature = "alloc"))]
    Array([u8; MAX_IMAGE_SPACE]),
}

impl ImageStoredData {
    #[inline]
    fn as_ptr(&self) -> *const u8 {
        match self {
            #[cfg(feature = "alloc")]
            Self::Boxed(ref b) => {
                let slice: &[u8] = &*b;
                slice as *const _ as *const _
            }
            #[cfg(not(feature = "alloc"))]
            Self::Array(ref a) => a.as_ptr(),
        }
    }

    #[inline]
    fn as_mut_slice(&mut self) -> &mut [u8] {
        match self {
            #[cfg(feature = "alloc")]
            Self::Boxed(ref mut b) => &mut *b,
            #[cfg(not(feature = "alloc"))]
            Self::Array(ref mut a) => &mut a,
        }
    }

    #[cfg(feature = "alloc")]
    fn new(capacity: usize) -> Self {
        Self::Boxed([0u8].iter().cycle().take(capacity).copied().collect())
    }

    #[cfg(not(feature = "alloc"))]
    fn new(_capacity: usize) -> Self {
        Self::Array([0; MAX_IMAGE_SPACE])
    }
}

pub struct Win32Image {
    // handle to the real bitmap
    bitmap: NonNull<GpBitmap>,
    // handle to the internal data
    data: Option<ImageStoredData>,
}

impl Drop for Win32Image {
    #[inline]
    fn drop(&mut self) {
        unsafe { gdiplusflat::GdipDisposeImage(self.img_ptr().as_ptr()) };
    }
}

impl Win32Image {
    #[inline]
    pub fn new(img: &dyn GenericImage) -> crate::Result<Self> {
        // get a pointer to the data
        let (ptr, data) = match img.raw_bytes() {
            Some(ptr) => (ptr, None),
            None => {
                // allocate data
                let mut container = ImageStoredData::new(img.len_elements());
                img.write_raw_bytes(container.as_mut_slice())?;
                (container.as_ptr(), Some(container))
            }
        };

        // create the bitmap
        let mut bitmap: *mut GpBitmap = ptr::null_mut();
        let Size2D { width, height, .. } = img.size();
        if unsafe {
            gdiplusflat::GdipCreateBitmapFromScan0(
                width as _,
                height as _,
                (width * img.color_space().size()) as _,
                match img.color_space() {
                    ColorSpace::Argb => gdipluspixelformats::PixelFormat32bppARGB,
                    ColorSpace::Rgb => gdipluspixelformats::PixelFormat24bppRGB,
                    ColorSpace::Grayscale => {
                        return Err(crate::Error::Unimplemented("Grayscale images on Windows"))
                    }
                },
                ptr as *mut _,
                &mut bitmap,
            )
        } != StatusOk
        {
            Err(crate::win32error("GdipCreateBitmapFromScan0"))
        } else {
            match NonNull::new(bitmap) {
                Some(bitmap) => Ok(Self { bitmap, data }),
                None => Err(crate::win32error("GdipCreateBitmapFromScan0")),
            }
        }
    }

    #[inline]
    pub fn img_ptr(&self) -> NonNull<GpImage> {
        self.bitmap.cast()
    }
}

/// Storage map for Win32 images.
#[repr(transparent)]
pub struct Win32ImageStorage {
    images: RwLock<StorageMap<usize, Win32Image, 12>>,
}

impl Win32ImageStorage {
    #[inline]
    pub fn new() -> Self {
        Self {
            images: RwLock::new(StorageMap::new()),
        }
    }

    #[inline]
    pub fn register_image(
        &self,
        img: &dyn GenericImage,
    ) -> crate::Result<OwningRef<RwLockReadGuard<'_, StorageMap<usize, Win32Image, 12>>, Win32Image>>
    {
        let id = img.id();
        if self.images.read().contains_key(&id) {
            Ok(OwningRef::new(self.images.read()).map(move |i| i.get(&id).unwrap()))
        } else {
            let mut images = self.images.write();
            images.insert(id, Win32Image::new(img)?);
            mem::drop(images);
            Ok(OwningRef::new(self.images.read()).map(move |i| i.get(&id).unwrap()))
        }
    }
}
