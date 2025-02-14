pub use palette_derive::FromColorUnclamped;

#[cfg(feature = "std")]
use crate::cast::{self, ArrayCast};

/// A trait for unchecked conversion of one color from another.
///
/// See [`FromColor`](crate::convert::FromColor) for a lossy version of this trait.
/// See [`TryFromColor`](crate::convert::TryFromColor) for a trait that gives an error when the result
/// is out of bounds.
///
/// See the [`convert`](crate::convert) module for how to implement `FromColorUnclamped` for
/// custom colors.
pub trait FromColorUnclamped<T>: Sized {
    /// Convert from T. The resulting color might be invalid in its color space.
    ///
    /// ```
    /// use palette::convert::FromColorUnclamped;
    /// use palette::{IsWithinBounds, Lch, Srgb};
    ///
    /// let rgb = Srgb::from_color_unclamped(Lch::new(50.0, 100.0, -175.0));
    /// assert!(!rgb.is_within_bounds());
    /// ```
    #[must_use]
    fn from_color_unclamped(val: T) -> Self;
}

#[cfg(feature = "std")]
impl<T, U> FromColorUnclamped<Vec<T>> for Vec<U>
where
    T: ArrayCast,
    U: ArrayCast<Array = T::Array> + FromColorUnclamped<T>,
{
    /// Convert all colors in place, without reallocating.
    ///
    /// ```
    /// use palette::{convert::FromColorUnclamped, SaturateAssign, Srgb, Lch};
    ///
    /// let srgb = vec![Srgb::new(0.8f32, 1.0, 0.2), Srgb::new(0.9, 0.1, 0.3)];
    /// let mut lch = Vec::<Lch>::from_color_unclamped(srgb);
    ///
    /// lch.saturate_assign(0.1);
    ///
    /// let srgb = Vec::<Srgb>::from_color_unclamped(lch);
    /// ```
    #[inline]
    fn from_color_unclamped(color: Vec<T>) -> Self {
        cast::map_vec_in_place(color, U::from_color_unclamped)
    }
}

#[cfg(feature = "std")]
impl<T, U> FromColorUnclamped<Box<[T]>> for Box<[U]>
where
    T: ArrayCast,
    U: ArrayCast<Array = T::Array> + FromColorUnclamped<T>,
{
    /// Convert all colors in place, without reallocating.
    ///
    /// ```
    /// use palette::{convert::FromColorUnclamped, SaturateAssign, Srgb, Lch};
    ///
    /// let srgb = vec![Srgb::new(0.8f32, 1.0, 0.2), Srgb::new(0.9, 0.1, 0.3)].into_boxed_slice();
    /// let mut lch = Box::<[Lch]>::from_color_unclamped(srgb);
    ///
    /// lch.saturate_assign(0.1);
    ///
    /// let srgb = Box::<[Srgb]>::from_color_unclamped(lch);
    /// ```
    #[inline]
    fn from_color_unclamped(color: Box<[T]>) -> Self {
        cast::map_slice_box_in_place(color, U::from_color_unclamped)
    }
}

/// A trait for unchecked conversion of a color into another.
///
/// `U: IntoColorUnclamped<T>` is implemented for every type `T: FromColorUnclamped<U>`.
///
/// See [`FromColorUnclamped`](crate::convert::FromColorUnclamped) for more details.
pub trait IntoColorUnclamped<T>: Sized {
    /// Convert into T. The resulting color might be invalid in its color space
    ///
    /// ```
    /// use palette::convert::IntoColorUnclamped;
    /// use palette::{IsWithinBounds, Lch, Srgb};
    ///
    ///let rgb: Srgb = Lch::new(50.0, 100.0, -175.0).into_color_unclamped();
    ///assert!(!rgb.is_within_bounds());
    ///```
    #[must_use]
    fn into_color_unclamped(self) -> T;
}

impl<T, U> IntoColorUnclamped<U> for T
where
    U: FromColorUnclamped<T>,
{
    #[inline]
    fn into_color_unclamped(self) -> U {
        U::from_color_unclamped(self)
    }
}
