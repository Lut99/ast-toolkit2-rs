//  PUNCTUATED.rs
//    by Lut99
//
//  Description:
//!   Defines a data structure that stores two types alternating in a single
//!   sequence.
//

// Declare the modules
mod iters;
#[cfg(feature = "loc")]
mod loc;
#[cfg(feature = "macros")]
mod macros;
#[cfg(feature = "serde")]
mod serde;

// Imports
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter, Result as FResult};
use std::hash::{Hash, Hasher};
use std::mem::MaybeUninit;
use std::ops::{Bound, Index, IndexMut, RangeBounds};

// Bring some of it into this namespace
pub use iters::*;

// Define a module for putting all auxillary things in this module
pub mod prelude {
    pub use super::{PunctIndex, PushPunctResult, ValueIndex};
}


/***** HELPER FUNCTIONS *****/
/// Resolves any [`RangeBounds`] to a concrete start- and stop index.
///
/// # Arguments
/// - `range`: Some [`RangeBounds`] to resolve.
/// - `len`: The length of the thing that it is spanning.
///
/// # Returns
/// A pair of `(start, stop)` that encodes the concrete indices into the list that are being
/// spanned by the range.
///
/// Note that the resulting range _may_ be empty or even negative.
#[inline]
fn resolve_range(range: impl RangeBounds<usize>, len: usize) -> (usize, usize) {
    let start: usize = match range.start_bound().cloned() {
        Bound::Excluded(start) => start.saturating_add(1),
        Bound::Included(start) => start,
        Bound::Unbounded => 0,
    };
    let end: usize = match range.end_bound().cloned() {
        Bound::Excluded(end) => {
            if end <= len {
                end
            } else {
                len
            }
        },
        Bound::Included(end) => {
            if end < len {
                end.saturating_add(1)
            } else {
                len
            }
        },
        Bound::Unbounded => len,
    };
    (start, end)
}





/***** AUXILLLARY *****/
/// Represents a possible index that can be used to index the values explicitly in a
/// [`Punctuated`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ValueIndex(pub usize);

/// Represents a possible index that can be used to index the punctuations in a [`Punctuated`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PunctIndex(pub usize);



/// Defines the result of [pushing punctuation](Punctuated::push_punct()) to a [`Punctuated`]-
/// list.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PushPunctResult<P> {
    /// The punctuation was inserted as new.
    Inserted,
    /// The punctuation was inserted, but overwrite the old one, which is returned here.
    Overwrite(P),
    /// The punctuation was not inserted because there were no values.
    Skipped,
}





/***** LIBRARY *****/
/// Implements a "punctuated" array, i.e., an array of values interspersed with some punctuation
/// values.
///
/// See the [`snack`]-module for parsing such lists with [`snack`](::snack).
///
/// # Generics
/// - `V`: The type of values in the list.
/// - `P`: The type of punctuations in the list.
pub struct Punctuated<V, P> {
    /// The actual storage
    data: Vec<(V, MaybeUninit<P>)>,
    /// Whether the last `P` is there or not.
    ///
    /// The implementation takes great care to set this flag IF AND ONLY IF the last `MaybeUninit`
    /// is initialized. It guarantees that, for any non-last `MaybeUninit`s, they are always
    /// initialized.
    has_trailing: bool,
}

// Constructors
impl<V, P> Default for Punctuated<V, P> {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl<V, P> Punctuated<V, P> {
    /// Constructor for Punctuated that will initialize as empty.
    ///
    /// This function has the same guarantees as [`Vec::new()`], i.e., it will not allocate
    /// anything yet until pushed to it.
    ///
    /// # Returns
    /// A new Punctuated that is empty.
    #[inline]
    pub const fn new() -> Self { Self { data: Vec::new(), has_trailing: false } }

    /// Constructor for Punctuated that will initialize as empty, but with space for a certain
    /// number of values allocated.
    ///
    /// The list can, for the same capacity of values, store exactly that amount of punctuations.
    /// I.e., it has space for the right number of punctuation including a trailing one.
    ///
    /// Note that this constructor _does_ allocate immediately.
    ///
    /// # Arguments
    /// - `capacity`: The minimum number of values to allocate space for.
    ///
    /// # Returns
    /// A new Punctuated that is empty, but can insert at least `capacity` number of values &
    /// punctuations before it has to re-allocate.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self { Self { data: Vec::with_capacity(capacity), has_trailing: false } }

    /// Constructor for Punctuated that will initialize it around the given element.
    ///
    /// # Arguments
    /// - `value`: The first `V`alue to store in the Punctuated.
    ///
    /// # Returns
    /// A new Punctuated of size 1 with the given `value` as only element.
    #[inline]
    pub fn singleton(value: V) -> Self { Self { data: vec![(value, MaybeUninit::uninit())], has_trailing: false } }
}

// Destructors
impl<V, P> Drop for Punctuated<V, P> {
    #[inline]
    fn drop(&mut self) {
        // We need to drop the punctuation!
        let data_len: usize = self.data.len();
        for (i, (_, p)) in self.data.iter_mut().enumerate() {
            // SAFETY: The `- 1` can never fail because we won't enter the loop if there is no data
            if i < data_len - 1 || self.has_trailing {
                // SAFETY: There is a punctuation to drop here because we're either looking at the
                // non-last element (which are guaranteed to be there) OR if we've marked for
                // ourselves that the last element exists (we have a trailing punctuation).
                unsafe { p.assume_init_drop() };
            }
        }

        // Then leave it up to the normal drop for `Vec` to drop the rest
    }
}

// Pushing
impl<V, P: Default> Punctuated<V, P> {
    /// Pushes a new value to the list.
    ///
    /// If there was no trailing punctuation at the end of it, then it will insert a new one using
    /// [`P::default()`](Default::default()).
    ///
    /// # Arguments
    /// - `value`: Some new `V`alue to push to the end of the list.
    #[inline]
    pub fn push(&mut self, value: V) {
        // Inject a punctuation if necessary
        if !self.has_trailing {
            if let Some((_, p)) = self.data.last_mut() {
                p.write(Default::default());
                // We would have to update `has_trailing` here, but note that we'll push to the
                // list in a second anyway, which would invalidate it again
            }
        }

        // Add the element
        self.data.push((value, MaybeUninit::uninit()));
        self.has_trailing = false;
    }

    /// Extends the list with multiple new values.
    ///
    /// Missing punctuation, either at the end of the existing last value or in between given
    /// values, is inserted by using [`P::default()`](Default::default()).
    ///
    /// # Arguments
    /// - `values`: Something producing `V`alues to insert at the end of the list. Note that the
    ///   matching iterator's [`Iterator::size_hint()`] function is used to optimize the vector re-
    ///   allocation.
    pub fn extend(&mut self, values: impl IntoIterator<Item = V>) {
        // Get the iterator
        let mut values = values.into_iter();
        let size_hint: (usize, Option<usize>) = values.size_hint();
        let additional: usize = size_hint.1.unwrap_or(size_hint.0);

        // First, reserve enough space
        self.reserve(additional);

        // The first element, if any, is special because there may OR may not be a previous
        // trailing punctuation.
        if let Some(first) = values.next() {
            if let Some((_, p)) = self.data.last_mut() {
                if !self.has_trailing {
                    p.write(Default::default());
                    // We would have to update `has_trailing` here, but note that we'll push to the
                    // list in a second anyway, which would invalidate it again
                }
            }
            self.data.push((first, MaybeUninit::uninit()));
            self.has_trailing = false;
        }

        // Then, the rest is more straightforward because we know more about the previous element
        for value in values {
            // SAFETY: We can unwrap unchecked here because we only reach this loop if we've added
            // an element; so there is one
            let (_, p): &mut (_, MaybeUninit<P>) = unsafe { self.data.last_mut().unwrap_unchecked() };
            // We can just write here, no memory leak, because we know the previous loop did NOT
            // write a punctuation
            p.write(Default::default());

            // Now add the value
            self.data.push((value, MaybeUninit::uninit()));
            // The trailing is still false at the last element, so we keep `has_trailing` on false,
            // as set above
        }
    }
}
impl<V, P> Punctuated<V, P> {
    /// Pushes a new value to the list.
    ///
    /// You must be sure to already have pushed a `P`unctuation if this is not the first element in
    /// the list. When in doubt, you can only push when [`Punctuated::can_push_value()`] returns
    /// true.
    ///
    /// Alternatively, if your `P` implements [`Default`], refer to [`Punctuated::push()`] to
    /// inject any missing punctuation automatically instead.
    ///
    /// # Arguments
    /// - `value`: Some new `V`alue to push to the end of the list.
    ///
    /// # Panics
    /// This function will panic if this list is non-empty and has no trailing punctuation.
    #[inline]
    #[track_caller]
    pub fn push_value(&mut self, value: V) {
        // Assert first that there is a punctuation, to uphold our requirement that any non-last
        // `MaybeUninit`s are initialized.
        if !self.can_push_value() {
            panic!("Cannot push a value to a Punctuated without trailing punctuation");
        }

        // Now update the list
        self.data.push((value, MaybeUninit::uninit()));
        self.has_trailing = false;
    }

    /// Pushes a new value to the list _without_ checking it was safe to do so, first.
    ///
    /// You must be sure to already have pushed a `P`unctuation if this is not the first element in
    /// the list. When in doubt, you can only push when [`Punctuated::can_push_value()`] returns
    /// true.
    ///
    /// Use this function only when you're sure the requirement above is met. If you're not, use
    /// [`Punctuated::push_value()`] to have the implementation check it for you. Alternatively, if
    /// your `P` implements [`Default`], refer to [`Punctuated::push()`] to inject any missing
    /// punctuation automatically instead.
    ///
    /// # Arguments
    /// - `value`: Some new `V`alue to push to the end of the list.
    ///
    /// # Panics
    /// This function will panic if this list is non-empty and has no trailing punctuation.
    #[inline]
    #[track_caller]
    pub unsafe fn push_value_unchecked(&mut self, value: V) {
        // SAFETY: The user is responsible for ensuring that there is a trailing punctuation!

        // Now update the list
        self.data.push((value, MaybeUninit::uninit()));
        self.has_trailing = false;
    }

    /// Pushes a new punctuation to the list.
    ///
    /// This will add a trailing punctuation to the last value in the list. If there is no value,
    /// then this function will do nothing.
    ///
    /// If there already was a trailing punctuation, it will be overwritten.
    ///
    /// Because of this behaviour, this function does not affect the
    /// [capacity](Punctuated::capacity()) of the list.
    ///
    /// # Arguments
    /// - `punct`: Some new `P`unctuation to push to the end of the list.
    ///
    /// # Returns
    /// What has happened:
    /// - The value was [`PushResult::Inserted`];
    /// - The value was inserted, but the old one was [`PushResult::Drop`]ped; or
    /// - The vlaue was [`PushPunctResult::Skipped`]ped because there was no initial value.
    #[inline]
    pub fn push_punct(&mut self, punct: P) -> PushPunctResult<P> {
        // Decide if there is a value
        if let Some((_, p)) = self.data.last_mut() {
            // Insert a new P, first
            let mut punct: MaybeUninit<P> = MaybeUninit::new(punct);
            std::mem::swap(&mut punct, p);

            // Now decide if there was an old `p` already there
            if self.has_trailing {
                // SAFETY: We can assume that `punct` is initialized because `has_trailing` was
                // true before our swap
                PushPunctResult::Overwrite(unsafe { punct.assume_init() })
            } else {
                // There is now
                self.has_trailing = true;
                PushPunctResult::Inserted
            }
        } else {
            PushPunctResult::Skipped
        }
    }



    /// Returns whether it is safe to call [`Punctuated::push_value()`] or
    /// [`Punctuated::push_value_unchecked()`].
    ///
    /// Specifically, it is safe to do so when the list is empty ([`Punctuated::is_empty()`]) or
    /// otherwise has trailing punctuation ([`Punctuated::has_trailing()`]).
    ///
    /// # Returns
    /// True if the list if ready to accept a new `V`alue, or false otherwise.
    #[inline]
    pub fn can_push_value(&self) -> bool { self.is_empty() || self.has_trailing() }
}

// Popping
impl<V, P> Punctuated<V, P> {
    /// Pops the last value from the punctuated list.
    ///
    /// If this value had a trailing punctuation, it is popped also.
    ///
    /// Note that this function leaves the capacity of the list untouched.
    ///
    /// # Returns
    /// A pair of the removed value and, if there was a trailing punctuation, that punctuation. If
    /// the list was empty, [`None`] is returned.
    #[inline]
    pub fn pop(&mut self) -> Option<(V, Option<P>)> {
        // SAFETY: We can assume the `p` is initialized iff we remember that we hae a trailing
        // punctuation due to our internal guarantee.
        let res: Option<(V, Option<P>)> = self.data.pop().map(|(v, p)| (v, if self.has_trailing { Some(unsafe { p.assume_init() }) } else { None }));

        // Before we return, be sure to accurately update `has_trailing`
        if !self.data.is_empty() {
            // There's an element left. By virtue of us promising ourselves that any non-last
            // elements are initialized, we can deduce there is a trailing punctuation now.
            self.has_trailing = true;
        } else {
            // We must be certain there isn't any.
            self.has_trailing = false;
        }

        // OK, return
        res
    }

    /// Pops the trailing punctuation off this list, if any.
    ///
    /// This will _not_ remove values if there is no trailing punctuation.
    ///
    /// Note that this function leaves the capacity of the list untouched.
    ///
    /// # Returns
    /// The popped `P`unctuation if there was any trailing.
    #[inline]
    pub fn pop_trailing_punct(&mut self) -> Option<P> {
        // It all depends on whether there is some trailing punctuation
        if self.has_trailing {
            self.has_trailing = false;
            // SAFETY: Three remarks:
            // 1. We can unwrap the last element because, in order to have a trailing punctuation,
            //    there must be at least one element;
            // 2. Because we marked there is a trailing punctuation, by our guarantee, we assume it
            //    is initialized; and
            // 3. We will never read more than once because we now mark that we _don't_ have
            //    trailing punctuation. As such, we are sure that the resulting object's associated
            //    memory won't get deallocated twice or something, leaving the original bits to be
            //    interpreted as uninitialized.
            Some(unsafe { self.data.last().unwrap_unchecked().1.assume_init_read() })
        } else {
            None
        }
    }

    /// Removes a `(value, punctuation)`-pair from the punctuated list and returns them both.
    ///
    /// # Arguments
    /// - `index`: The position in the list of the pair to remove.
    ///
    /// # Returns
    /// A `(V, Option<P>)`-pair that is the removed pair.
    ///
    /// # Panics
    /// This function panics if `index` is out-of-bounds for the list.
    #[inline]
    #[track_caller]
    pub fn remove(&mut self, index: usize) -> (V, Option<P>) {
        // Assert the rangeness first
        let data_len: usize = self.data.len();
        let has_trailing: bool = self.has_trailing;
        if index >= data_len {
            panic!("Index {index} is out-of-bounds for a Punctuated of length {data_len}");
        }
        let (v, p): (V, MaybeUninit<P>) = self.data.remove(index);
        // SAFETY: We can do `data_len - 1` because we know there is at least 1 element (else,
        // `index` could not have been within range).
        if index == data_len - 1 {
            // If we removed the last element, we know we now have a trailing punctuation because
            // all non-last elements do
            self.has_trailing = true;
        }
        // SAFETY: We can assume `p` is initialized because we uphold our guarantees with the
        // preceding if-statement (it is either not the last element, or if it is, we remember it
        // is trailing).
        (v, if index < data_len - 1 || has_trailing { Some(unsafe { p.assume_init() }) } else { None })
    }

    /// Removes a given range of values from the punctuated list.
    ///
    /// This operation is very similar to [`Punctuated::drain()`], except that it does not return
    /// the removed elements. Due to that function's relatively unoptimized implementation, using
    /// this one can achieve nice speed benifits if it's possible for you to use it.
    ///
    /// # Arguments
    /// - `range`: Some range of indices that determines the slice of values to remove from the
    ///   list.
    ///
    /// # Returns
    /// How many elements were actually removed from the range.
    #[inline]
    pub fn remove_range(&mut self, range: impl RangeBounds<usize>) {
        // Step 1: Resolve the range to a concrete one of start- and stop indices
        let data_len: usize = self.data.len();
        let (start, end): (usize, usize) = resolve_range(range, data_len);
        if start >= end {
            // The resulting range is empty. Nothing to do!
            return;
        }

        // Step 2: Deallocate anything not needed
        for i in start..end {
            // SAFETY: We can unwrap the data at this index, because `resolve_range()` guarantees
            // that anything within `start..end` is within range of the list (after checking it is
            // non-negative, that is).
            let (v, p): &mut (V, MaybeUninit<P>) = unsafe { self.data.get_unchecked_mut(i) };

            // Always drop `v`
            // SAFETY: There's quite a lot of properties to uphold, but let's just say that `V` is
            // valid for dropping because we know we are 1) within range of the list, and 2) we use
            // the normal `Vec` guarantees for ensuring that the memory is read- and writable,
            // properly aligned and that `V` hasn't been deallocated before etc.
            unsafe { std::ptr::drop_in_place(v as *mut V) };

            // Then drop `p` if there is any
            // SAFETY: We can do `data_len - 1` here because we know the range is non-empty
            if i < data_len - 1 || self.has_trailing {
                // SAFETY: We can assume that `p` is initialized and valid because our if-statement
                // ensures that we're in a position where the guarantees we're trying to provide
                // also hold: we're either dropping a non-last punctuation, or we remember there
                // was any.
                unsafe { p.assume_init_drop() };
            }
        }

        // Step 3: Move the rest of the values back to take up the place of the dropped ones.
        let pre_len = start;
        let aft_len = self.data.len() - end;
        if aft_len > 0 {
            // SAFETY: Here we go again
            // 1. `src` is valid, because we know there's exactly `aft_len` elements between `end`
            //    and the end of the data array;
            // 2. `dst` is valid, because we know start < end and therefore there is always enough
            //    space. Further, reading from the source will not invalidate the destination
            //    pointer.
            // 3. They are also both properly aligned due to guarantees by `Vec`.
            //
            // We don't need to worry about deallocating the old elements, because they are already
            // copied to `res`.
            unsafe { std::ptr::copy(self.data[end..].as_ptr(), self.data[start..].as_mut_ptr(), aft_len) }
        } else {
            // If there _is_ nothing to copy, it means that we have removed the last element from
            // the list. Hence, we know that there WILL be a trailing punctuation (it remains from
            // a non-last element, which is guaranteed to have it) EXCEPT for when the resulting
            // list is empty.
            if pre_len > 0 {
                self.has_trailing = true;
            } else {
                self.has_trailing = false;
            }
        }
        // SAFETY: We won't leak any memory, because the new length only covers elements not
        // already deallocated.
        unsafe {
            self.data.set_len(pre_len + aft_len);
        }
        // NOTE: The original vector is now in a valid state again

        // Done
    }

    /// Drains the given ranges of values from the punctuated list.
    ///
    /// # Optimization
    /// Note that, for safety reasons, this draining is less optimized than that of the standard
    /// library's [`Vec`](Vec::drain()). In particular, this function will ensure that the list is
    /// already updated _before_ the iterator is returned, meaning it does not depent on the
    /// original vector anymore.
    ///
    /// As such, if you're not interested in the results, consider using
    /// [`Punctuated::remove_range()`] instead.
    ///
    /// # Arguments
    /// - `range`: Some range of indices that determines the slice of values to remove from the
    ///   list.
    ///
    /// # Returns
    /// A [`DoubleEndedIterator`] yielding removed elements. Note that, for efficiency purposes,
    /// this has a mutable claim on `self`.
    #[inline]
    pub fn drain<'s>(&'s mut self, range: impl RangeBounds<usize>) -> Drain<V, P> {
        // Step 1: Resolve the range to a concrete one of start- and stop indices
        let data_len: usize = self.data.len();
        let (start, end): (usize, usize) = resolve_range(range, data_len);
        if start >= end {
            // The resulting range is empty. Nothing to do!
            return Punctuated::new().into_iter();
        }

        // Now we start to move objects out of the list
        let slice_len: usize = end - start;
        let mut res: Vec<(V, MaybeUninit<P>)> = Vec::with_capacity(slice_len);
        // SAFETY: Let's check all three requirements:
        // 1. `src` is valid for `slice_len * size_of::<(V, MaybeUninit<P>)>` reads because it is
        //    an array with those elements and we already asserted that `start..end` is a non-empty
        //    range, and within bounds of `self.data`.
        // 2. `dst` is also valid for `slice_len * size_of::<(V, MaybeUninit<P>)>` writes because
        //    we have allocated an appropriate vector above. We're also sure that it will remain
        //    valid during the copy.
        // 3. They are also both properly aligned due to guarantees by `Vec`.
        // 4. We guarantee the memory ranges are non-overlapping because they ar from different
        //    allocations.
        //
        // Finally, separately, we promise ourselves that we won't re-use the copied contents of
        // `self.data` because they may not be `Copy` (i.e., there's two "handles" to the same
        // objects, potentially).
        unsafe { std::ptr::copy_nonoverlapping(self.data[start..end].as_ptr(), res.as_mut_ptr(), slice_len) }
        // SAFETY: We can now update the `res` to have the correct size, as we just copied
        // `slice_len` elements in the previous step
        unsafe { res.set_len(slice_len) };

        // Update the old vector to move the elements back if needed
        let pre_len = start;
        let aft_len = self.data.len() - end;
        let res_has_trailing: bool = if aft_len > 0 {
            // SAFETY: Here we go again
            // 1. `src` is valid, because we know there's exactly `aft_len` elements between `end`
            //    and the end of the data array;
            // 2. `dst` is valid, because we know start < end and therefore there is always enough
            //    space. Further, reading from the source will not invalidate the destination
            //    pointer.
            // 3. They are also both properly aligned due to guarantees by `Vec`.
            //
            // We don't need to worry about deallocating the old elements, because they are already
            // copied to `res`.
            unsafe { std::ptr::copy(self.data[end..].as_ptr(), self.data[start..].as_mut_ptr(), aft_len) }
            // We know the removed area has a trailing punctuation because it's not the last one
            true
        } else {
            // If there _is_ nothing to copy, it means that we have removed the last element from
            // the list. Hence, we know that there WILL be a trailing punctuation (it remains from
            // a non-last element, which is guaranteed to have it) EXCEPT for when the resulting
            // list is empty.
            let res_has_trailing: bool = self.has_trailing;
            if pre_len > 0 {
                self.has_trailing = true;
            } else {
                self.has_trailing = false;
            }
            // The last element is in the removed slice. Hence, the old value will determinate if
            // the slice is trailing.
            res_has_trailing
        };
        // SAFETY: We won't leak any memory, because the new length only covers elements not in
        // `res`
        unsafe {
            self.data.set_len(pre_len + aft_len);
        }
        // NOTE: The original vector is now in a valid state again

        // We've done the copy, now all that remains is returning the data structure!
        Punctuated { data: res, has_trailing: res_has_trailing }.into_iter()
    }
}

// Non-resizing element access
impl<V, P> Punctuated<V, P> {
    /// Returns the first _value_ in the punctuated list.
    ///
    /// This is simply an alias for calling [`Punctuated::get(0)`](Punctuated::get()).
    ///
    /// # Returns
    /// A reference to the first value in the list, or [`None`] if it is empty.
    #[inline]
    pub fn first(&self) -> Option<&V> { self.data.first().map(|(v, _)| v) }

    /// Returns the first _value_ in the punctuated list, mutably.
    ///
    /// This is simply an alias for calling [`Punctuated::get_mut(0)`](Punctuated::get_mut()).
    ///
    /// # Returns
    /// A mutable reference to the first value in the list, or [`None`] if it is empty.
    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut V> { self.data.first_mut().map(|(v, _)| v) }

    /// Returns the first value/punctuation pair in the punctuated list.
    ///
    /// This is simply an alias for calling [`Punctuated::get_pair(0)`](Punctuated::get_pair()).
    ///
    /// # Returns
    /// A reference to the first value and, if any, its punctuation in the list, or [`None`] if it
    /// is empty.
    #[inline]
    pub fn first_pair(&self) -> Option<(&V, Option<&P>)> {
        // SAFETY: We can assume `p` is initialized because we assert it is either a non-last
        // value, in which case it's guaranteed to be initialized; or else we remember it existing.
        self.data.first().map(|(v, p)| (v, if self.data.len() > 1 || self.has_trailing { Some(unsafe { p.assume_init_ref() }) } else { None }))
    }

    /// Returns the first value/punctuation pair in the punctuated list, mutably.
    ///
    /// This is simply an alias for calling
    /// [`Punctuated::get_pair_mut(0)`](Punctuated::get_pair_mut()).
    ///
    /// # Returns
    /// A mutable reference to the first value and, if any, its punctuation in the list, or
    /// [`None`] if it is empty.
    #[inline]
    pub fn first_pair_mut(&mut self) -> Option<(&mut V, Option<&mut P>)> {
        let data_len: usize = self.data.len();
        // SAFETY: We can assume `p` is initialized because we assert it is either a non-last
        // value, in which case it's guaranteed to be initialized; or else we remember it existing.
        self.data.first_mut().map(|(v, p)| (v, if data_len > 1 || self.has_trailing { Some(unsafe { p.assume_init_mut() }) } else { None }))
    }



    /// Returns the last _value_ in the punctuated list.
    ///
    /// This is simply an alias for calling [`Punctuated::get(0)`](Punctuated::get()).
    ///
    /// # Returns
    /// A reference to the last value in the list, or [`None`] if it is empty.
    #[inline]
    pub fn last(&self) -> Option<&V> { self.data.last().map(|(v, _)| v) }

    /// Returns the last _value_ in the punctuated list, mutably.
    ///
    /// This is simply an alias for calling [`Punctuated::get_mut(0)`](Punctuated::get_mut()).
    ///
    /// # Returns
    /// A mutable reference to the last value in the list, or [`None`] if it is empty.
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut V> { self.data.last_mut().map(|(v, _)| v) }

    /// Returns the last value/punctuation pair in the punctuated list.
    ///
    /// This is simply an alias for calling [`Punctuated::get_pair(0)`](Punctuated::get_pair()).
    ///
    /// # Returns
    /// A reference to the last value and, if any, its punctuation in the list, or [`None`] if it
    /// is empty.
    #[inline]
    pub fn last_pair(&self) -> Option<(&V, Option<&P>)> {
        // SAFETY: `p` is guaranteed to be the last element, so it is only initialized if there is
        // a trailing punctuation.
        self.data.last().map(|(v, p)| (v, if self.has_trailing { Some(unsafe { p.assume_init_ref() }) } else { None }))
    }

    /// Returns the last value/punctuation pair in the punctuated list, mutably.
    ///
    /// This is simply an alias for calling
    /// [`Punctuated::get_pair_mut(0)`](Punctuated::get_pair_mut()).
    ///
    /// # Returns
    /// A mutable reference to the last value and, if any, its punctuation in the list, or
    /// [`None`] if it is empty.
    #[inline]
    pub fn last_pair_mut(&mut self) -> Option<(&mut V, Option<&mut P>)> {
        // SAFETY: `p` is guaranteed to be the last element, so it is only initialized if there is
        // a trailing punctuation.
        self.data.last_mut().map(|(v, p)| (v, if self.has_trailing { Some(unsafe { p.assume_init_mut() }) } else { None }))
    }



    /// Returns the _value_ at the given index of the punctuated list.
    ///
    /// # Arguments
    /// - `index`: The index to get the element at.
    ///
    /// # Returns
    /// A reference to the value in the list at position `index`, or [`None`] if the `index` was
    /// out-of-bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&V> { self.data.get(index).map(|(v, _)| v) }

    /// Returns the _value_ at the given index of the punctuated list.
    ///
    /// This version of the function will _not_ check whether this value exists. As such, it is
    /// **your** responsibility that `index` is within bounds! (I.e.,
    /// [`index < Punctuated::len()`](Punctuated::len()))
    ///
    /// If you don't want to make such guarantees yourself, use the safe [`Punctuated::get()`]
    /// counterpart.
    ///
    /// # Arguments
    /// - `index`: The index to get the element at.
    ///
    /// # Returns
    /// A reference to the value at the given index.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &V { unsafe { &self.data.get_unchecked(index).0 } }

    /// Returns the _value_ at the given index of the punctuated list, mutably.
    ///
    /// # Arguments
    /// - `index`: The index to get the element at.
    ///
    /// # Returns
    /// A mutable reference to the value in the list at position `index`, or [`None`] if the
    /// `index` was out-of-bounds.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut V> { self.data.get_mut(index).map(|(v, _)| v) }

    /// Returns the _value_ at the given index of the punctuated list, mutably.
    ///
    /// This version of the function will _not_ check whether this value exists. As such, it is
    /// **your** responsibility that `index` is within bounds! (I.e.,
    /// [`index < Punctuated::len()`](Punctuated::len()))
    ///
    /// If you don't want to make such guarantees yourself, use the safe
    /// [`Punctuated::get_mut()`] counterpart.
    ///
    /// # Arguments
    /// - `index`: The index to get the element at.
    ///
    /// # Returns
    /// A mutable reference to the value at the given index.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut V { unsafe { &mut self.data.get_unchecked_mut(index).0 } }



    /// Returns the _punctuation_ at the given index of the punctuated list.
    ///
    /// # Arguments
    /// - `index`: The index to get the punctuation at.
    ///
    /// # Returns
    /// A reference to the punctation in the list at position `index`, or [`None`] if the `index`
    /// was out-of-bounds. It may also return [`None`] if `index` pointed to the end of the list
    /// but there was no trailing punctuation.
    #[inline]
    pub fn get_punct(&self, index: usize) -> Option<&P> {
        // Handle the special end-of-list case first
        if index == self.data.len() - 1 && !self.has_trailing {
            return None;
        }

        // Now we can get the punctuation
        // SAFETY: This is OK because we guarantee that 1) `p` is initialized for every non-last
        // punctuation, and 2) that we quit if the last `p` is uninitialized above.
        self.data.get(index).map(|(_, p)| unsafe { p.assume_init_ref() })
    }

    /// Returns the _punctuation_ at the given index of the punctuated list.
    ///
    /// This version of the function will _not_ check whether this punctuation exists. As such, it
    /// is **your** responsibility that `index` is BOTH:
    /// 1. within bounds! (I.e., [`index < Punctuated::len()`](Punctuated::len())); and
    /// 2. _if_ `index` points to the last element, there is a trailing punctuation to return.
    ///
    /// Failure to guarantee either will result in UB! If you don't want to make such guarantees
    /// yourself, use the safe [`Punctuated::get_punct()`] instead.
    ///
    /// # Arguments
    /// - `index`: The index to get the element at.
    ///
    /// # Returns
    /// A reference to the punctuation at the given index.
    #[inline]
    pub unsafe fn get_punct_unchecked(&self, index: usize) -> &P {
        // SAFETY: Nothing. Everything's up to the user. However, we've done our part by adding the
        // requirements to this function's description.
        unsafe { self.data.get_unchecked(index).1.assume_init_ref() }
    }

    /// Returns the _punctuation_ at the given index of the punctuated list, mutably.
    ///
    /// # Arguments
    /// - `index`: The index to get the punctuation at.
    ///
    /// # Returns
    /// A mutable reference to the punctation in the list at position `index`, or [`None`] if the
    /// `index` was out-of-bounds. It may also return [`None`] if `index` pointed to the end of the
    /// list but there was no trailing punctuation.
    #[inline]
    pub fn get_punct_mut(&mut self, index: usize) -> Option<&mut P> {
        // Handle the special end-of-list case first
        if index == self.data.len() - 1 && !self.has_trailing {
            return None;
        }

        // Now we can get the punctuation
        // SAFETY: This is OK because we guarantee that 1) `p` is initialized for every non-last
        // punctuation, and 2) that we quit if the last `p` is uninitialized above.
        self.data.get_mut(index).map(|(_, p)| unsafe { p.assume_init_mut() })
    }

    /// Returns the _punctuation_ at the given index of the punctuated list, mutably.
    ///
    /// This version of the function will _not_ check whether this punctuation exists. As such, it
    /// is **your** responsibility that `index` is BOTH:
    /// 1. within bounds! (I.e., [`index < Punctuated::len()`](Punctuated::len())); and
    /// 2. _if_ `index` points to the last element, there is a trailing punctuation to return.
    ///
    /// Failure to guarantee either will result in UB! If you don't want to make such guarantees
    /// yourself, use the safe [`Punctuated::get_punct_mut()`] instead.
    ///
    /// # Arguments
    /// - `index`: The index to get the element at.
    ///
    /// # Returns
    /// A mutably reference to the punctuation at the given index.
    #[inline]
    pub unsafe fn get_punct_unchecked_mut(&mut self, index: usize) -> &mut P {
        // SAFETY: Nothing. Everything's up to the user. However, we've done our part by adding the
        // requirements to this function's description.
        unsafe { self.data.get_unchecked_mut(index).1.assume_init_mut() }
    }



    /// Returns a full pair of a value with its (optional) punctuation at the given index of the
    /// punctuated list.
    ///
    /// # Arguments
    /// - `index`: The index to get the value & punctuation at.
    ///
    /// # Returns
    /// A reference to a pair of value and its suffixing punctuation, if any, or [`None`] if
    /// `index` is out-of-bounds.
    ///
    /// The punctuation is only missing if `index` points to the last element, but there was no
    /// [trailing punctuation](Punctuated::has_trailing()).
    #[inline]
    pub fn get_pair(&self, index: usize) -> Option<(&V, Option<&P>)> {
        self.data
            .get(index)
            // SAFETY: We can assume the `p` is initialized here because the if-statement preceding
            // it assures it either 1) regards a non-last `p` (which, by our guarantee, is always
            // initialized), or 2) regards a trailing punctuation we know exists (due to our other
            // guarantee when `has_trailing` is true)
            .map(|(v, p)| (v, if index < self.data.len() - 1 || self.has_trailing { Some(unsafe { p.assume_init_ref() }) } else { None }))
    }

    /// Returns a full pair of a value with its (optional) punctuation at the given index of the
    /// punctuated list.
    ///
    /// This version of the function will _not_ check whether this pair exists. As such, it is
    /// **your** responsibility that `index` is within bounds! (I.e.,
    /// [`index < Punctuated::len()`](Punctuated::len()))
    ///
    /// If you don't want to make such guarantees yourself, use the safe
    /// [`Punctuated::get_pair()`] counterpart.
    ///
    /// # Arguments
    /// - `index`: The index to get the element at.
    ///
    /// # Returns
    /// A reference to a pair of value and its suffixing punctuation, if any.
    ///
    /// The punctuation is only missing if `index` points to the last element, but there was no
    /// [trailing punctuation](Punctuated::has_trailing()).
    #[inline]
    pub unsafe fn get_pair_unchecked(&self, index: usize) -> (&V, Option<&P>) {
        // SAFETY: Nothing. Everything's up to the user. However, we've done our part by adding the
        // requirements to this function's description.
        let (v, p): &(V, MaybeUninit<P>) = unsafe { self.data.get_unchecked(index) };
        // SAFETY: This one _is_ ours to uphold, which we do, because the if-statement preceding
        // it assures it either 1) regards a non-last `p` (which, by our guarantee, is always
        // initialized), or 2) regards a trailing punctuation we know exists (due to our other
        // guarantee when `has_trailing` is true)
        (v, if index < self.data.len() - 1 || self.has_trailing { Some(unsafe { p.assume_init_ref() }) } else { None })
    }

    /// Returns a full pair of a value with its (optional) punctuation at the given index of the
    /// punctuated list, mutably.
    ///
    /// # Arguments
    /// - `index`: The index to get the value & punctuation at.
    ///
    /// # Returns
    /// A mutable reference to a pair of value and its suffixing punctuation, if any, or [`None`]
    /// if `index` is out-of-bounds.
    ///
    /// The punctuation is only missing if `index` points to the last element, but there was no
    /// [trailing punctuation](Punctuated::has_trailing()).
    #[inline]
    pub fn get_pair_mut(&mut self, index: usize) -> Option<(&mut V, Option<&mut P>)> {
        let data_len: usize = self.data.len();
        self.data
            .get_mut(index)
            // SAFETY: We can assume the `p` is initialized here because the if-statement preceding
            // it assures it either 1) regards a non-last `p` (which, by our guarantee, is always
            // initialized), or 2) regards a trailing punctuation we know exists (due to our other
            // guarantee when `has_trailing` is true)
            .map(|(v, p)| (v, if index < data_len - 1 || self.has_trailing { Some(unsafe { p.assume_init_mut() }) } else { None }))
    }

    /// Returns a full pair of a value with its (optional) punctuation at the given index of the
    /// punctuated list, mutably.
    ///
    /// This version of the function will _not_ check whether this pair exists. As such, it is
    /// **your** responsibility that `index` is within bounds! (I.e.,
    /// [`index < Punctuated::len()`](Punctuated::len()))
    ///
    /// If you don't want to make such guarantees yourself, use the safe
    /// [`Punctuated::get_pair()`] counterpart.
    ///
    /// # Arguments
    /// - `index`: The index to get the element at.
    ///
    /// # Returns
    /// A mutable reference to a pair of value and its suffixing punctuation, if any.
    ///
    /// The punctuation is only missing if `index` points to the last element, but there was no
    /// [trailing punctuation](Punctuated::has_trailing()).
    #[inline]
    pub unsafe fn get_pair_unchecked_mut(&mut self, index: usize) -> (&mut V, Option<&mut P>) {
        let data_len: usize = self.data.len();
        // SAFETY: Nothing. Everything's up to the user. However, we've done our part by adding the
        // requirements to this function's description.
        let (v, p): &mut (V, MaybeUninit<P>) = unsafe { self.data.get_unchecked_mut(index) };
        // SAFETY: This one _is_ ours to uphold, which we do, because the if-statement preceding
        // it assures it either 1) regards a non-last `p` (which, by our guarantee, is always
        // initialized), or 2) regards a trailing punctuation we know exists (due to our other
        // guarantee when `has_trailing` is true)
        (v, if index < data_len - 1 || self.has_trailing { Some(unsafe { p.assume_init_mut() }) } else { None })
    }
}

// Capacity
impl<V, P> Punctuated<V, P> {
    /// Ensures there is space for at least a given number of additional values.
    ///
    /// This function has the same guarantees as [`Vec::reserve()`]. Specifically, it guarantees
    /// that no reallocation will happen if the [`Punctuated::capacity()`] is enough to cover the
    /// requested number of values.
    ///
    /// Note that, because values are paired with punctuations, pushing punctuations does not
    /// matter for the capacity; i.e., there is space for exactly the same number of punctuations
    /// as there is for values.
    ///
    /// # Arguments
    /// - `additional`: The number of _additional_ values to _at least_ reserve space for.
    #[inline]
    pub fn reserve(&mut self, additional: usize) { self.data.reserve(additional) }

    /// Returns the number of values this list can store before having to re-allocate.
    ///
    /// Note that, unlike [`Punctuated::reserve()`], this is the _total_ capacity of the list. Not
    /// "what remains".
    ///
    /// For every value in the list, a punctuation can be stored for free, because they are matched
    /// to values.
    ///
    /// # Returns
    /// The number of values that can be stored in this list before reallocation.
    #[inline]
    pub fn capacity(&self) -> usize { self.data.capacity() }
}

// Properties
impl<V, P> Punctuated<V, P> {
    /// Returns whether this list has a trailing punctuation.
    ///
    /// To have a trailing punctuation, there must be at least one element, and then
    /// [`Punctuated::push_punct()`] must have been called after anything pushing a value.
    ///
    /// # Returns
    /// True if there is a trailing punctuation stored in this list, or false otherwise.
    #[inline]
    pub const fn has_trailing(&self) -> bool { self.has_trailing }

    /// Returns the number of values in this list.
    ///
    /// You can deduce the number of punctuations from that. It is equal to the number of values,
    /// minus 1 but only [if there is _no_ trailing punctuation](Punctuated::has_trailing()). For
    /// your convenience, this is done automatically in [`Punctuated::punct_len()`].
    ///
    /// # Returns
    /// The number of values stored in this list.
    #[inline]
    pub fn len(&self) -> usize { self.data.len() }

    /// Returns the number of punctuation in this list.
    ///
    /// This is usually the same as the [number of values](Punctuated::len()), except that it may
    /// be one smaller [if a trailing punctuation is _not_ present](Punctuated::has_trailing()).
    ///
    /// # Returns
    /// The number of punctuations stored in this list.
    #[inline]
    pub fn punct_len(&self) -> usize { self.data.len().saturating_sub(if !self.has_trailing { 1 } else { 0 }) }

    /// Returns whether this list contains anything.
    ///
    /// # Returns
    /// True if there are no values or punctuations, or false otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
}

// Operators
impl<V: Clone, P: Clone> Clone for Punctuated<V, P> {
    #[inline]
    fn clone(&self) -> Self {
        // Allocate a vector with unintialized memory
        let data_len: usize = self.data.len();
        let mut data: Vec<MaybeUninit<(V, MaybeUninit<P>)>> = Vec::with_capacity(data_len);
        for _ in 0..data_len {
            data.push(MaybeUninit::uninit());
        }

        // Copy the elements over one-by-one
        for (i, (v, p)) in self.data.iter().enumerate() {
            // We will write in the uninitialized memory bc we're cool
            // SAFETY: We know that `i` is in range because it has the same length as `self.data`
            unsafe { data.get_unchecked_mut(i) }.write((
                v.clone(),
                // SAFETY: We clone the `p` if it exists, which is only does if it's a non-last
                // punctuation OR we have a trailing punctuation.
                if i < data_len - 1 || self.has_trailing { MaybeUninit::new(unsafe { p.assume_init_ref() }.clone()) } else { MaybeUninit::uninit() },
            ));
        }

        // That'll do her
        // SAFETY: This transmute is safe, because `MaybeUninit` has the same layouting guarantees
        // as the tuple, which it wraps. Further, it is an issue if the container is a struct
        // (due to field ordering; not applicable) or if it relies on e.g. memory niches (not
        // applicable either).
        Self { data: unsafe { std::mem::transmute(data) }, has_trailing: self.has_trailing }
    }
}
impl<V: Debug, P: Debug> Debug for Punctuated<V, P> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        let data_len: usize = self.data.len();
        let mut fmt = f.debug_list();
        for (i, (v, p)) in self.data.iter().enumerate() {
            fmt.entry(v);
            if i < data_len - 1 || self.has_trailing {
                // SAFETY: We can assume the punctuation is initialized because:
                // - We are either not at the last element, in which case we rely on our promise
                //   that any non-last punctuation is initialized; or
                // - We are at the last element but we noted down it is trailing, relying on our
                //   promise this flag is accurate.
                fmt.entry(unsafe { p.assume_init_ref() });
            }
        }
        fmt.finish()
    }
}
impl<V: Eq, P: Eq> Eq for Punctuated<V, P> {}
impl<V: PartialEq, P: PartialEq> PartialEq for Punctuated<V, P> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        // Ensure the lists have similar length- and trailingness
        if self.data.len() != other.data.len() || self.has_trailing != other.has_trailing {
            return false;
        }

        // Otherwise, they must match element-for-element
        for (i, ((lv, lp), (rv, rp))) in self.data.iter().zip(other.data.iter()).enumerate() {
            // Values are easy
            if lv.ne(rv) {
                return false;
            }

            // Punctuation requires our usual guard
            if i < self.data.len() - 1 || self.has_trailing {
                // SAFETY: We can assume _both_ punctuations are initialized here because:
                // - `lp` is in a spot where our guarantees allow us to (it's not the last element
                //   OR we noted down there is a trailing punctuation); and
                // - `rp` has the same length- and trailingness properties as `lp` (the if-
                //   statement) at the start of the function).
                if unsafe { lp.assume_init_ref() }.ne(unsafe { rp.assume_init_ref() }) {
                    return false;
                }
            }
        }

        // We made it
        true
    }
}
impl<V: Hash, P: Hash> Hash for Punctuated<V, P> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // We'll just has every element in the list
        for (i, (v, p)) in self.data.iter().enumerate() {
            // Hashing `v` is easy
            v.hash(state);

            // Hashing `p` requires our usual guard
            if i < self.data.len() - 1 || self.has_trailing {
                // SAFETY: We can assume `p` is initialized because it's protected by our
                // guarantees: either it is the non-last `p`, which is always initialized, or we
                // have remembered it is trailing.
                unsafe { p.assume_init_ref() }.hash(state);
            }
            // Otherwise, hash nothing
        }
    }
}
impl<V: Ord, P: Ord> Ord for Punctuated<V, P> {
    #[inline]
    #[track_caller]
    fn cmp(&self, other: &Self) -> Ordering {
        // We simply rely on `PartialOrd` being implemented correctly
        self.partial_cmp(other).unwrap_or_else(|| {
            panic!(
                "Expected Punctuated::partial_cmp() to return non-None result, but it did. This can happen if one of `V` or `P`'s `partial_cmp()` \
                 implementations returns `None`, which is not correct if they also implement `Ord`"
            )
        })
    }
}
impl<V: PartialOrd, P: PartialOrd> PartialOrd for Punctuated<V, P> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Then try every element
        for (i, ((lv, lp), (rv, rp))) in self.data.iter().zip(other.data.iter()).enumerate() {
            // If `v` is not the same, that determines the ordering
            let cmp: Ordering = lv.partial_cmp(rv)?;
            if !cmp.is_eq() {
                return Some(cmp);
            }

            // Then also compare `p`
            if (i < self.data.len() - 1 || self.has_trailing) && (i < other.data.len() - 1 || other.has_trailing) {
                // SAFETY: We can assume both `p`s are initialized here because we do the usual
                // guarantee check beforehand for both. I.e., we know that for both lists, it is
                // either not the last `p` or it is but we noted it exists.
                let cmp: Ordering = unsafe { lp.assume_init_ref() }.partial_cmp(unsafe { rp.assume_init_ref() })?;
                if !cmp.is_eq() {
                    return Some(cmp);
                }
            } else if !self.has_trailing && other.has_trailing {
                // These two if-statement can only occur if one of the two lists is at the end and
                // does _not_ have a trailing punctuation. Regardless of what the other list has,
                // which is either a non-last punctuation, a trailing punctuation or no
                // punctuation, this list should be deemed smaller because it has simple ended
                // earlier.
                //
                // This specific branch is for when `self` has ended but `other` not, i.e., we are
                return Some(Ordering::Less);
            } else if self.has_trailing && !other.has_trailing {
                // These two if-statement can only occur if one of the two lists is at the end and
                // does _not_ have a trailing punctuation. Regardless of what the other list has,
                // which is either a non-last punctuation, a trailing punctuation or no
                // punctuation, this list should be deemed smaller because it has simple ended
                // earlier.
                //
                // This specific branch is for when `other` has ended but `self` not, i.e., we are
                return Some(Ordering::Greater);
            }
            // The last case, when both have ended and have no trailing, is handled by our
            // finishing statement and hence requires no special treatment
        }

        // If all elements are the same, then their relative lengths determines the ordering
        Some(self.data.len().cmp(&other.data.len()))
    }
}
impl<V, P> Index<usize> for Punctuated<V, P> {
    type Output = V;

    #[inline]
    #[track_caller]
    fn index(&self, index: usize) -> &Self::Output { <Self as Index<ValueIndex>>::index(self, ValueIndex(index)) }
}
impl<V, P> IndexMut<usize> for Punctuated<V, P> {
    #[inline]
    #[track_caller]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output { <Self as IndexMut<ValueIndex>>::index_mut(self, ValueIndex(index)) }
}
impl<V, P> Index<ValueIndex> for Punctuated<V, P> {
    type Output = V;

    #[inline]
    fn index(&self, ValueIndex(index): ValueIndex) -> &Self::Output {
        self.get(index).unwrap_or_else(|| panic!("Value index {index} is out-of-bounds for Punctuated of length {}", self.len()))
    }
}
impl<V, P> IndexMut<ValueIndex> for Punctuated<V, P> {
    #[inline]
    #[track_caller]
    fn index_mut(&mut self, ValueIndex(index): ValueIndex) -> &mut Self::Output {
        let self_len: usize = self.len();
        self.get_mut(index).unwrap_or_else(|| panic!("Value index {index} is out-of-bounds for Punctuated of length {self_len}"))
    }
}
impl<V, P> Index<PunctIndex> for Punctuated<V, P> {
    type Output = P;

    #[inline]
    fn index(&self, PunctIndex(index): PunctIndex) -> &Self::Output {
        self.get_punct(index).unwrap_or_else(|| panic!("Punctuation index {index} is out-of-bounds for Punctuated of length {}", self.len()))
    }
}
impl<V, P> IndexMut<PunctIndex> for Punctuated<V, P> {
    #[inline]
    fn index_mut(&mut self, PunctIndex(index): PunctIndex) -> &mut Self::Output {
        let self_len: usize = self.len();
        self.get_punct_mut(index).unwrap_or_else(|| panic!("Punctuation index {index} is out-of-bounds for Punctuated of length {self_len}"))
    }
}

// Iteration
impl<V, P> Punctuated<V, P> {
    /// Returns an iterator over references to pairs of values and punctuations.
    ///
    /// For all non-last elements yielded by it, it holds that `P` is always [`Some`]. Only for the
    /// last element, if [`Punctuated::has_trailing()`] returns false, is it [`None`].
    ///
    /// # Returns
    /// An iterator yielding `(V, Option<P>)`-pairs.
    #[inline]
    pub fn iter(&self) -> Iter<'_, V, P> { Iter { iter: self.data.iter().enumerate(), len: self.data.len(), has_trailing: self.has_trailing } }

    /// Returns an iterator over mutable references to pairs of values and punctuations.
    ///
    /// For all non-last elements yielded by it, it holds that `P` is always [`Some`]. Only for the
    /// last element, if [`Punctuated::has_trailing()`] returns false, is it [`None`].
    ///
    /// # Returns
    /// An iterator yielding mutable `(V, Option<P>)`-pairs.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, V, P> {
        let len: usize = self.data.len();
        IterMut { iter: self.data.iter_mut().enumerate(), len, has_trailing: self.has_trailing }
    }



    /// Consumes this Punctuated into an iterator that yields its values by ownership.
    ///
    /// # Returns
    /// An iterator yielding `V` by ownership.
    #[inline]
    pub fn into_values(mut self) -> IntoValues<V, P> {
        // Get the data out
        let mut data = Vec::new();
        std::mem::swap(&mut data, &mut self.data);

        // Yield the iterator with it
        IntoValues { iter: data.into_iter() }
    }

    /// Consumes this Punctuated into an iterator that yields its values by reference.
    ///
    /// # Returns
    /// An iterator yielding `V`.
    #[inline]
    pub fn values(&self) -> Values<'_, V, P> { Values { iter: self.data.iter() } }

    /// Consumes this Punctuated into an iterator that yields its values mutably.
    ///
    /// # Returns
    /// An iterator yielding mutable `V`.
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, V, P> { ValuesMut { iter: self.data.iter_mut() } }



    /// Consumes this Punctuated into an iterator that yields its punctuation by ownership.
    ///
    /// Note that it may yield one element less than the length of the punctuated list. This can
    /// only happen if there was a last element which did not have a trailing punctuation.
    ///
    /// # Returns
    /// An iterator yielding `P` by ownership.
    #[inline]
    pub fn into_puncts(mut self) -> IntoPuncts<V, P> {
        // Get the data out
        let mut data = Vec::new();
        std::mem::swap(&mut data, &mut self.data);

        // Yield the iterator with it
        let len: usize = self.data.len();
        IntoPuncts { iter: data.into_iter().enumerate(), len, has_trailing: self.has_trailing }
    }

    /// Consumes this Punctuated into an iterator that yields its punctuation by reference.
    ///
    /// Note that it may yield one element less than the length of the punctuated list. This can
    /// only happen if there was a last element which did not have a trailing punctuation.
    ///
    /// # Returns
    /// An iterator yielding `P`.
    #[inline]
    pub fn puncts(&self) -> Puncts<'_, V, P> { Puncts { iter: self.data.iter().enumerate(), len: self.data.len(), has_trailing: self.has_trailing } }

    /// Consumes this Punctuated into an iterator that yields its punctuation mutably.
    ///
    /// Note that it may yield one element less than the length of the punctuated list. This can
    /// only happen if there was a last element which did not have a trailing punctuation.
    ///
    /// # Returns
    /// An iterator yielding mutable `P`.
    #[inline]
    pub fn puncts_mut(&mut self) -> PunctsMut<'_, V, P> {
        let len: usize = self.data.len();
        PunctsMut { iter: self.data.iter_mut().enumerate(), len, has_trailing: self.has_trailing }
    }
}
impl<V, P> IntoIterator for Punctuated<V, P> {
    type Item = (V, Option<P>);
    type IntoIter = IntoIter<V, P>;

    #[inline]
    fn into_iter(mut self) -> Self::IntoIter {
        // Get the data out
        let mut data = Vec::new();
        std::mem::swap(&mut data, &mut self.data);

        // Yield the iterator with it
        let data_len: usize = data.len();
        IntoIter { iter: data.into_iter().enumerate(), len: data_len, has_trailing: self.has_trailing }
    }
}
impl<'a, V, P> IntoIterator for &'a Punctuated<V, P> {
    type Item = (&'a V, Option<&'a P>);
    type IntoIter = Iter<'a, V, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}
impl<'a, V, P> IntoIterator for &'a mut Punctuated<V, P> {
    type Item = (&'a mut V, Option<&'a mut P>);
    type IntoIter = IterMut<'a, V, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

// Conversion
impl<V, P: Default, I: IntoIterator<Item = V>> From<I> for Punctuated<V, P> {
    #[inline]
    fn from(value: I) -> Self { Self::from_iter(value) }
}
impl<V, P: Default> FromIterator<V> for Punctuated<V, P> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let mut punct = Punctuated::new();
        punct.extend(iter);
        punct
    }
}
impl<V, P> FromIterator<(V, Option<P>)> for Punctuated<V, P> {
    #[inline]
    #[track_caller]
    fn from_iter<T: IntoIterator<Item = (V, Option<P>)>>(iter: T) -> Self {
        // Conjure up the iterator
        let mut iter = iter.into_iter();
        let mut i: usize = 0;
        if let Some((v, mut p)) = iter.next() {
            // Build the punctuated
            let size_hint: (usize, Option<usize>) = iter.size_hint();
            let mut punct = Punctuated::with_capacity(size_hint.1.unwrap_or(size_hint.0));
            // SAFETY: We know it's the first value, and therefore, safe to push.
            unsafe { punct.push_value_unchecked(v) };
            for (v, newp) in iter {
                i += 1;

                // We know we need a p
                punct.push_punct(p.unwrap_or_else(|| panic!("Missing punctuation before non-last value at position {i}")));

                // Add the value and carry the punctuation
                // SAFETY: We just pushed a punctuation, meaning it's always safe to push the value
                unsafe { punct.push_value_unchecked(v) };
                p = newp;
            }
            // The last one may be optional
            if let Some(p) = p {
                punct.push_punct(p);
            }
            punct
        } else {
            // Nothing to do, so not even necessary to allocate
            Punctuated::new()
        }
    }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_eq_punct {
        // Helping count (we only count values)
        (__oddcount) => {0};
        (__oddcount $v:expr $(, $vn:expr)*) => {
            1 + assert_eq_punct!(__evencount $($vn),*)
        };
        (__evencount) => {0};
        (__evencount $v:expr $(, $vn:expr)*) => {
            assert_eq_punct!(__oddcount $($vn),*)
        };

        // Helper alternating recursion
        {__odd($punct:ident, $i:expr) []} => {};
        {__odd($punct:ident, $i:expr) [$v:expr $(, $vn:expr)*]} => {
            assert_eq!($punct.get($i), Some(&$v));
            assert_eq_punct!(__even($punct, $i) [$($vn),*]);
        };
        {__even($punct:ident, $i:expr) []} => {};
        {__even($punct:ident, $i:expr) [$v:expr $(, $vn:expr)*]} => {
            assert_eq!($punct.get_punct($i), Some(&$v));
            assert_eq_punct!(__odd($punct, $i + 1) [$($vn),*]);
        };

        // Public interface
        ($punct:expr,[$($v:expr),*]) => {
            let __punct = &$punct;
            assert_eq!(__punct.len(), assert_eq_punct!(__oddcount $($v),*));
            assert_eq_punct!{__odd(__punct, 0) [$($v),*]};
        };
    }


    #[test]
    fn test_push_manual() {
        // One value
        let mut punct: Punctuated<&str, char> = Punctuated::new();
        punct.push_value("Hello");
        assert_eq_punct!(punct, ["Hello"]);

        // One value with punct
        let mut punct: Punctuated<&str, char> = Punctuated::new();
        punct.push_value("Hello");
        punct.push_punct(',');
        assert_eq_punct!(punct, ["Hello", ',']);

        // Two values
        let mut punct: Punctuated<&str, char> = Punctuated::new();
        punct.push_value("Hello");
        punct.push_punct(',');
        punct.push_value("world");
        assert_eq_punct!(punct, ["Hello", ',', "world"]);

        // Wrong pushing
        let mut punct: Punctuated<&str, char> = Punctuated::new();
        punct.push_value("Hello");
        assert!(std::panic::catch_unwind(move || punct.push_value("world")).is_err());
    }

    #[test]
    fn test_push_default() {
        // One value
        let mut punct: Punctuated<&str, char> = Punctuated::new();
        punct.push("Hello");
        assert_eq_punct!(punct, ["Hello"]);

        // Two values
        let mut punct: Punctuated<&str, char> = Punctuated::new();
        punct.push("Hello");
        punct.push("world");
        assert_eq_punct!(punct, ["Hello", '\0', "world"]);
    }

    #[test]
    fn test_extend() {
        let mut punct: Punctuated<&str, char> = Punctuated::new();
        punct.extend(["Hello", "world"]);
        assert_eq_punct!(punct, ["Hello", '\0', "world"]);

        punct.extend(["!", "such", "a", "beautiful", "day", "today"]);
        assert_eq_punct!(punct, ["Hello", '\0', "world", '\0', "!", '\0', "such", '\0', "a", '\0', "beautiful", '\0', "day", '\0', "today"]);
    }

    #[test]
    fn test_pop() {
        let mut punct: Punctuated<&str, char> = Punctuated::from(["Hello", "world"]);
        assert_eq_punct!(punct, ["Hello", '\0', "world"]);
        assert_eq!(punct.pop(), Some(("world", None)));
        assert_eq_punct!(punct, ["Hello", '\0']);
        assert_eq!(punct.pop(), Some(("Hello", Some('\0'))));
        assert_eq_punct!(punct, []);
        assert_eq!(punct.pop(), None);
        assert_eq_punct!(punct, []);
    }

    #[test]
    fn test_pop_trailing_punct() {
        let mut punct: Punctuated<&str, char> = Punctuated::from(["Hello", "world"]);
        punct.push_punct(',');
        assert_eq_punct!(punct, ["Hello", '\0', "world", ',']);

        assert_eq!(punct.pop_trailing_punct(), Some(','));
        assert_eq_punct!(punct, ["Hello", '\0', "world"]);
        assert_eq!(punct.pop_trailing_punct(), None);
        assert_eq_punct!(punct, ["Hello", '\0', "world"]);
    }

    #[test]
    fn test_remove() {
        let mut punct: Punctuated<&str, char> = Punctuated::from(["Hello", "world", "!"]);
        assert_eq!(punct.remove(1), ("world", Some('\0')));
        assert_eq_punct!(punct, ["Hello", '\0', "!"]);
        assert_eq!(punct.remove(1), ("!", None));
        assert_eq_punct!(punct, ["Hello", '\0']);
        assert_eq!(punct.remove(0), ("Hello", Some('\0')));
        assert_eq_punct!(punct, []);
        assert!(std::panic::catch_unwind(move || punct.remove(1)).is_err());
    }

    #[test]
    fn test_remove_range() {
        let mut punct: Punctuated<&str, char> = Punctuated::from(["Hello", "world", "!"]);
        punct.remove_range(1..2);
        assert_eq_punct!(punct, ["Hello", '\0', "!"]);
        punct.remove_range(1..2);
        assert_eq_punct!(punct, ["Hello", '\0']);
        punct.remove_range(1..2);
        assert_eq_punct!(punct, ["Hello", '\0']);
        punct.remove_range(0..);
        assert_eq_punct!(punct, []);
        punct.remove_range(0..);
        assert_eq_punct!(punct, []);
    }

    #[test]
    fn test_drain() {
        let mut punct: Punctuated<&str, char> = Punctuated::from(["Hello", "world", "!"]);
        let removed: Vec<(&str, Option<char>)> = punct.drain(1..2).collect();
        assert_eq_punct!(punct, ["Hello", '\0', "!"]);
        assert_eq!(removed, vec![("world", Some('\0'))]);
        let removed: Punctuated<&str, char> = punct.drain(1..2).collect();
        assert_eq_punct!(punct, ["Hello", '\0']);
        assert_eq_punct!(removed, ["!"]);
        let removed: Punctuated<&str, char> = punct.drain(1..2).collect();
        assert_eq_punct!(punct, ["Hello", '\0']);
        assert_eq_punct!(removed, []);
        let removed: Punctuated<&str, char> = punct.drain(0..).collect();
        assert_eq_punct!(punct, []);
        assert_eq_punct!(removed, ["Hello", '\0']);
        let removed: Punctuated<&str, char> = punct.drain(0..).collect();
        assert_eq_punct!(punct, []);
        assert_eq_punct!(removed, []);
    }

    #[test]
    fn test_clone() {
        let punct: Punctuated<&str, char> = Punctuated::new();
        let punct_prime: Punctuated<&str, char> = punct.clone();
        assert_eq!(punct, punct_prime);

        let punct: Punctuated<&str, char> = Punctuated::singleton("Hello");
        let punct_prime: Punctuated<&str, char> = punct.clone();
        assert_eq!(punct, punct_prime);

        let punct: Punctuated<&str, char> = Punctuated::from(["Hello", "world"]);
        let punct_prime: Punctuated<&str, char> = punct.clone();
        assert_eq!(punct, punct_prime);
    }

    #[test]
    fn test_debug() {
        assert_eq!(format!("{:?}", Punctuated::<&str, char>::new()), "[]");
        assert_eq!(format!("{:?}", Punctuated::<&str, char>::from(["Hello"])), "[\"Hello\"]");
        assert_eq!(format!("{:?}", Punctuated::<&str, char>::from(["Hello", "world"])), "[\"Hello\", '\\0', \"world\"]");
        assert_eq!(
            format!("{:?}", {
                let mut punct = Punctuated::<&str, char>::from(["Hello", "world"]);
                punct.push_punct('\0');
                punct
            }),
            "[\"Hello\", '\\0', \"world\", '\\0']"
        );
        assert_eq!(format!("{:#?}", Punctuated::<&str, char>::from(["Hello", "world"])), "[\n    \"Hello\",\n    '\\0',\n    \"world\",\n]");
    }

    #[test]
    fn test_eq() {
        assert_eq!(Punctuated::<&str, char>::new(), Punctuated::<&str, char>::new());
        assert_ne!(Punctuated::<&str, char>::from(["Hello"]), Punctuated::<&str, char>::new());
        assert_eq!(Punctuated::<&str, char>::from(["Hello"]), Punctuated::<&str, char>::from(["Hello"]));
        assert_ne!(Punctuated::<&str, char>::from(["Hello", "world"]), Punctuated::<&str, char>::from(["Hello"]));
        assert_eq!(Punctuated::<&str, char>::from(["Hello", "world"]), Punctuated::<&str, char>::from(["Hello", "world"]));
        assert_ne!(
            Punctuated::<&str, char>::from_iter([("Hello", Some(',')), ("world", None)]),
            Punctuated::<&str, char>::from_iter([("Hello", Some('\0')), ("world", None)])
        );
        assert_ne!(
            {
                let mut punct = Punctuated::<&str, char>::from(["Hello", "world"]);
                punct.push_punct('\0');
                punct
            },
            Punctuated::<&str, char>::from(["Hello", "world"])
        );
        assert_eq!(
            {
                let mut punct = Punctuated::<&str, char>::from(["Hello", "world"]);
                punct.push_punct('\0');
                punct
            },
            {
                let mut punct = Punctuated::<&str, char>::from(["Hello", "world"]);
                punct.push_punct('\0');
                punct
            }
        );
    }

    #[test]
    fn test_ord() {
        assert!(Punctuated::<&str, char>::new() >= Punctuated::<&str, char>::new());
        assert!(Punctuated::<&str, char>::from(["Hello"]) > Punctuated::<&str, char>::new());
        assert!(Punctuated::<&str, char>::from(["Hello"]) <= Punctuated::<&str, char>::from(["Hello"]));
        assert!(Punctuated::<&str, char>::from(["Hello", "world"]) > Punctuated::<&str, char>::from(["Hello"]));
        assert!(Punctuated::<&str, char>::from(["Hello", "world"]) <= Punctuated::<&str, char>::from(["Hello", "world"]));
        assert!(
            Punctuated::<&str, char>::from_iter([("Hello", Some(',')), ("world", None)])
                > Punctuated::<&str, char>::from_iter([("Hello", Some('\0')), ("world", None)])
        );
        assert!(
            {
                let mut punct = Punctuated::<&str, char>::from(["Hello", "world"]);
                punct.push_punct('\0');
                punct
            } > Punctuated::<&str, char>::from(["Hello", "world"])
        );
        assert!(
            {
                let mut punct = Punctuated::<&str, char>::from(["Hello", "world"]);
                punct.push_punct('\0');
                punct
            } >= {
                let mut punct = Punctuated::<&str, char>::from(["Hello", "world"]);
                punct.push_punct('\0');
                punct
            }
        );
    }
}
