//  STEP 4 - RENDERING.rs
//    by Lut99
//
//  Description:
//!   Defines the fourth step in the [layouting algorithm](super).
//

use std::borrow::Cow;
use std::fmt::{self, Display, Formatter, Result as FResult, Write as _};
use std::marker::PhantomData;

use console::Style;

use super::super::annots::Severity;
use super::super::themes::{SourceTheme, Theme};
use super::layouter::Layouter;
use super::s3_place::{AnnotatedMessage, AnnotatedSnippet, Draw};


/***** HELPERS *****/
struct Fmt<F>(F);
impl<F: for<'a, 'b> Fn(&'a mut Formatter<'b>) -> FResult> Display for Fmt<F> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { (self.0)(f) }
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
pub fn s4_render<'d, E>(f: &mut fmt::Formatter<'_>, theme: &'d Theme, layouter: &impl Layouter<E>, msgs: Vec<AnnotatedMessage<'d, E>>) -> FResult {
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
            AnnotatedMessage::Main(m) => {
                let sev_style = theme.header_severity_style(m.sev);
                let msg_style = &theme.header_extra_msg;
                let scaffolding_style = &theme.scaffolding;
                let loc_style = &theme.header_loc;

                writeln!(
                    f,
                    "{}{}{}",
                    sev_style.apply_to(m.sev.as_str()),
                    if let Some(code) = m.code { format!("[{}]", theme.header_code.apply_to(code)) } else { String::new() },
                    msg_style.apply_to(format!(": {}", m.msg))
                )?;
                for snippet in m.snippets {
                    let largest_line = snippet.lines.iter().map(|l| l.line + 1).max().unwrap_or(0);
                    // let line_maxwidth = 1 + largest_line.ilog10() as usize;
                    let line_maxwidth = layouter.line_no_width(largest_line);

                    writeln!(
                        f,
                        "{:>line_maxwidth$}{} {}",
                        "",
                        scaffolding_style.apply_to("-->"),
                        loc_style.apply_to(Fmt(|f: &mut Formatter<'_>| layouter.render_loc(f, snippet.source_id, snippet.pos)))
                    )?;
                    writeln!(f, "{:>line_maxwidth$} {}", "", scaffolding_style.apply_to("|"))?;
                    for line in &snippet.lines {
                        // Write the source line
                        writeln!(
                            f,
                            "{:>line_maxwidth$} {} {}",
                            Fmt(|f: &mut Formatter<'_>| layouter.render_line_no(f, &theme.header_line_number, line.line)),
                            scaffolding_style.apply_to("|"),
                            Fmt(|f: &mut Formatter<'_>| layouter.render_line(
                                f,
                                theme.replace_theme(false),
                                snippet.source_id,
                                line.line,
                                line.annots.iter().map(|a| a.range)
                            ))
                        )?;
                        // Write the rendered annotation lines
                        for line in &line.rendered {
                            write!(f, "{:>line_maxwidth$} {} ", "", scaffolding_style.apply_to("|"))?;
                            for draw in &line.draw {
                                match draw {
                                    Draw::Empty(n) => write!(f, "{:>n$}", "")?,
                                    Draw::Marker(marker, style, n) => {
                                        for _ in 0..*n {
                                            write!(f, "{}", style.apply_to(marker))?
                                        }
                                    },
                                    Draw::Text(text, style, _) => write!(f, "{}", style.apply_to(text))?,
                                }
                            }
                            writeln!(f)?;
                        }
                    }
                    writeln!(f, "{:>line_maxwidth$} {}", "", scaffolding_style.apply_to("|"))?;
                }
            },

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
                // let line_maxwidth = 1 + largest_line.ilog10() as usize;
                let line_maxwidth = layouter.line_no_width(largest_line);

                writeln!(f, "{}{}", sev_style.apply_to(m.sev.as_str()), msg_style.apply_to(format!(": {}", m.msg)))?;
                writeln!(
                    f,
                    "{:>line_maxwidth$}{} {}",
                    "",
                    scaffolding_style.apply_to("-->"),
                    loc_style.apply_to(Fmt(|f: &mut Formatter<'_>| layouter.render_loc(f, m.source_id, m.pos)))
                )?;
                writeln!(f, "{:>line_maxwidth$} {}", "", scaffolding_style.apply_to("|"))?;
                for line in &m.lines {
                    // Write the source line
                    writeln!(
                        f,
                        "{:>line_maxwidth$} {} {}",
                        Fmt(|f: &mut Formatter<'_>| layouter.render_line_no(f, &theme.header_line_number, line.line)),
                        scaffolding_style.apply_to("-"),
                        Fmt(|f: &mut Formatter<'_>| layouter.render_line(f, theme.replace_theme(false), m.source_id, line.line, [line.range]))
                    )?;
                }
                for line in m.lines {
                    // Write the replacing line
                    // TODO render to a buffer and then with proper line starts
                    let mut buf = String::new();
                    writeln!(
                        &mut buf,
                        "{}",
                        Fmt(|f: &mut Formatter<'_>| layouter.render_line_replaced(
                            f,
                            theme.replace_theme(true),
                            m.source_id,
                            line.line,
                            line.range,
                            m.repl
                        ))
                    )
                    .unwrap();

                    // Now write the lines in that line
                    let mut l: u64 = line.line;
                    let mut lines = buf.lines().peekable();
                    while let Some(line) = lines.next() {
                        if lines.peek().is_none() && line.trim().is_empty() {
                            continue;
                        }
                        writeln!(
                            f,
                            "{:>line_maxwidth$} {} {}",
                            Fmt(|f: &mut Formatter<'_>| layouter.render_line_no(f, &theme.header_line_number, l)),
                            scaffolding_style.apply_to("+"),
                            line
                        )?;
                        l += 1;
                    }
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
