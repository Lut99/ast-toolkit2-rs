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
    pub scaffolding: Style,
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

        marker_error: ('-', Style::new()),
        marker_help: ('^', Style::new()),
        marker_suggestion: ('+', Style::new()),
        marker_warn: ('-', Style::new()),

        scaffolding: Style::new(),
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

        marker_error: ('-', Style::new().red().bold()),
        marker_help: ('^', Style::new().cyan().bold()),
        marker_suggestion: ('+', Style::new().green().bold()),
        marker_warn: ('-', Style::new().yellow().bold()),

        scaffolding: Style::new().blue().bold(),
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
                marker_error,
                marker_help,
                marker_suggestion,
                marker_warn,
                scaffolding,
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
            fmt.field("marker_error", marker_error);
            fmt.field("marker_help", marker_help);
            fmt.field("marker_suggestion", marker_suggestion);
            fmt.field("marker_warn", marker_warn);
            fmt.field("scaffolding", scaffolding);
            fmt.finish()
        }
    }
}
