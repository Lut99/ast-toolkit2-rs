//  REPLACE WITH.rs
//    by Lut99
//
//  Description:
//!   Implements a, dare I say it, essential trait when doing visiting.
//

use std::panic::UnwindSafe;


/***** INTERFACES *****/
/// Convenience trait for replacing a mutable reference with another instance.
///
/// It's actually a tricky trait, because while you're editing the original object, there's _two_
/// variables containing the same bits (one in the original reference and one in your closure). If
/// there is resources associated with this object, then dropping any of them will make the other
/// invalid. This is a problem if your closure panics: first, the closure's version of the bits are
/// dropped and then, during unwinding, the bits of the original reference are dropped too,
/// resulting in double frees. As such, this trait provides multiple methods, all to deal with this
/// problem in a different way.
///
/// Heavily inspired/adapted from the [`replace_with`](docs.rs/replace_with)-crate, but modified as
/// a trait object.
pub trait ReplaceWith: Sized {
    /// Replaces the object behind this mutable reference with another object.
    ///
    /// This is the ideal implementation if your type is [`Copy`], where copying the bits of `self`
    /// actually _doesn't_ cause problems with associated memory resources because there aren't any.
    ///
    /// # Arguments
    /// - `replace_with`: An [`FnOnce`] closure that takes an owned version of the object and
    ///   generates a replacement. If it panics, it is handled as usual.
    fn replace_with(&mut self, replace_with: impl FnOnce(Self) -> Self)
    where
        Self: Copy;


    /// Replaces the object behind this mutable reference with another object.
    ///
    /// This is the most ignorant implementation, and it will simply abort the program instead of
    /// unwinding the stack when a panic occurs.
    ///
    /// # Arguments
    /// - `replace_with`: An [`FnOnce`] closure that takes an owned version of the object and
    ///   generates a replacement. If it panics, then the program is aborted immediately and no
    ///   stack is unwound.
    fn replace_with_abort(&mut self, replace_with: impl UnwindSafe + FnOnce(Self) -> Self)
    where
        Self: UnwindSafe;


    /// Replaces the object behind this mutable reference with another object.
    ///
    /// This is a saner implementation, ergnomic implementation if your type is [`Default`]. When
    /// a panic occurs in your closure, the function will replace `self` with its default value and
    /// then return.
    ///
    /// # Arguments
    /// - `replace_with`: An [`FnOnce`] closure that takes an owned version of the object and
    ///   generates a replacement. If it panics, `self` is replaced with its default value and then
    ///   the panic is re-casted.
    fn replace_with_default(&mut self, replace_with: impl UnwindSafe + FnOnce(Self) -> Self)
    where
        Self: Default + UnwindSafe;

    /// Replaces the object behind this mutable reference with another object.
    ///
    /// This is a paranoid version of [`ReplaceWith::replace_with_default()`] where, instead of
    /// waiting until you panic, the value in `self` is immediately replaced with a default. Use
    /// this if your closure does scary things with panic hooks and such.
    ///
    /// # Arguments
    /// - `replace_with`: An [`FnOnce`] closure that takes an owned version of the object and
    ///   generates a replacement. If it panics, it is handled as usual because `self` is already
    ///   replaced with its default value.
    fn replace_with_safe_default(&mut self, replace_with: impl FnOnce(Self) -> Self)
    where
        Self: Default;


    /// Replaces the object behind this mutable reference with another object.
    ///
    /// This is a saner implementation implementation but less ergonomic than
    /// [`ReplaceWith::replace_with_default()] for when your type is _not_ [`Default`]. When a
    /// panic occurs in your closure, the function will replace `self` with the given value and
    /// then return.
    ///
    /// # Arguments
    /// - `replace_with`: An [`FnOnce`] closure that takes an owned version of the object and
    ///   generates a replacement. If it panics, `self` is replaced with `value` and then the panic
    ///   is re-casted.
    fn replace_with_value(&mut self, value: Self, replace_with: impl UnwindSafe + FnOnce(Self) -> Self)
    where
        Self: UnwindSafe;

    /// Replaces the object behind this mutable reference with another object.
    ///
    /// This is a paranoid version of [`ReplaceWith::replace_with_value()`] where, instead of
    /// waiting until you panic, the value in `self` is immediately replaced with `value`. Use
    /// this if your closure does scary things with panic hooks and such.
    ///
    /// # Arguments
    /// - `replace_with`: An [`FnOnce`] closure that takes an owned version of the object and
    ///   generates a replacement. If it panics, it is handled as usual because `self` is already
    ///   replaced with `value`.
    fn replace_with_safe_value(&mut self, value: Self, replace_with: impl FnOnce(Self) -> Self);
}

// Blanket impl
impl<T: Sized> ReplaceWith for T {
    #[inline]
    fn replace_with(&mut self, replace_with: impl FnOnce(Self) -> Self)
    where
        Self: Copy,
    {
        // SAFETY: The pointer is always a valid object because it comes from a reference.
        let this: T = unsafe { std::ptr::read(self as *const T) };
        // NOTE: `self` is perfectly OK, neat!

        // Run the mutation
        let this: T = replace_with(this);

        // OK, now write the value back to `self`
        // SAFETY: This is safe to do because `self` is a reference, always making it have correct
        // alignment etc.
        unsafe { std::ptr::write(self as *mut T, this) };
    }

    #[inline]
    fn replace_with_abort(&mut self, replace_with: impl UnwindSafe + FnOnce(Self) -> Self)
    where
        Self: UnwindSafe,
    {
        // SAFETY: The pointer is always a valid object because it comes from a reference.
        let this: T = unsafe { std::ptr::read(self as *const T) };
        // NOTE: `self` is now no longer in a valid state. Well, it is, but `this` owns all of its
        // resources (if any) and therefore we cannot trust `self` to deallocate itself properly
        // without `this` becoming invalid too. Don't touch `self`!

        // Run the mutation, but with a panic catch
        let this: T = match std::panic::catch_unwind(|| replace_with(this)) {
            Ok(this) => this,
            Err(_) => {
                eprintln!("ERROR: Program panicked while replace a value in an invalid state, cannot unwind!");
                std::process::abort()
            },
        };

        // OK, now write the value back to `self`
        // SAFETY: This is safe to do because `self` is a reference, always making it have correct
        // alignment etc. In addition, it's crucial that `write` doesn't deallocate here! Which,
        // thankfully, it doesn't. Thank the lord for that. (Writing this in case it ever changes
        // behaviour and nobody knows what's going on anymore...)
        unsafe { std::ptr::write(self as *mut T, this) };
    }

    #[inline]
    fn replace_with_default(&mut self, replace_with: impl UnwindSafe + FnOnce(Self) -> Self)
    where
        Self: Default + UnwindSafe,
    {
        // SAFETY: The pointer is always a valid object because it comes from a reference.
        let this: T = unsafe { std::ptr::read(self as *const T) };
        // NOTE: `self` is now no longer in a valid state. Well, it is, but `this` owns all of its
        // resources (if any) and therefore we cannot trust `self` to deallocate itself properly
        // without `this` becoming invalid too. Don't touch `self`!

        // Run the mutation, but with a panic catch
        let this: T = match std::panic::catch_unwind(|| replace_with(this)) {
            Ok(this) => this,
            Err(err) => {
                // Replace self and then resume as normal
                *self = Default::default();
                std::panic::resume_unwind(err)
            },
        };

        // OK, now write the value back to `self`
        // SAFETY: This is safe to do because `self` is a reference, always making it have correct
        // alignment etc. In addition, it's crucial that `write` doesn't deallocate here! Which,
        // thankfully, it doesn't. Thank the lord for that. (Writing this in case it ever changes
        // behaviour and nobody knows what's going on anymore...)
        unsafe { std::ptr::write(self as *mut T, this) };
    }

    #[inline]
    fn replace_with_safe_default(&mut self, replace_with: impl FnOnce(Self) -> Self)
    where
        Self: Default,
    {
        // Instead of ready and/or writing, we just swap safely.
        let mut default = Default::default();
        std::mem::swap(self, &mut default);

        // Run the mutation, let it panic at will, see if I care
        default = replace_with(default);

        // OK, now write the value back to `self` and drop the original `default`
        std::mem::swap(self, &mut default);
    }


    #[inline]
    fn replace_with_value(&mut self, value: Self, replace_with: impl UnwindSafe + FnOnce(Self) -> Self)
    where
        Self: UnwindSafe,
    {
        // SAFETY: The pointer is always a valid object because it comes from a reference.
        let this: T = unsafe { std::ptr::read(self as *const T) };
        // NOTE: `self` is now no longer in a valid state. Well, it is, but `this` owns all of its
        // resources (if any) and therefore we cannot trust `self` to deallocate itself properly
        // without `this` becoming invalid too. Don't touch `self`!

        // Run the mutation, but with a panic catch
        let this: T = match std::panic::catch_unwind(|| replace_with(this)) {
            Ok(this) => this,
            Err(err) => {
                // Replace self and then resume as normal
                *self = value;
                std::panic::resume_unwind(err)
            },
        };

        // OK, now write the value back to `self`
        // SAFETY: This is safe to do because `self` is a reference, always making it have correct
        // alignment etc. In addition, it's crucial that `write` doesn't deallocate here! Which,
        // thankfully, it doesn't. Thank the lord for that. (Writing this in case it ever changes
        // behaviour and nobody knows what's going on anymore...)
        unsafe { std::ptr::write(self as *mut T, this) };
    }

    #[inline]
    fn replace_with_safe_value(&mut self, mut value: Self, replace_with: impl FnOnce(Self) -> Self) {
        // Instead of ready and/or writing, we just swap safely.
        std::mem::swap(self, &mut value);

        // Run the mutation, let it panic at will, see if I care
        value = replace_with(value);

        // OK, now write the value back to `self` and drop the original `value`
        std::mem::swap(self, &mut value);
    }
}
