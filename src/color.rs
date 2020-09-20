// MIT/Apache2 License

//! Utilities for creating and modifying colors. This exports the `Rgba` type, which
//! represents an RGBA color made up of four floats.

use atomic_float::AtomicF32;
use core::{fmt, sync::atomic::Ordering};
use num_traits::{AsPrimitive, Bounded};
use ordered_float::NotNan;

// container for floats, they are non-Nan and between 0.0f32 and 1.0f32
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(transparent)]
struct FloatClamp(NotNan<f32>);

impl FloatClamp {
    #[inline]
    const fn new(val: f32) -> crate::Result<Self> {
        if val.is_nan() || 0.0f32 > val || 1.0f32 < val {
            Err(crate::Error::InvalidColorElement(val))
        } else {
            // SAFETY: confirmed to be safe
            Ok(unsafe { Self::new_unchecked(val) })
        }
    }

    #[inline]
    const unsafe fn new_unchecked(val: f32) -> Self {
        Self(NotNan::unchecked_new(val))
    }

    #[inline]
    const fn inner(self) -> f32 {
        union ConstTransmuter {
            notnan: NotNan<f32>,
            real: f32,
        }

        let c = ConstTransmuter { notnan: self.0 };
        unsafe { c.real }
    }
}

impl fmt::Debug for FloatClamp {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner(), f)
    }
}

/// An RGBA color.
///
/// This structure is represented internally by four floats. Unless unsafe code
/// is used, this structure is verified at runtime to only contain non-NaN values
/// from 0.0 to 1.0 inclusive. These represent the percentage of influence their
/// respective element has over the color. For instance, `(0.0, 1.0, 0.0, 1.0)`
/// represents the color green.
///
/// # Examples
///
/// ```rust
/// # use gui_tools::color::Rgba;
/// // instantiate a new color that is red
/// let red = Rgba::new(1.0, 0.0, 0.0, 1.0).unwrap();
///
/// // let's make sure red isn't blue
/// let blue = Rgba::new(0.0, 0.0, 1.0, 1.0).unwrap();
/// assert!(red != blue);
///
/// // we can easily clamp these values to a certain range based on a value
/// // for instance, lets say we want to convert to u8's
/// let (r, g, b, a) = red.convert_elements::<u8>();
/// assert_eq!(r, std::u8::MAX);
/// assert_eq!(g, 0);
/// assert_eq!(b, 0);
/// assert_eq!(a, std::u8::MAX);
/// ```
#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Rgba(FloatClamp, FloatClamp, FloatClamp, FloatClamp);

impl Default for Rgba {
    #[inline]
    fn default() -> Self {
        colors::BLACK
    }
}

impl Rgba {
    /// Create a new RGBA color, checking if the color elements are valid.
    ///
    /// A color element is considered valid if it is:
    ///
    /// * Not an Nan value.
    /// * Between 0.0 and 1.0 inclusive.
    ///
    /// This allows it to be represented as a percentage between 0% and 100%,
    /// indicating how present the respective element of color is.
    ///
    /// # Errors
    ///
    /// If any of the values are not valid color elements, this function returns
    /// `Error::InvalidColorElement`. Otherwise, it will return `Ok` alongside
    /// the color object.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gui_tools::color::Rgba;
    /// // a valid color
    /// let purple_ish = Rgba::new(0.67, 0.33, 1.0, 1.0);
    /// assert!(purple_ish.is_ok());
    ///
    /// // invalid: contains a Nan value
    /// // if any colors are Nan the constructor returns an error
    /// let invalid = Rgba::new(0.67, 0.33, std::f32::NAN, 1.0);
    /// assert!(invalid.is_err());
    ///
    /// // invalid: contains values not between 0.0 or 1.0
    /// // same as above: any invalid value causes the constructor to error out
    /// let invalid1 = Rgba::new(0.67, -0.33, 1.0, 1.0);
    /// let invalid2 = Rgba::new(1.2, 0.33, 1.0, 1.0);
    /// assert!(invalid1.is_err());
    /// assert!(invalid2.is_err());
    /// ```
    #[inline]
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> crate::Result<Self> {
        Ok(Self(
            FloatClamp::new(r)?,
            FloatClamp::new(g)?,
            FloatClamp::new(b)?,
            FloatClamp::new(a)?,
        ))
    }

    /// Create a new RGBA color without checking for validity.
    ///
    /// This function allows the user to create a color without
    /// actually checking if the values are valid color elements.
    /// The main advantage of this function is that it can be run in
    /// a constant context, allowing it to be used to create colors at
    /// compile time where the programmer knows that the color is valid.
    ///
    /// # Safety
    ///
    /// It is not recommended to use this function for user input or for
    /// instances where it is uncertain if the color values actually represent
    /// valid color elements.
    #[inline]
    pub const unsafe fn new_unchecked(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(
            FloatClamp::new_unchecked(r),
            FloatClamp::new_unchecked(g),
            FloatClamp::new_unchecked(b),
            FloatClamp::new_unchecked(a),
        )
    }

    /// Get the red component of this color.
    #[inline]
    pub const fn r(&self) -> f32 {
        self.0.inner()
    }

    /// Set the red component of this color, checking the value for validity.
    ///
    /// # Errors
    ///
    /// If the color value is not valid (i.e. it is NaN or not between 0.0 and 1.0 inclusive),
    /// this function will return `Error::InvalidColorElement`. Otherwise, it will return `Ok(())`.
    #[inline]
    pub fn set_r(&mut self, val: f32) -> crate::Result<()> {
        self.0 = FloatClamp::new(val)?;
        Ok(())
    }

    /// Set the red component of this color without checking for validity.
    #[inline]
    pub unsafe fn set_r_unchecked(&mut self, val: f32) {
        self.0 = FloatClamp::new_unchecked(val);
    }

    /// Get the green component of this color.
    #[inline]
    pub const fn g(&self) -> f32 {
        self.1.inner()
    }

    /// Set the green component of this color, checking the value for validity.
    ///
    /// # Errors
    ///
    /// If the color value is not valid (i.e. it is NaN or not between 0.0 and 1.0 inclusive),
    /// this function will return `Error::InvalidColorElement`. Otherwise, it will return `Ok(())`.
    #[inline]
    pub fn set_g(&mut self, val: f32) -> crate::Result<()> {
        self.1 = FloatClamp::new(val)?;
        Ok(())
    }

    /// Set the green component of this color without checking for validity.
    #[inline]
    pub unsafe fn set_g_unchecked(&mut self, val: f32) {
        self.1 = FloatClamp::new_unchecked(val);
    }

    /// Get the blue component of this color.
    #[inline]
    pub const fn b(&self) -> f32 {
        self.2.inner()
    }

    /// Set the blue component of this color, checking the value for validity.
    ///
    /// # Errors
    ///
    /// If the color value is not valid (i.e. it is NaN or not between 0.0 and 1.0 inclusive),
    /// this function will return `Error::InvalidColorElement`. Otherwise, it will return `Ok(())`.
    #[inline]
    pub fn set_b(&mut self, val: f32) -> crate::Result<()> {
        self.2 = FloatClamp::new(val)?;
        Ok(())
    }

    /// Set the blue component of this color without checking for validity.
    #[inline]
    pub unsafe fn set_b_unchecked(&mut self, val: f32) {
        self.2 = FloatClamp::new_unchecked(val);
    }

    /// Get the alpha component of this color.
    #[inline]
    pub const fn a(&self) -> f32 {
        self.3.inner()
    }

    /// Set the alpha component of this color, checking the value for validity.
    ///
    /// # Errors
    ///
    /// If the color value is not valid (i.e. it is NaN or not between 0.0 and 1.0 inclusive),
    /// this function will return `Error::InvalidColorElement`. Otherwise, it will return `Ok(())`.
    #[inline]
    pub fn set_a(&mut self, val: f32) -> crate::Result<()> {
        self.3 = FloatClamp::new(val)?;
        Ok(())
    }

    /// Set the alpha component of this color without checking for validity.
    #[inline]
    pub unsafe fn set_a_unchecked(&mut self, val: f32) {
        self.3 = FloatClamp::new_unchecked(val);
    }

    /// Get the elements of this color. This is equivalent to calling the `r()`, `g()`, `b()`,
    /// and `a()` elements, and exists primarily for convenience.
    #[inline]
    pub fn elements(&self) -> (f32, f32, f32, f32) {
        (self.r(), self.g(), self.b(), self.a())
    }

    /// Convert this element to a certain color space.
    ///
    /// The elements stored within this color object represent a "scale" of color. This
    /// function converts it from scales to actual values. For instance, if the red value
    /// is `0.5f32` and you convert the element to `u8`, the resulting red value will
    /// be `127`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use gui_tools::color::Rgba;
    /// let pink = Rgba::new(1.0, 0.5, 1.0, 1.0).unwrap();
    ///
    /// // let's convert the color pink to 16-bit true color
    /// let (r, g, b, a) = pink.convert_elements::<u16>();
    ///
    /// assert_eq!(r, std::u16::MAX);
    /// assert_eq!(g, std::u16::MAX / 2);
    /// assert_eq!(b, std::u16::MAX);
    /// assert_eq!(a, std::u16::MAX);
    /// ```
    #[inline]
    pub fn convert_elements<T>(&self) -> (T, T, T, T)
    where
        f32: AsPrimitive<T>,
        T: AsPrimitive<f32> + Bounded,
    {
        macro_rules! cvt {
            ($val: expr, $target: ty) => {{
                let max: f32 = <$target>::max_value().as_();
                let res = max * $val;
                let res_target: $target = res.as_();
                res_target
            }};
        }

        (
            cvt!(self.r(), T),
            cvt!(self.g(), T),
            cvt!(self.b(), T),
            cvt!(self.a(), T),
        )
    }
}

/// An atomic container for a color.
pub struct AtomicRgba(AtomicF32, AtomicF32, AtomicF32, AtomicF32);

impl AtomicRgba {
    #[inline]
    pub fn new(rgba: Rgba) -> Self {
        Self(
            AtomicF32::new(rgba.r()),
            AtomicF32::new(rgba.g()),
            AtomicF32::new(rgba.b()),
            AtomicF32::new(rgba.a()),
        )
    }

    #[inline]
    pub fn load(&self, ordering: Ordering) -> Rgba {
        unsafe {
            Rgba::new_unchecked(
                self.0.load(ordering),
                self.1.load(ordering),
                self.2.load(ordering),
                self.3.load(ordering),
            )
        }
    }

    #[inline]
    pub fn store(&self, rgba: Rgba, ordering: Ordering) {
        self.0.store(rgba.r(), ordering);
        self.1.store(rgba.g(), ordering);
        self.2.store(rgba.b(), ordering);
        self.3.store(rgba.a(), ordering);
    }
}

/// Pre-defined colors. This is used to provide a brief selection of basic colors. All of
/// these colors are defined as constants, so they can be used in constant contexts.
pub mod colors {
    use super::Rgba;

    pub const TRANSPARENT: Rgba = unsafe { Rgba::new_unchecked(0.0, 0.0, 0.0, 0.0) };
    pub const BLACK: Rgba = unsafe { Rgba::new_unchecked(0.0, 0.0, 0.0, 1.0) };
    pub const WHITE: Rgba = unsafe { Rgba::new_unchecked(1.0, 1.0, 1.0, 1.0) };
    pub const RED: Rgba = unsafe { Rgba::new_unchecked(1.0, 0.0, 0.0, 1.0) };
    pub const GREEN: Rgba = unsafe { Rgba::new_unchecked(0.0, 1.0, 0.0, 1.0) };
    pub const BLUE: Rgba = unsafe { Rgba::new_unchecked(0.0, 0.0, 1.0, 1.0) };
}
