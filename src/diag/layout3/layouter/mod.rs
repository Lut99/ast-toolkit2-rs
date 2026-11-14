//  LAYOUTER.rs
//    by Lut99
//
//  Description:
//!   Defines the interface to layouting algorithms of source text.
//

// Nested modules
#[cfg(feature = "sources")]
mod hex;

use std::cell::{Ref, RefMut};
use std::fmt::{Formatter, Result as FResult};
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

#[cfg(feature = "sources")]
pub use hex::HexLayouter;

// Imports & Exports
use crate::loc::Range;


/***** HELPER MACROS *****/
macro_rules! layouter_ptr_impl {
    (__impl) => {
        #[inline(always)]
        fn max_width(&self) -> usize { <T as Layouter<E>>::max_width(self) }

        #[inline(always)]
        fn range_to_2d(&self, source_id: u64, ranges: impl IntoIterator<Item = Range>) -> Vec<Range2D> {
            <T as Layouter<E>>::range_to_2d(self, source_id, ranges)
        }

        #[inline(always)]
        fn render_loc(&self, f: &mut Formatter<'_>, source_id: u64, pos: u64) -> FResult { <T as Layouter<E>>::render_loc(self, f, source_id, pos) }

        #[inline(always)]
        fn render_line(&self, f: &mut Formatter<'_>, source_id: u64, line: u64) -> FResult {
            <T as Layouter<E>>::render_line(self, f, source_id, line)
        }
    };

    ('a, $ty:ty) => {
        impl<'a, T: Layouter<E>, E> Layouter<E> for $ty {
            layouter_ptr_impl!(__impl);
        }
    };
    ($ty:ty) => {
        impl<T: Layouter<E>, E> Layouter<E> for $ty {
            layouter_ptr_impl!(__impl);
        }
    };
}





/***** AUXILLARY *****/
/// A 2 dimensional version of a [`Range`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Range2D {
    /// The lines that are touched by this range, together with where the line is touched.
    pub lines: Vec<LineRange>,
}

/// A range only spanning a single line.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LineRange {
    /// The line this range lives on.
    pub line: u64,
    /// The start pos of the range, relative to the start of the line.
    pub pos:  usize,
    /// The length of the range, bounded by the end of the line.
    pub len:  usize,
}





/***** INTERFACES *****/
/// Represents a way of rendering source text.
pub trait Layouter<E> {
    /// Returns the maximum width of every line in the layouter.
    fn max_width(&self) -> usize;

    /// Turns a set of 1-dimensional [`Range`]s into 2-dimensional [`LineRange`]s.
    ///
    /// The layouter should compute where each given [`Range`] lies in 2D-space on the source text;
    /// then, it should split each range into one [`LineRange`] for every line it touches.
    ///
    /// The fact that multiple [`Range`]s in one go are given, is to make optimizations on the end
    /// of the layouter possible. The function should work as if each is given individually.
    ///
    /// # Arguments
    /// - `source_id`: The ID of the source for which we're looking. It is expected that the
    ///   [`Layouter`] can resolve this to a physical source by itself.
    /// - `ranges`: The list of [`Range`]s to process.
    ///
    /// # Returns
    /// A collection of [`Range2D`], one for every [`Range`], describing how it is split over
    /// lines.
    fn range_to_2d(&self, source_id: u64, ranges: impl IntoIterator<Item = Range>) -> Vec<Range2D>;

    /// Renders the source information of the given loc.
    ///
    /// # Arguments
    /// - `f`: A [`Formatter`] to render to.
    /// - `source_id`: The source ID of the source that the loc refers to.
    /// - `pos`: The position of the error in the source text.
    ///
    /// # Errors
    /// This function should only error if it failed to write to `f`.
    fn render_loc(&self, f: &mut Formatter<'_>, source_id: u64, pos: u64) -> FResult;

    /// Renders a line from the source text.
    ///
    /// # Arguments
    /// - `f`: A [`Formatter`] to render to.
    /// - `source_id`: The source ID of the source that the loc refers to.
    /// - `line`: The zero-indexed line number to render.
    ///
    /// # Errors
    /// This function should only error if it failed to write to `f`.
    fn render_line(&self, f: &mut Formatter<'_>, source_id: u64, line: u64) -> FResult;
}

// Pointer-like impls
layouter_ptr_impl!('a, &'a T);
layouter_ptr_impl!('a, &'a mut T);
layouter_ptr_impl!(Box<T>);
layouter_ptr_impl!(Rc<T>);
layouter_ptr_impl!(Arc<T>);
layouter_ptr_impl!('a, Ref<'a, T>);
layouter_ptr_impl!('a, RefMut<'a, T>);
layouter_ptr_impl!('a, MutexGuard<'a, T>);
layouter_ptr_impl!('a, RwLockReadGuard<'a, T>);
layouter_ptr_impl!('a, RwLockWriteGuard<'a, T>);
