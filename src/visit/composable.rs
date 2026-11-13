//  COMPOSABLE.rs
//    by Lut99
//
//  Description:
//!   A trait that defines a generic interface to types resulted by traversals.
//

use std::collections::{HashMap, HashSet};
use std::hash::Hash;


/***** HELPER MACROS *****/
macro_rules! initializable_primitive_impl {
    ($ty:ty, $value:expr) => {
        impl Initializable for $ty {
            #[inline(always)]
            fn init() -> Self { $value }
        }
    };
}

macro_rules! composable_primitive_impl {
    ($ty:ty) => {
        impl Composable for $ty {
            #[inline(always)]
            fn compose_with(&mut self, other: Self) { *self += other }
        }
    };
}





/***** LIBRARY *****/
/// Defines a type can start composition of results to traversals.
///
/// It is, in fact, exactly equivalent to [`Default`], but because it is not implemented for all
/// [`std`] types (most relevantly, [`Result`]), and because of orphan rules, we cannot tie
/// [`Initializable`] to the presence of [`Default`] in any way (try it yourself, I dare you...).
pub trait Initializable {
    /// Starts a value that can be used for composition.
    ///
    /// This is, e.g., 0, [`Vec::new()`] or [`None`].
    fn init() -> Self;
}

// Pointer-like impls
impl<T: Initializable> Initializable for Box<T> {
    #[inline(always)]
    fn init() -> Self { Box::new(T::init()) }
}

// Std impls
initializable_primitive_impl!(u8, 0);
initializable_primitive_impl!(u16, 0);
initializable_primitive_impl!(u32, 0);
initializable_primitive_impl!(u64, 0);
initializable_primitive_impl!(u128, 0);
initializable_primitive_impl!(usize, 0);
initializable_primitive_impl!(i8, 0);
initializable_primitive_impl!(i16, 0);
initializable_primitive_impl!(i32, 0);
initializable_primitive_impl!(i64, 0);
initializable_primitive_impl!(i128, 0);
initializable_primitive_impl!(isize, 0);
initializable_primitive_impl!(f32, 0.0);
initializable_primitive_impl!(f64, 0.0);
initializable_primitive_impl!((), ());

impl<T> Initializable for Option<T> {
    #[inline(always)]
    fn init() -> Self { None }
}
impl<T: Initializable, E> Initializable for Result<T, E> {
    #[inline(always)]
    fn init() -> Self { Ok(T::init()) }
}

impl<T> Initializable for Vec<T> {
    #[inline(always)]
    fn init() -> Self { Vec::new() }
}
impl<T> Initializable for HashSet<T> {
    #[inline(always)]
    fn init() -> Self { HashSet::new() }
}
impl<K, V> Initializable for HashMap<K, V> {
    #[inline(always)]
    fn init() -> Self { HashMap::new() }
}



/// Defines generic composition of results to traversals.
///
/// See [`Composable::compose_with()`] for more information.
///
/// Note that, while this trait requires [`Initializable`], this is automatically implemented for
/// all types that implement [`Default`]. Hence, implementing that is sufficient too.
pub trait Composable: Initializable {
    /// Composes `other` into `self`.
    ///
    /// This function is used to generically compose two answers of derivations.
    ///
    /// For example, it can be used to sum counts, accumulate [`Some`]s or concatenate [`String`]s.
    ///
    /// The type must also be [`Default`] in order to initialize it.
    fn compose_with(&mut self, other: Self);
}

// Pointer-like impls
impl<T: Composable> Composable for Box<T> {
    #[inline(always)]
    fn compose_with(&mut self, other: Self) { T::compose_with(self, *other); }
}

// Std impls
composable_primitive_impl!(u8);
composable_primitive_impl!(u16);
composable_primitive_impl!(u32);
composable_primitive_impl!(u64);
composable_primitive_impl!(u128);
composable_primitive_impl!(usize);
composable_primitive_impl!(i8);
composable_primitive_impl!(i16);
composable_primitive_impl!(i32);
composable_primitive_impl!(i64);
composable_primitive_impl!(i128);
composable_primitive_impl!(isize);
composable_primitive_impl!(f32);
composable_primitive_impl!(f64);
impl Composable for () {
    #[inline(always)]
    fn compose_with(&mut self, _other: Self) {}
}

impl<T: Composable> Composable for Option<T> {
    #[inline]
    fn compose_with(&mut self, other: Self) {
        match (self, other) {
            (Some(lhs), Some(rhs)) => lhs.compose_with(rhs),
            (Some(_) | None, None) => {},
            (this @ None, Some(rhs)) => *this = Some(rhs),
        }
    }
}
impl<T: Composable, E> Composable for Result<T, E> {
    /// Composes `T` unless an error occurs. Then, the first (=leftmost) error is fasttracked.
    #[inline]
    fn compose_with(&mut self, other: Self) {
        match (self, other) {
            (Ok(lhs), Ok(rhs)) => lhs.compose_with(rhs),
            (this @ Ok(_), Err(err)) => *this = Err(err),
            (Err(_), _) => {},
        }
    }
}

impl<T> Composable for Vec<T> {
    #[inline(always)]
    fn compose_with(&mut self, other: Self) { self.extend(other) }
}
impl<T: Eq + Hash> Composable for HashSet<T> {
    #[inline(always)]
    fn compose_with(&mut self, other: Self) { self.extend(other) }
}
impl<K: Eq + Hash, V> Composable for HashMap<K, V> {
    /// Composes two HashMaps by keys.
    ///
    /// The values are NOT composed further if the key exists, but instead simply overwritten.
    #[inline(always)]
    fn compose_with(&mut self, other: Self) { self.extend(other) }
}
