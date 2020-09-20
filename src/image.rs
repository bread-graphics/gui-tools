// MIT/Apache2 License

use crate::{color::Rgba, geometry::Pixel};
use core::{
    any::TypeId,
    hash::{Hash, Hasher},
    ops::Div,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};
use euclid::Size2D;
use num_traits::{AsPrimitive, Bounded};

#[cfg(feature = "alloc")]
use alloc::sync::Arc;

/// The color space for an image.
#[derive(Copy, Clone, Debug, Hash)]
pub enum ColorSpace {
    Argb,
    Rgb,
    Grayscale,
}

impl ColorSpace {
    #[inline]
    pub fn size(self) -> usize {
        match self {
            Self::Argb => 4,
            Self::Rgb => 3,
            Self::Grayscale => 1,
        }
    }
}

/// The data for a bitmap image.
///
/// This represents a pointer to a series of color elements that
/// represent a bitmap image. On allocation-supported targets, this can be
/// a smart pointer to a heap-allocated series of bytes. Otherwise, it can
/// only be a reference to a static series of values.
#[derive(Clone)]
pub enum ImageData<T: 'static> {
    /// References a series of bytes that represents an image.
    Reference(&'static [T]),
    /// A smart pointer to bytes representing a bitmap image.
    #[cfg(feature = "alloc")]
    OwnedPointer(Arc<[T]>),
}

impl<T: 'static> ImageData<T> {
    #[inline]
    pub fn as_ptr(&self) -> *const [T] {
        match self {
            Self::Reference(r) => (*r) as *const _,
            #[cfg(feature = "alloc")]
            Self::OwnedPointer(ref op) => Arc::as_ptr(op),
        }
    }
}

impl<T: Hash + 'static> Hash for ImageData<T> {
    #[inline]
    fn hash<H: Hasher>(&self, h: &mut H) {
        match self {
            Self::Reference(ref r) => Hash::hash(r, h),
            #[cfg(feature = "alloc")]
            Self::OwnedPointer(ref op) => Hash::hash(&*op, h),
        }
    }
}

// global image ID
static NEXT_IMAGE_ID: AtomicUsize = AtomicUsize::new(0);

/// A representation of a bitmap image.
#[derive(Clone)]
pub struct Image<T: 'static> {
    data: ImageData<T>,
    size: Size2D<usize, Pixel>,
    color_space: ColorSpace,
    id: usize,
}

/// A type-erased version of `Image` to allow it to be used for
/// dynamic dispatch.
pub trait GenericImage {
    /// Get the color space of this image.
    fn color_space(&self) -> ColorSpace;

    /// Get the size of the image component.
    fn img_cmp_sizeof(&self) -> usize;

    /// Get the length of this image in raw elements.
    fn len_elements(&self) -> usize;

    /// Get the length of this image in color elements.
    #[inline]
    fn len_colors(&self) -> usize {
        self.len_elements() / self.color_space().size()
    }

    /// Get the color element at a certain index.
    fn color_at_flat(&self, index: usize) -> Option<Rgba>;

    /// Get the size of this image.
    fn size(&self) -> Size2D<usize, Pixel>;

    /// Get the width of this image.
    #[inline]
    fn width(&self) -> usize {
        self.size().width
    }

    /// Get the height of this image.
    #[inline]
    fn height(&self) -> usize {
        self.size().height
    }

    /// Get the color element at a certain (X, Y) location.
    #[inline]
    fn color_at(&self, x: usize, y: usize) -> Option<Rgba> {
        self.color_at_flat((y * self.height()) + x)
    }

    /// Get a unique ID associated with this image.
    fn id(&self) -> usize;

    /// If possible, get a pointer to the raw bytes.
    fn raw_bytes(&self) -> Option<*const u8>;

    /// Write the bytes of an item to a mutable slice.
    fn write_raw_bytes(&self, bytes: &mut [u8]) -> crate::Result<()>;
}

impl<T> GenericImage for Image<T>
where
    T: AsPrimitive<f32> + Bounded + Clone + Div + 'static,
{
    #[inline]
    fn color_space(&self) -> ColorSpace {
        self.color_space
    }

    #[inline]
    fn img_cmp_sizeof(&self) -> usize {
        core::mem::size_of::<T>()
    }

    #[inline]
    fn len_elements(&self) -> usize {
        self.data().len()
    }

    #[inline]
    fn color_at_flat(&self, index: usize) -> Option<Rgba> {
        #[inline]
        fn element_at<T: AsPrimitive<f32> + Bounded + Div>(
            data: &[T],
            index: usize,
        ) -> Option<f32> {
            match data.get(index) {
                None => None,
                Some(item) => Some(item.as_() / T::max_value().as_()),
            }
        }

        match self.color_space {
            ColorSpace::Grayscale => {
                let element = element_at(self.data(), index)?;
                Some(unsafe { Rgba::new_unchecked(element, element, element, 1.0) })
            }
            ColorSpace::Rgb => Some(unsafe {
                Rgba::new_unchecked(
                    element_at(self.data(), index)?,
                    element_at(self.data(), index + 1)?,
                    element_at(self.data(), index + 2)?,
                    1.0,
                )
            }),
            ColorSpace::Argb => Some(unsafe {
                Rgba::new_unchecked(
                    element_at(self.data(), index + 1)?,
                    element_at(self.data(), index + 2)?,
                    element_at(self.data(), index + 3)?,
                    element_at(self.data(), index)?,
                )
            }),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn size(&self) -> Size2D<usize, Pixel> {
        self.size
    }

    #[inline]
    fn id(&self) -> usize {
        self.id
    }

    #[inline]
    fn raw_bytes(&self) -> Option<*const u8> {
        if TypeId::of::<T>() == TypeId::of::<u8>() {
            Some(self.as_ptr().cast())
        } else {
            None
        }
    }

    #[inline]
    fn write_raw_bytes(&self, bytes: &mut [u8]) -> crate::Result<()> {
        #[inline]
        fn clamp_element<T: AsPrimitive<f32> + Bounded + Div>(data: T) -> u8 {
            ((data.as_() / T::max_value().as_()) * (core::u8::MAX as f32)) as u8
        }

        if bytes.len() < self.len_elements() {
            Err(crate::Error::NoSpaceForImage)
        } else {
            self.data()
                .iter()
                .enumerate()
                .for_each(|(i, d)| bytes[i] = clamp_element(d.clone()));
            Ok(())
        }
    }
}

impl<T: 'static> Image<T> {
    #[inline]
    pub unsafe fn from_elements_unchecked(
        data: &'static [T],
        width: usize,
        height: usize,
        color_space: ColorSpace,
    ) -> Self {
        Self {
            data: ImageData::Reference(data),
            size: Size2D::new(width, height),
            color_space,
            id: NEXT_IMAGE_ID.fetch_add(1, Ordering::Acquire),
        }
    }

    #[inline]
    pub fn from_elements(
        data: &'static [T],
        width: usize,
        height: usize,
        color_space: ColorSpace,
    ) -> Option<Self> {
        if width * height != data.len() || data.len() % color_space.size() != 0 {
            None
        } else {
            Some(unsafe { Self::from_elements_unchecked(data, width, height, color_space) })
        }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    pub unsafe fn from_elements_arc_unchecked(
        data: Arc<[T]>,
        width: usize,
        height: usize,
        color_space: ColorSpace,
    ) -> Self {
        Self {
            data: ImageData::OwnedPointer(data),
            size: Size2D::new(width, height),
            color_space,
            id: NEXT_IMAGE_ID.fetch_add(1, Ordering::Acquire),
        }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    pub fn from_elements_arc<A: Into<Arc<[T]>>>(
        data: A,
        width: usize,
        height: usize,
        color_space: ColorSpace,
    ) -> Option<Self> {
        let arc: Arc<[T]> = data.into();
        if width * height != arc.len() || arc.len() % color_space.size() != 0 {
            None
        } else {
            Some(unsafe { Self::from_elements_arc_unchecked(arc, width, height, color_space) })
        }
    }

    #[inline]
    pub fn data(&self) -> &[T] {
        match self.data {
            ImageData::Reference(r) => r,
            #[cfg(feature = "alloc")]
            ImageData::OwnedPointer(ref op) => &*op,
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const [T] {
        self.data.as_ptr()
    }
}
