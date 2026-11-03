//  HEXADECIMAL LAYOUTER.rs
//    by Lut99
//
//  Description:
//!   Defines a layouter that lays out bytes as hexadecimal.
//

use super::super::{Layouter, Range2D};
use crate::loc::Range;


/***** LIBRARY *****/
/// A layouter that lays out bytes as a hexviewer grid.
pub struct HexLayouter;

// Layouter
impl Layouter<u8> for HexLayouter {
    fn layout(&self, _source_id: Option<u64>, ranges: Vec<Range>) -> Vec<Range2D> {
        // Example row - 00 01 02 03  04 05 06 07  08 09 0A 0B  0C 0D 0E 0F    ..Hello,world!.-

        let mut res = Vec::with_capacity(ranges.len());
        for range in ranges {}
        res
    }

    fn render(&self) {}
}




/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_layouter_layout() {}
}
