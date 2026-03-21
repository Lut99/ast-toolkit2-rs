//  SERDE.rs
//    by Lut99
//
//  Description:
//!   Implements [`Deserialize`] and [`Serialize`] on [`Punctuated`] in order
//!   to make it (de)serializable.
//

use std::fmt::{Formatter, Result as FResult};
use std::marker::PhantomData;

use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::ser::{SerializeSeq as _, Serializer};
use serde::{Deserialize, Serialize};

use super::Punctuated;


/***** HELPERS *****/
/// [`Visitor`] for [`Punctuated`]'s [`Deserialize`] impl.
struct PunctuatedVisitor<V, P> {
    _v: PhantomData<V>,
    _p: PhantomData<P>,
}
impl<'de, V: Deserialize<'de>, P: Deserialize<'de>> Visitor<'de> for PunctuatedVisitor<V, P> {
    type Value = Punctuated<V, P>;

    #[inline]
    fn expecting(&self, f: &mut Formatter) -> FResult { write!(f, "a punctuated list with optional trailing punctuation") }

    #[inline]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let size_hint: Option<usize> = seq.size_hint();

        // Get the first value
        let mut res: Punctuated<V, P> = Punctuated::with_capacity(size_hint.map(|s| (1 + s) / 2).unwrap_or(0));
        while let Some(v) = seq.next_element::<V>()? {
            // Store the value
            // SAFETY: We can always push because it's either the first, or we pushed a punctuation
            unsafe { res.push_value_unchecked(v) };

            // Now parse a punctuation
            if let Some(p) = seq.next_element::<P>()? {
                res.push_punct(p);
            } else {
                break;
            }
        }
        Ok(res)
    }
}





/***** IMPLS *****/
impl<'de, V: Deserialize<'de>, P: Deserialize<'de>> Deserialize<'de> for Punctuated<V, P> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(PunctuatedVisitor { _v: PhantomData, _p: PhantomData })
    }
}
impl<V: Serialize, P: Serialize> Serialize for Punctuated<V, P> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data_len: usize = self.data.len();
        let size_hint: usize = (2 * data_len).saturating_sub(if self.has_trailing { 0 } else { 1 });
        let mut ser = serializer.serialize_seq(Some(size_hint))?;
        for (i, (v, p)) in self.data.iter().enumerate() {
            // Always serialize the value
            ser.serialize_element(v)?;
            // Then serialize the punctuation if it's there
            // SAFETY: This will never underflow because we won't enter the body's loop if there
            // are no elements.
            if i < data_len - 1 || self.has_trailing {
                // SAFETY: `p` is initialized because we asserted it is either a non-last element,
                // in which case it always exists, or there is a trailing punctuation.
                ser.serialize_element(unsafe { p.assume_init_ref() })?;
            }
        }
        ser.end()
    }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_serde {
        ($punct:expr, $raw:literal) => {{
            // First, serialize
            let punct: Punctuated<&str, char> = $punct;
            let text: String = serde_json::to_string(&punct).unwrap();
            assert_eq!(text, $raw);

            // Then de-serialize again
            let punct_prime: Punctuated<&str, char> = serde_json::from_str(&text).unwrap();
            assert_eq!(punct, punct_prime);
        }};
    }

    #[test]
    fn test_serde_nontrailing() {
        assert_serde!(Punctuated::new(), "[]");
        assert_serde!(Punctuated::from(["Hello"]), "[\"Hello\"]");
        assert_serde!(Punctuated::from_iter([("Hello", Some(',')), ("world", None)]), "[\"Hello\",\",\",\"world\"]");
        assert_serde!(Punctuated::from_iter([("Hello", Some(',')), ("world", Some(','))]), "[\"Hello\",\",\",\"world\",\",\"]");
    }
}
