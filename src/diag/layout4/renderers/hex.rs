//  HEX RENDERER.rs
//    by Lut99
//
//  Description:
//!   Provides the builtin [`HexRenderer`].
//

use std::fmt;
use std::hash::BuildHasher;

use super::super::{Renderer, sourceless};
use crate::sources::Sources;


/***** LIBRARY *****/
/// A renderer for showing binary source code in a hexgrid.
pub struct HexRenderer<'s, 'r, R> {
    max_line_width: usize,
    sources: &'s Sources<'r, R>,
}

// Constructors
impl<'s, 'r, R> HexRenderer<'s, 'r, R> {
    /// Constructor for the HexRenderer.
    ///
    /// # Arguments
    /// - `max_line_width`: The maximum line width to render.
    /// - `sources`: A [`Sources]` able to map source IDs to actual texts.
    #[inline(always)]
    pub const fn new(max_line_width: usize, sources: &'s Sources<'r, R>) -> Self { Self { max_line_width, sources } }
}

// Renderer
impl<'s, 'r, R: BuildHasher> Renderer<u8> for HexRenderer<'s, 'r, R> {
    fn render_main(&self, f: (), msg: ()) -> fmt::Result { todo!() }

    fn render_replace(&self, f: (), msg: ()) -> fmt::Result { todo!() }

    fn render_sourceless(&self, mut f: sourceless::Formatter<'_, '_>, msg: sourceless::Message<'_>) -> fmt::Result {
        f.set_max_line_width(self.max_line_width);
        f.write_sourceless(msg)
    }
}
