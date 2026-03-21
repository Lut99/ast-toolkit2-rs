//  SPAN.rs
//    by Lut99
//
//  Description:
//!   Does [`Spanning`]-impls for the [`Punctuated`] list.
//

use super::Punctuated;
use crate::loc::{Loc, Located};


/***** IMPLEMENTATIONS *****/
impl<'s, V: Located, P: Located> Located for Punctuated<V, P> {
    #[inline]
    fn loc(&self) -> Loc {
        let mut res: Option<Loc> = None;
        for (value, punct) in self {
            // The value is always loc'd first
            let value_loc: Loc = value.loc();
            if let Some(res) = &mut res {
                res.extend(value_loc);
            } else {
                res = Some(value_loc);
            }
            // Then, if there is punct, we add it
            if let Some(punct) = punct {
                // SAFETY: We know that at least one `loc` exists because of
                // the value code.
                res.as_mut().unwrap().extend(punct.loc());
            }
        }
        res.unwrap_or_else(Loc::new)
    }
}
