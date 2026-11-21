//  COMMON.rs
//    by Lut99
//
//  Description:
//!   Common code for the three formatter impls.
//

use std::fmt;

use console::Style;
use unicode_segmentation::UnicodeSegmentation as _;

use super::super::super::annots::Severity;
use super::super::super::themes::Theme;


/***** LIBRARY *****/
/// Function that writes the given string to `f` linewrapped.
///
/// Note that this function completely re-writes all whitespace in the string to spaces.
pub fn write_linewrapped(f: &mut fmt::Formatter<'_>, max_line_width: usize, mut col: usize, style: &Style, s: &str) -> fmt::Result {
    let mut first: bool = true;
    for mut word in s.split_whitespace() {
        // If the word would cause the line to overflow, write a newline first
        let mut space_len: usize = if !first && col > 0 { 1 } else { 0 };
        let mut word_len: usize = word.graphemes(true).count();
        if col > 0 && col + space_len + word_len > max_line_width {
            writeln!(f)?;
            col = 0;
            space_len = 0;
        }

        // If the word is too long, write chunks of it as newlines first
        while word_len > max_line_width {
            let i: usize = word.grapheme_indices(true).map(|(i, _)| i).take(max_line_width + 1).last().unwrap();
            writeln!(f, "{}", style.apply_to(&word[..i]))?;
            word = &word[i..];
            word_len -= max_line_width;
        }

        // Then write the word
        for _ in 0..space_len {
            write!(f, " ")?;
        }
        write!(f, "{}", style.apply_to(word))?;
        col += space_len + word_len;
        first = false;
    }
    Ok(())
}



/// Function writing the standard message header.
pub fn write_header(f: &mut fmt::Formatter<'_>, theme: &Theme, max_line_width: usize, is_main: bool, sev: Severity, msg: &str) -> fmt::Result {
    let sev_style = theme.header_severity_style(sev);
    let msg_style = &theme.header_extra_msg;

    // Write the initial severity
    let ssev: &str = sev.as_str();
    write!(f, "{}{}", sev_style.apply_to(ssev), msg_style.apply_to(": "))?;

    // Write the message linewrapped
    write_linewrapped(f, max_line_width, ssev.len() + 2, if is_main { &theme.header_main_msg } else { &theme.header_extra_msg }, msg)?;
    writeln!(f)
}



/// Function writing the standard loc string.
pub fn write_loc(f: &mut fmt::Formatter<'_>, theme: &Theme, max_line_no_width: usize, loc: &str) -> fmt::Result {
    let scaffolding_style = &theme.scaffolding;
    let loc_style = &theme.header_loc;

    // Write the string
    writeln!(f, "{:>max_line_no_width$}{} {}", "", scaffolding_style.apply_to("-->"), loc_style.apply_to(loc))
    // writeln!(f, "{:>line_maxwidth$} {}", "", scaffolding_style.apply_to("|"))?;
}



/// Function writing that first empty line of a snippet.
pub fn write_empty_source_line(f: &mut fmt::Formatter<'_>, theme: &Theme, max_line_no_width: usize) -> fmt::Result {
    let scaffolding_style = &theme.scaffolding;
    writeln!(f, "{:>max_line_no_width$} {}", "", scaffolding_style.apply_to("|"))
}



/// Function writing the start of a source line.
pub fn write_source_line_start(f: &mut fmt::Formatter<'_>, theme: &Theme, max_line_no_width: usize, scaffolding_char: char, l: &str) -> fmt::Result {
    let scaffolding_style = &theme.scaffolding;
    write!(f, "{:>max_line_no_width$} {} ", l, scaffolding_style.apply_to(scaffolding_char))
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;

    struct Display<'s>(usize, usize, &'s str);
    impl<'s> fmt::Display for Display<'s> {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write_linewrapped(f, self.0, self.1, &Style::new(), self.2) }
    }

    #[test]
    fn test_write_linewrapped() {
        let s = "Did you ever hear the tragedy of Darth        Plagueis the wise? I thought not. It's not a story the Jedi would tell you. It's a \
                 Sith legend. Darthplagueisthewisewassowise,sopowerfulthathecouldeven";

        assert_eq!(
            format!("{}", Display(20, 0, &s)),
            "Did you ever hear\nthe tragedy of Darth\nPlagueis the wise? I\nthought not. It's\nnot a story the Jedi\nwould tell you. It's\na Sith \
             legend.\nDarthplagueisthewise\nwassowise,sopowerful\nthathecouldeven"
        );
        assert_eq!(
            format!("{}", Display(20, 5, &s)),
            "Did you ever\nhear the tragedy of\nDarth Plagueis the\nwise? I thought not.\nIt's not a story the\nJedi would tell you.\nIt's a Sith \
             legend.\nDarthplagueisthewise\nwassowise,sopowerful\nthathecouldeven"
        );
    }
}
