//  PRINT.rs
//    by Lut99
//
//  Description:
//!   Contributes macros for the `diag` module in `ast-toolkit2`.
//


/***** LIBRARY *****/
/// [`std::write!()`]-like macro that will write something using the current theme's highlighted
/// style.
#[macro_export]
macro_rules! write_highlighted {
    ($f:expr, $($t:tt)*) => {
        ::std::write!($f, ::ast_toolkit2::diag::layout4::replace::Formatter::apply_highlighted_style(::std::format_args!($($t)*)))
    };
}
