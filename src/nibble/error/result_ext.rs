//  RESULT EXT.rs
//    by Lut99
//
//  Description:
//!   Implements convenience methods on [`Result`].
//

use super::NibbleError;


/***** LIBRARY *****/
/// Trait for conveniently calling [`NibbleError`]'s map functions through a [`Result`].
pub trait ResultExt<T, F, E> {
    /// Transforms any [`NibbleError::Unmatched()`] into an [`Option`] for convenience.
    ///
    /// # Returns
    /// The same but now the result is [`Some`] if it wasn't an error, or [`None`] if
    /// [`NibbleError::Unmatched`] or [`NibbleError::NotEnough`] was the error. It still returns
    /// [`Err`] if the error was [`NibbleError::Error`] (but now directly with the error).
    fn transpose(self) -> Result<Option<T>, E>;



    /// Allows one to call [`NibbleError::into_atleast()`] through a [`Result`].
    ///
    /// # Returns
    /// A new instance of `Self` with any internal [`Needed::Bounded`] mapped to a
    /// [`Needed::AtLeast`].
    fn into_atleast(self) -> Result<T, NibbleError<F, E>>;

    /// Allows one to call [`NibbleError::auto_map()`] through a [`Result`].
    ///
    /// # Returns
    /// An equivalent error but with a mapped `F`.
    fn auto_map<F2: From<F>, E2: From<E>>(self) -> Result<T, NibbleError<F2, E2>>;

    /// Allows one to call [`NibbleError::map_fmt()`] through a [`Result`].
    ///
    /// # Arguments
    /// - `map`: Some [`FnOnce`] doing the mapping.
    ///
    /// # Returns
    /// An equivalent error but with a mapped `F`.
    fn map_fmt<F2, E2: From<E>>(self, map: impl FnOnce(F) -> F2) -> Result<T, NibbleError<F2, E2>>;

    /// Allows one to call [`NibbleError::map_err()`] through a [`Result`].
    ///
    /// # Arguments
    /// - `map`: Some [`FnOnce`] doing the mapping.
    ///
    /// # Returns
    /// An equivalent error but with a mapped `E`.
    fn map_nerr<F2: From<F>, E2>(self, map: impl FnOnce(E) -> E2) -> Result<T, NibbleError<F2, E2>>;
}
impl<T, F, E> ResultExt<T, F, E> for Result<T, NibbleError<F, E>> {
    #[inline]
    fn transpose(self) -> Result<Option<T>, E> {
        match self {
            Ok(res) => Ok(Some(res)),
            Err(NibbleError::Unmatched(_, _)) => Ok(None),
            Err(NibbleError::NotEnough(_, _, _)) => Ok(None),
            Err(NibbleError::Error(err)) => Err(err),
        }
    }



    #[inline]
    fn into_atleast(self) -> Result<T, NibbleError<F, E>> {
        match self {
            Ok(res) => Ok(res),
            Err(err) => Err(err.into_atleast()),
        }
    }
    #[inline]
    fn auto_map<F2: From<F>, E2: From<E>>(self) -> Result<T, NibbleError<F2, E2>> {
        match self {
            Ok(res) => Ok(res),
            Err(err) => Err(err.auto_map()),
        }
    }

    #[inline]
    fn map_fmt<F2, E2: From<E>>(self, map: impl FnOnce(F) -> F2) -> Result<T, NibbleError<F2, E2>> {
        match self {
            Ok(res) => Ok(res),
            Err(err) => Err(err.map_fmt(map)),
        }
    }

    #[inline]
    fn map_nerr<F2: From<F>, E2>(self, map: impl FnOnce(E) -> E2) -> Result<T, NibbleError<F2, E2>> {
        match self {
            Ok(res) => Ok(res),
            Err(err) => Err(err.map_nerr(map)),
        }
    }
}
