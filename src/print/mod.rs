//  PRINT.rs
//    by Lut99
//
//  Description:
//!   A module for pretty-printing ASTs in a convenient and predictable way.
//

// Modules
mod display;
mod fmt;
#[cfg(feature = "color")]
mod style;

// Imports
use std::cell::{Ref, RefMut};
use std::fmt::Result as FResult;
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

#[cfg(feature = "decl-macros")]
pub use ast_toolkit2_decl_macros::write_styled;
pub use display::Display;
pub use fmt::Formatter;

/// A module that collects all traits of the printing module in one importable location.
pub mod prelude {
    pub use super::{Coloring, PrettyPrintExt};
}


/***** HELPER MACROS *****/
/// Implements pointer-like blanket impls for [`PrettyPrint`].
macro_rules! pretty_print_ptr_impl {
    ('a, $ty:ty) => {
        impl<'a, T: PrettyPrint> PrettyPrint for $ty {
            #[inline]
            fn fmt(&self, f: &mut Formatter<'_>) -> FResult { <T as PrettyPrint>::fmt(self, f) }
        }
    };
    ($ty:ty) => {
        impl<T: PrettyPrint> PrettyPrint for $ty {
            #[inline]
            fn fmt(&self, f: &mut Formatter<'_>) -> FResult { <T as PrettyPrint>::fmt(self, f) }
        }
    };
}





/***** AUXILLARY *****/
/// Defines the choice of coloring to apply.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Coloring {
    /// Always use colors.
    Always,
    /// Never use colors.
    Never,
    /// Only use colors if stdout is a TTY.
    AutoStdout,
    /// Only use colors if stderr is a TTY.
    AutoStderr,
}





/***** LIBRARY *****/
/// A trait for pretty-printing a node in the AST.
///
/// It is very much like [`Debug`](std::fmt::Debug) or [`Display`](std::fmt::Display), except that
/// a custom and more elaborate [`Formatter`] is used that also has deep integration for
/// ANSI-colors.
///
/// Note, though, that you rarely use this trait as an end-user. See [`PrettyPrintExt`] instead.
pub trait PrettyPrint {
    /// Implements the pretty printing for a single node.
    ///
    /// # Arguments
    /// - `f`: The [`Formatter`] that you can write to using the normal [`write!()`] macros and all
    ///   that.
    ///
    /// # Errors
    /// This function should only error if you fail to write to the `f`ormatter.
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult;
}

// Pointer-like impls
pretty_print_ptr_impl!('a, &'a T);
pretty_print_ptr_impl!('a, &'a mut T);
pretty_print_ptr_impl!(Box<T>);
pretty_print_ptr_impl!(Rc<T>);
pretty_print_ptr_impl!(Arc<T>);
pretty_print_ptr_impl!('a, Ref<'a, T>);
pretty_print_ptr_impl!('a, RefMut<'a, T>);
pretty_print_ptr_impl!('a, MutexGuard<'a, T>);
pretty_print_ptr_impl!('a, RwLockReadGuard<'a, T>);
pretty_print_ptr_impl!('a, RwLockWriteGuard<'a, T>);



/// A trait for pretty-printing a node in the AST.
///
/// This trait is what you interact with as an end-user. To implement the trait for nodes, see
/// [`PrettyPrint`] instead.
pub trait PrettyPrintExt: PrettyPrint {
    /// Returns a pretty printer for this AST node that implements [`Display`].
    ///
    /// This option will never use coloring. If you want to, enable the `color`-feature and use
    /// [`PrettyPrintExt::display_color()`] instead.
    ///
    /// # Returns
    /// A [`Display`] struct that will render the nodes.
    fn display(&self) -> Display<'_, Self>;

    /// Returns a pretty printer for this AST node that implements [`Display`].
    ///
    /// This option will use coloring depending on your [`Coloring`] choice. If you haven't got the
    /// `color` feature enabled, or you don't want to use color, you can also use
    /// [`PrettyPrintExt::display()`].
    ///
    /// # Arguments
    /// - `coloring`: What kind of [`Coloring`] to apply.
    ///
    /// # Returns
    /// A [`Display`] struct that will render the nodes.
    #[cfg(feature = "color")]
    fn display_color(&self, coloring: Coloring) -> Display<'_, Self>;
}

// Blanket impl for all `PrettyPrint`ing things
impl<T: PrettyPrint> PrettyPrintExt for T {
    #[inline]
    fn display(&self) -> Display<'_, Self> {
        Display(
            self,
            #[cfg(feature = "color")]
            Coloring::Never,
        )
    }

    #[cfg(feature = "color")]
    #[inline]
    fn display_color(&self, coloring: Coloring) -> Display<'_, Self> { Display(self, coloring) }
}
