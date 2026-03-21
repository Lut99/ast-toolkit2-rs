//  ITERS.rs
//    by Lut99
//
//  Description:
//!   Defines some iterators for the [`Punctuated`](crate::Punctuated)-list.
//

use std::mem::MaybeUninit;


/***** HELPER MACROS *****/
/// Implements one of the three pair iterators.
macro_rules! iter_pair_impl {
    {
        $(#[$($attrs:meta)*])*
        name = $name:ident,
        $(lifetime = $lt:lifetime,)?
        iterator = $iter:ty,
        output = $output:ty,
        assume_init = $assume_init:ident,
    } => {
        $(#[$($attrs)*])*
        pub struct $name<$($lt,)? V, P> {
            /// The iterator yielding elements from the data list
            pub(crate) iter: std::iter::Enumerate<$iter>,
            /// The total number of elements to yield. Use to recognize the last one.
            pub(crate) len: usize,
            /// Whether the last element in the list has a punctuation.
            pub(crate) has_trailing: bool,
        }
        impl<$($lt,)? V, P> DoubleEndedIterator for $name<$($lt,)? V, P> {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                // Get the next element
                let (i, (v, p)): (usize, _) = self.iter.next_back()?;

                // Decide if there is a punctuation
                // SAFETY: `self.len - 1` will never panic, because the iterator above won't yield if the
                // list is empty
                if i < self.len - 1 || self.has_trailing {
                    // SAFETY: We can assume `p` is initialized, because the if-statement above asserts
                    // that our guarantees are used properly: it is either not the last element of the list
                    // (which guarantees it exists) or we remember there is a trailing one
                    Some((v, Some(unsafe { p.$assume_init() })))
                } else {
                    // SAFETY: We won't have to deallocate `p` because we know it does not exist
                    Some((v, None))
                }
            }
        }
        impl<$($lt,)? V, P> ExactSizeIterator for $name<$($lt,)? V, P> {
            #[inline]
            fn len(&self) -> usize { <std::iter::Enumerate<$iter> as ExactSizeIterator>::len(&self.iter) }
        }
        impl<$($lt,)? V, P> Iterator for $name<$($lt,)? V, P> {
            type Item = $output;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                // Get the next element
                let (i, (v, p)): (usize, _) = self.iter.next()?;

                // Decide if there is a punctuation
                // SAFETY: `self.len - 1` will never panic, because the iterator above won't yield if the
                // list is empty
                if i < self.len - 1 || self.has_trailing {
                    // SAFETY: We can assume `p` is initialized, because the if-statement above asserts
                    // that our guarantees are used properly: it is either not the last element of the list
                    // (which guarantees it exists) or we remember there is a trailing one
                    Some((v, Some(unsafe { p.$assume_init() })))
                } else {
                    // SAFETY: We won't have to deallocate `p` because we know it does not exist
                    Some((v, None))
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
        }
    };
}

/// Implements one of the three value iterators.
macro_rules! iter_value_impl {
    {
        $(#[$($attrs:meta)*])*
        name = $name:ident,
        $(lifetime = $lt:lifetime,)?
        iterator = $iter:ty,
        output = $output:ty,
    } => {
        $(#[$($attrs)*])*
        pub struct $name<$($lt,)? V, P> {
            /// The iterator yielding elements from the data list
            pub(crate) iter: $iter,
        }
        impl<$($lt,)? V, P> DoubleEndedIterator for $name<$($lt,)? V, P> {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> { self.iter.next_back().map(|(v, _)| v) }
        }
        impl<$($lt,)? V, P> ExactSizeIterator for $name<$($lt,)? V, P> {
            #[inline]
            fn len(&self) -> usize { <$iter as ExactSizeIterator>::len(&self.iter) }
        }
        impl<$($lt,)? V, P> Iterator for $name<$($lt,)? V, P> {
            type Item = $output;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> { self.iter.next().map(|(v, _)| v) }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
        }
    };
}

/// Implements one of the three punctuation iterators.
macro_rules! iter_punct_impl {
    {
        $(#[$($attrs:meta)*])*
        name = $name:ident,
        $(lifetime = $lt:lifetime,)?
        iterator = $iter:ty,
        output = $output:ty,
        assume_init = $assume_init:ident,
    } => {
        $(#[$($attrs)*])*
        pub struct $name<$($lt,)? V, P> {
            /// The iterator yielding elements from the data list
            pub(crate) iter: std::iter::Enumerate<$iter>,
            /// The total number of elements to yield. Use to recognize the last one.
            pub(crate) len: usize,
            /// Whether the last element in the list has a punctuation.
            pub(crate) has_trailing: bool,
        }
        impl<$($lt,)? V, P> DoubleEndedIterator for $name<$($lt,)? V, P> {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                // Get the next element first, if any
                let (i, (_, p)): (usize, _) = self.iter.next_back()?;

                // Decide whether we can assume the `p` is initialized or not.
                // SAFETY: We can do `self.len - 1` because, if `self.len` is 0, the iterator above
                // never yields.
                if i < self.len - 1 || self.has_trailing {
                    // SAFETY: We can unwrap here because the if-statement ensures we fall within
                    // our guarantees: it is either always initialized because it isn't the last
                    // element, OR we remember there is a last element.
                    Some(unsafe { p.$assume_init() })
                } else {
                    // SAFETY: No need to deallocate, as this is only reached when we find the last
                    // element without trailing punctuation (and hence, it doesn't exist).
                    None
                }
            }
        }
        impl<$($lt,)? V, P> ExactSizeIterator for $name<$($lt,)? V, P> {
            #[inline]
            fn len(&self) -> usize { <std::iter::Enumerate<$iter> as ExactSizeIterator>::len(&self.iter) }
        }
        impl<$($lt,)? V, P> Iterator for $name<$($lt,)? V, P> {
            type Item = $output;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                // Get the next element first, if any
                let (i, (_, p)): (usize, _) = self.iter.next()?;

                // Decide whether we can assume the `p` is initialized or not.
                // SAFETY: We can do `self.len - 1` because, if `self.len` is 0, the iterator above
                // never yields.
                if i < self.len - 1 || self.has_trailing {
                    // SAFETY: We can unwrap here because the if-statement ensures we fall within
                    // our guarantees: it is either always initialized because it isn't the last
                    // element, OR we remember there is a last element.
                    Some(unsafe { p.$assume_init() })
                } else {
                    // SAFETY: No need to deallocate, as this is only reached when we find the last
                    // element without trailing punctuation (and hence, it doesn't exist).
                    None
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
        }
    };
}





/***** LIBRARY *****/
iter_pair_impl! {
    /// Defines an iterator that yields elements from a [`Punctuated`](crate::Punctuated) by
    /// ownership.
    ///
    /// It is returned by [`Punctuated::into_iter()`](crate::Punctuated::into_iter()).
    name = IntoIter,
    iterator = std::vec::IntoIter<(V, MaybeUninit<P>)>,
    output = (V, Option<P>),
    assume_init = assume_init,
}
iter_pair_impl! {
    /// Defines an iterator that yields elements from a [`Punctuated`](crate::Punctuated) by
    /// reference.
    ///
    /// It is returned by [`Punctuated::iter()`](crate::Punctuated::iter()).
    name = Iter,
    lifetime = 'a,
    iterator = std::slice::Iter<'a, (V, MaybeUninit<P>)>,
    output = (&'a V, Option<&'a P>),
    assume_init = assume_init_ref,
}
iter_pair_impl! {
    /// Defines an iterator that yields elements from a [`Punctuated`](crate::Punctuated) mutably.
    ///
    /// It is returned by [`Punctuated::iter_mut()`](crate::Punctuated::iter_mut()).
    name = IterMut,
    lifetime = 'a,
    iterator = std::slice::IterMut<'a, (V, MaybeUninit<P>)>,
    output = (&'a mut V, Option<&'a mut P>),
    assume_init = assume_init_mut,
}



iter_value_impl! {
    /// Defines an iterator that yields values from a [`Punctuated`](crate::Punctuated) by
    /// ownership.
    ///
    /// It is returned by [`Punctuated::into_values()`](crate::Punctuated::into_values()).
    name = IntoValues,
    iterator = std::vec::IntoIter<(V, MaybeUninit<P>)>,
    output = V,
}

iter_value_impl! {
    /// Defines an iterator that yields values from a [`Punctuated`](crate::Punctuated) by
    /// reference.
    ///
    /// It is returned by [`Punctuated::values()`](crate::Punctuated::values()).
    name = Values,
    lifetime = 'a,
    iterator = std::slice::Iter<'a, (V, MaybeUninit<P>)>,
    output = &'a V,
}

iter_value_impl! {
    /// Defines an iterator that yields values from a [`Punctuated`](crate::Punctuated) mutably.
    ///
    /// It is returned by [`Punctuated::values_mut()`](crate::Punctuated::values_mut()).
    name = ValuesMut,
    lifetime = 'a,
    iterator = std::slice::IterMut<'a, (V, MaybeUninit<P>)>,
    output = &'a mut V,
}



iter_punct_impl! {
    /// Defines an iterator that yields punctuation from a [`Punctuated`](crate::Punctuated) by
    /// ownership.
    ///
    /// It is returned by [`Punctuated::into_puncts()`](crate::Punctuated::into_puncts()).
    name = IntoPuncts,
    iterator = std::vec::IntoIter<(V, MaybeUninit<P>)>,
    output = P,
    assume_init = assume_init,
}

iter_punct_impl! {
    /// Defines an iterator that yields punctuation from a [`Punctuated`](crate::Punctuated) by
    /// reference.
    ///
    /// It is returned by [`Punctuated::puncts()`](crate::Punctuated::puncts()).
    name = Puncts,
    lifetime = 'a,
    iterator = std::slice::Iter<'a, (V, MaybeUninit<P>)>,
    output = &'a P,
    assume_init = assume_init_ref,
}

iter_punct_impl! {
    /// Defines an iterator that yields punctuation from a [`Punctuated`](crate::Punctuated)
    /// mutably.
    ///
    /// It is returned by [`Punctuated::puncts_mut()`](crate::Punctuated::puncts_mut()).
    name = PunctsMut,
    lifetime = 'a,
    iterator = std::slice::IterMut<'a, (V, MaybeUninit<P>)>,
    output = &'a mut P,
    assume_init = assume_init_mut,
}



/// Defines an iterator that yields removed elements when calling
/// [`Punctuated::drain()`](crate::Punctuated::drain()).
pub type Drain<V, P> = IntoIter<V, P>;
