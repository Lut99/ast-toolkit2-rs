//  THEMES.rs
//    by Lut99
//
//  Description:
//!   Defines the base [`Theme`] trait, as well as a few default themes coming
//!   with the library.
//

use std::fmt::{Debug, Formatter, Result as FResult};

use console::Style;

use crate::diag::Severity;


/***** LIBRARY *****/
/// Defines a theme for customizing layouting of source snippets with their annotations.
///
/// To build your own, simply create this struct yourself or start with a provided theme (one of
/// the constants attached to this type) and go from there.
#[derive(Clone, Eq, PartialEq)]
pub struct Theme {
    // Header
    /// Style for [`Severity::Error`](Severity::Error) in the header.
    pub header_error: Style,
    /// Style for [`Severity::Help`](Severity::Help) in the header.
    pub header_help: Style,
    /// Style for [`Severity::Suggestion`](Severity::Suggestion) in the header.
    pub header_suggestion: Style,
    /// Style for [`Severity::Warning`](Severity::Warning) in the header.
    pub header_warn: Style,
    /// Style for a code in the header, if any.
    pub header_code: Style,
    /// The message of the main message's header's style.
    pub header_main_msg: Style,
    /// The message of any other message's header's style.
    pub header_extra_msg: Style,
    /// The style of the source reference in the header.
    pub header_loc: Style,
    /// The style of the line nubmer.
    pub header_line_number: Style,

    // Source
    /// The style of the basic source text.
    pub source_base: Style,
    /// The style of [`Severity::Error`](Severity::Error)-highlighted parts in the source text.
    pub source_error: Style,
    /// The style of [`Severity::Help`](Severity::Help)-highlighted parts in the source text.
    pub source_help: Style,
    /// The style of [`Severity::Suggestion`](Severity::Suggestion)-highlighted parts in the source text.
    pub source_suggestion: Style,
    /// The style of [`Severity::Warning`](Severity::Warning)-highlighted parts in the source text.
    pub source_warn: Style,
    /// The style of something that might be replaced.
    pub source_replace_old: Style,
    /// The style of something that replaces something else.
    pub source_replace_new: Style,

    // Markers
    /// Marker style for [`Severity::Error`](Severity::Error).
    pub marker_error: (char, Style),
    /// Marker style for [`Severity::Help`](Severity::Help).
    pub marker_help: (char, Style),
    /// Marker style for [`Severity::Suggestion`](Severity::Suggestion).
    pub marker_suggestion: (char, Style),
    /// Marker style for [`Severity::Warning`](Severity::Warning).
    pub marker_warn: (char, Style),

    // Misc
    /// The style of punctuation used to visually make up the message.
    pub scaffolding:     Style,
    /// The style of the `note:` in the footer, if any.
    pub footer_note:     Style,
    /// The style of any message attached to the `note`.
    pub footer_note_msg: Style,
}

// Provided themes
impl Theme {
    /// The plain theme does not color anything, and is hence the most basic theme you can find.
    pub const PLAIN: Self = Self {
        header_error: Style::new(),
        header_help: Style::new(),
        header_suggestion: Style::new(),
        header_warn: Style::new(),
        header_code: Style::new(),
        header_main_msg: Style::new(),
        header_extra_msg: Style::new(),
        header_loc: Style::new(),
        header_line_number: Style::new(),

        source_base: Style::new(),
        source_error: Style::new(),
        source_help: Style::new(),
        source_suggestion: Style::new(),
        source_warn: Style::new(),
        source_replace_old: Style::new(),
        source_replace_new: Style::new(),

        marker_error: ('-', Style::new()),
        marker_help: ('^', Style::new()),
        marker_suggestion: ('+', Style::new()),
        marker_warn: ('-', Style::new()),

        scaffolding:     Style::new(),
        footer_note:     Style::new(),
        footer_note_msg: Style::new(),
    };

    /// Simulates colors as used by the Rust compiler.
    pub const RUST: Self = Self {
        header_error: Style::new().red().bold(),
        header_help: Style::new().cyan().bold(),
        header_suggestion: Style::new().green().bold(),
        header_warn: Style::new().yellow().bold(),
        header_code: Style::new().bold(),
        header_main_msg: Style::new().bold(),
        header_extra_msg: Style::new(),
        header_loc: Style::new(),
        header_line_number: Style::new().blue().bold().bright(),

        source_base: Style::new(),
        source_error: Style::new(),
        source_help: Style::new(),
        source_suggestion: Style::new(),
        source_warn: Style::new(),
        source_replace_old: Style::new().red().bold(),
        source_replace_new: Style::new().green().bold(),

        marker_error: ('-', Style::new().red().bold()),
        marker_help: ('^', Style::new().cyan().bold()),
        marker_suggestion: ('+', Style::new().green().bold()),
        marker_warn: ('-', Style::new().yellow().bold()),

        scaffolding:     Style::new().blue().bold().bright(),
        footer_note:     Style::new().bold(),
        footer_note_msg: Style::new(),
    };
}

// Helpers
impl Theme {
    /// Returns the marker style for the given severity level.
    #[inline]
    pub const fn header_severity_style(&self, sev: Severity) -> &Style {
        match sev {
            Severity::Error => &self.header_error,
            Severity::Help => &self.header_help,
            Severity::Suggestion => &self.header_suggestion,
            Severity::Warning => &self.header_warn,
        }
    }

    /// Returns the marker style for the given severity level.
    #[inline]
    pub const fn source_highlight_style(&self, sev: Severity) -> &Style {
        match sev {
            Severity::Error => &self.source_error,
            Severity::Help => &self.source_help,
            Severity::Suggestion => &self.source_suggestion,
            Severity::Warning => &self.source_warn,
        }
    }

    /// Returns the marker style for the given severity level.
    #[inline]
    pub const fn marker_style(&self, sev: Severity) -> &(char, Style) {
        match sev {
            Severity::Error => &self.marker_error,
            Severity::Help => &self.marker_help,
            Severity::Suggestion => &self.marker_suggestion,
            Severity::Warning => &self.marker_warn,
        }
    }
}

// Ops
impl Debug for Theme {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        if self == &Self::PLAIN {
            write!(f, "Theme::PLAIN")
        } else if self == &Self::RUST {
            write!(f, "Theme::RUST")
        } else {
            let Self {
                header_error,
                header_help,
                header_suggestion,
                header_warn,
                header_code,
                header_main_msg,
                header_extra_msg,
                header_loc,
                header_line_number,
                source_base,
                source_error,
                source_help,
                source_suggestion,
                source_warn,
                source_replace_old,
                source_replace_new,
                marker_error,
                marker_help,
                marker_suggestion,
                marker_warn,
                scaffolding,
                footer_note,
                footer_note_msg,
            } = self;
            let mut fmt = f.debug_struct("Theme::CUSTOM");
            fmt.field("header_error", header_error);
            fmt.field("header_help", header_help);
            fmt.field("header_suggestion", header_suggestion);
            fmt.field("header_warn", header_warn);
            fmt.field("header_code", header_code);
            fmt.field("header_main_msg", header_main_msg);
            fmt.field("header_extra_msg", header_extra_msg);
            fmt.field("header_loc", header_loc);
            fmt.field("header_line_number", header_line_number);
            fmt.field("source_base", source_base);
            fmt.field("source_error", source_error);
            fmt.field("source_help", source_help);
            fmt.field("source_suggestion", source_suggestion);
            fmt.field("source_warn", source_warn);
            fmt.field("source_replace_old", source_replace_old);
            fmt.field("source_replace_new", source_replace_new);
            fmt.field("marker_error", marker_error);
            fmt.field("marker_help", marker_help);
            fmt.field("marker_suggestion", marker_suggestion);
            fmt.field("marker_warn", marker_warn);
            fmt.field("scaffolding", scaffolding);
            fmt.field("footer_note", footer_note);
            fmt.field("footer_note_msg", footer_note_msg);
            fmt.finish()
        }
    }
}
