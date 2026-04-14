//  PRINT.rs
//    by Lut99
//
//  Description:
//!   Contributes macros for the `print` module in `ast-toolkit2`.
//


/***** LIBRARY *****/
/// [`std::write!()`]-like macro that will write something using a `Style`.
///
/// # Example
/// ```ignore
/// use ast_toolkit2::print::{write_styled, Formatter, Style};
///
/// let mut buf = Vec::<u8>::new();
/// let mut fmt = Formatter::new(&mut buf);
/// let style = fmt.style().bold();
/// write_styled!(fmt, style, "Howdy, kids!");
/// ```
#[macro_export]
macro_rules! write_styled {
    ($f:expr, $style:expr, $($t:tt)*) => {
        ::std::write!($f, "{}", <$crate::Style>::apply_to($style, ::std::format_args!($($t)*)))
    };
}
