//  STEP 4 - RENDERING.rs
//    by Lut99
//
//  Description:
//!   Defines the fourth step in the [layouting algorithm](super).
//

use std::fmt::{Display, Formatter, Result as FResult};
use std::marker::PhantomData;

use super::super::themes::Theme;
use super::layouter::Layouter;
use super::s3_place::AnnotatedMessage;


/***** HELPERS *****/
struct RenderLoc<'l, L, E>(u64, u64, &'l L, PhantomData<E>);
impl<'l, L: Layouter<E>, E> Display for RenderLoc<'l, L, E> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { self.2.render_loc(f, self.0, self.1) }
}

struct RenderReplace<'l, L, E>(u64, &'l L, PhantomData<E>);
impl<'l, L: Layouter<E>, E> Display for RenderReplace<'l, L, E> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { self.1.render_replace(f, self.0) }
}





/***** LIBRARY *****/
/// Runs the third step in the [layouting algorithm](super).
///
/// # Arguments
/// - `f`: A [`Formatter`] that we can write to.
/// - `theme`: The [`Theme`] deciding how to render markers and which colours belong to which severity.
/// - `layouter`: A [`Layouter`] which determines the maximum line length.
/// - `msgs`: A list of message to compute annotation placement in.
///
/// # Returns
/// A new list of messages with rendered annotations attached.
pub fn s4_render<'t, 'd, E>(f: &mut Formatter<'_>, theme: &'t Theme, layouter: &impl Layouter<E>, msgs: Vec<AnnotatedMessage<'t, 'd, E>>) -> FResult {
    // Show everything in-order!
    for msg in msgs {
        match msg {
            // <severity>(<code>): <msg>
            //   --> <file>:<line>:<col>
            //        |
            // <line> | <snippet>
            //        |
            // ...
            //        |
            // <line> | <snippet>
            //        |
            //        = note: <note>
            AnnotatedMessage::Main(m) => continue,

            // <severity>: <msg>
            //   --> <file>:<line>:<col>
            //        |
            // <line> - <snippet>
            // <line> + <replace>
            //        |
            AnnotatedMessage::Replace(m) => {
                let sev_style = theme.header_severity_style(m.sev);
                let msg_style = &theme.header_extra_msg;
                let scaffolding_style = &theme.scaffolding;
                let loc_style = &theme.header_loc;

                let largest_line = m.lines.iter().map(|l| l.line + 1).max().unwrap_or(0);
                let line_maxwidth = 1 + largest_line.ilog10() as usize;

                let source = RenderReplace(m.source_id, layouter, PhantomData::<E>).to_string();

                writeln!(f, "{}{}", sev_style.apply_to(m.sev.as_str()), msg_style.apply_to(format!(": {}", m.msg)))?;
                writeln!(
                    f,
                    "{:>line_maxwidth$}{} {}",
                    "",
                    scaffolding_style.apply_to("-->"),
                    loc_style.apply_to(RenderLoc(m.source_id, m.pos, layouter, PhantomData::<E>))
                )?;
                writeln!(f, "{:>line_maxwidth$} {}", "", scaffolding_style.apply_to("|"))?;
                for (i, line) in source.lines().enumerate() {
                    let l = ();
                    writeln!(f, "{:>line_maxwidth$} {}", "", scaffolding_style.apply_to("|"))?;
                }
                writeln!(f, "{:>line_maxwidth$} {}", "", scaffolding_style.apply_to("|"))?;
            },

            // <severity>: <msg>
            AnnotatedMessage::Sourceless(m) => {
                let sev_style = theme.header_severity_style(m.sev);
                let msg_style = &theme.header_extra_msg;

                writeln!(f, "{}{}", sev_style.apply_to(m.sev.as_str()), msg_style.apply_to(format!(": {}", m.msg)))?;
                writeln!(f)?;
            },
        }
    }

    // Done
    Ok(())
}
