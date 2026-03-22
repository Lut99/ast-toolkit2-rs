//  PUNCT.rs
//    by Lut99
//
//  Description:
//!   Defines convenient macros for building [`Punctuated`](super::Punctuated) lists ergonomically.
//


/***** LIBRARY *****/
/// Macro for initializing a [`Punctuated`](super::Punctuated) ergonomically.
///
/// You can use this macro by giving a comma-separated list of values interspersed by punctuations,
/// optionally giving a trailing one, much like you would with [`vec![]`](vec!).
///
/// It returns a new [`Punctuated`](super::Punctuated).
///
/// # Examples
/// ```ignore
/// use ast_toolkit2::punct::{Punctuated, punct};
///
/// assert_eq!(format!("{:?}", punct!["Hello", ',', "world"]), "[\"Hello\", ',', \"world\"]");
/// ```
#[macro_export]
macro_rules! punct {
    /* Counting */
    // Empty base cases
    (__count) => { 0 };
    (__count $v:expr $(, $vn:expr)*) => { 1 + $crate::punct!(__count $($vn),*) };

    /* Internal recursion */
    // Empty base cases
    (__odd($var:ident)) => {};
    (__even($var:ident)) => {};
    // First / third / ... push; values
    (__odd($var:ident) $v:expr $(, $vn:expr)*) => {
        $var.push_value($v);
        $crate::punct!{__even($var) $($vn),*};
    };
    // Second / fourth / ... push; punctuation
    (__even($var:ident) $v:expr $(, $vn:expr)*) => {
        $var.push_punct($v);
        $crate::punct!{__odd($var) $($vn),*};
    };

    /* Outward syntax */
    [crate: $($v:expr),* $(,)?] => {{
        #[allow(unused_mut)]
        let mut __res = crate::punct::Punctuated::with_capacity((1 + $crate::punct!(__count $($v),*)) / 2);
        $crate::punct!{__odd(__res) $($v),*};
        __res
    }};
    [$($v:expr),* $(,)?] => {{
        #[allow(unused_mut)]
        let mut __res = ::ast_toolkit2::punct::Punctuated::with_capacity((1 + $crate::punct!(__count $($v),*)) / 2);
        $crate::punct!{__odd(__res) $($v),*};
        __res
    }};
}
