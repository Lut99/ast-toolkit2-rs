//  FORMATTER.rs
//    by Lut99
//
//  Description:
//!   Defines a custom [`Formatter`] that's more elaborate than the standard one.
//

use std::fmt::{Result as FResult, Write};

use super::Coloring;
#[cfg(feature = "color")]
use super::style::Style;


/***** CONSTANTS *****/
/// The number of spaces that make an indentation.
pub const INDENT: &'static str = "    ";





/***** LIBRARY *****/
pub struct Formatter<'w> {
    /// The writer we're a-writin' to
    writer:   &'w mut dyn Write,
    /// The number of spaces to write after every newline.
    indent:   usize,
    /// The style to return on a [`Formatter::style()`]-call.
    #[cfg(feature = "color")]
    style:    Style,
    /// Whether to apply ANSI-colors to the text resulting from this Formatter or not.
    #[cfg(feature = "color")]
    coloring: Coloring,
}

// Constructors
impl<'w> Formatter<'w> {
    /// Constructor for the Formatter.
    ///
    /// # Arguments
    /// - `writer`: A [`Write`]r that this Formatter writes to.
    ///
    /// # Returns
    /// A new Formatter ready to format.
    #[inline]
    pub const fn new(writer: &'w mut dyn Write) -> Self {
        Self {
            writer,
            indent: 0,
            #[cfg(feature = "color")]
            style: Style::new(),
            #[cfg(feature = "color")]
            coloring: Coloring::Never,
        }
    }

    /// Constructor for the Formatter that takes your color preference into account.
    ///
    /// Only available with the `color`-feature; use [`Formatter::new()`] otherwise.
    ///
    /// # Arguments
    /// - `writer`: A [`Write`]r that this Formatter writes to.
    /// - `coloring`: Whether to respect [`Formatter::style()`] choices and such.
    ///
    /// # Returns
    /// A new Formatter ready to format with your chosen color settings.
    #[cfg(feature = "color")]
    #[inline]
    pub const fn with_color(writer: &'w mut dyn Write, coloring: Coloring) -> Self { Self { writer, indent: 0, style: Style::new(), coloring } }
}

// Indentation
impl<'w> Formatter<'w> {
    /// Runs a piece of code with some indentation set.
    ///
    /// This is anagolous to calling [`Formatter::add_indent()`], run the given code, then call
    /// [`Formatter::rem_indent()`] when it completes.
    ///
    /// This is useful in case your function early-quits; anytime it returns, the indentation is
    /// removed, regardless of whether it quit before the end.
    ///
    /// # Arguments
    /// - `closure`: The [`FnOnce`] encoding the code to run with indentation.
    ///
    /// # Returns
    /// The result of `closure`.
    #[inline]
    pub fn with_indent<R>(&mut self, closure: impl FnOnce() -> R) -> R {
        self.add_indent();
        let res: R = closure();
        self.rem_indent();
        res
    }

    /// Adds an indentation to the formatter.
    ///
    /// This means that, anytime a newline is printed, the formatter injects additional spaces
    /// compared to the spaces it did before. The amount that is injected is encoded by
    /// [`INDENT_WIDTH`].
    ///
    /// If you want to add multiple indentations at the same time, see
    /// [`Formatter::add_n_indent()`] instead.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub const fn add_indent(&mut self) -> &mut Self {
        self.indent += 1;
        self
    }

    /// Adds a number of given indentation levels to the formatter.
    ///
    /// This is equivalent to calling [`Formatter::add_indent()`] `n` times.
    ///
    /// # Arguments
    /// - `n`: The number of indentation levels to add.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub const fn add_n_indent(&mut self, n: usize) -> &mut Self {
        self.indent += n;
        self
    }

    /// Removes an indentation from the formatter.
    ///
    /// This means that, anytime a newline is printed, the formatter injects fewer spaces compared
    /// to the spaces it did before. The amount that is removed is encoded by [`INDENT_WIDTH`].
    /// Note the number cannot go below 0.
    ///
    /// If you want to remove multiple indentations at the same time, see
    /// [`Formatter::rem_n_indent()`] instead.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub const fn rem_indent(&mut self) -> &mut Self {
        self.indent = self.indent.saturating_sub(1);
        self
    }

    /// Removes a number of indentation levels from the formatter.
    ///
    /// This is equivalent to calling [`Formatter::rem_indent()`] `n` times. Note the number cannot
    /// go below 0.
    ///
    /// # Arguments
    /// - `n`: The number of indentation levels to remove.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub const fn rem_n_indent(&mut self, n: usize) -> &mut Self {
        self.indent = self.indent.saturating_sub(n);
        self
    }
}

// Color
#[cfg(feature = "color")]
impl<'w> Formatter<'w> {
    /// Gets a [`Style`] object to apply to something [`Display`]able with nice styling.
    ///
    /// # Returns
    /// A fresh [`Style`] object that can be further tailored by calling methods on it.
    ///
    /// Note that it may be "fixed"; i.e., if you called [`Formatter::fix_style()`] (or
    /// derivatives), then this [`Style`] is not unstyled but instead carries that style, and
    /// calling methods on it does not change the styling that the object will apply.
    #[inline]
    pub fn style(&self) -> Style {
        match self.coloring {
            Coloring::Always => self.style.clone().force_styling(true),
            Coloring::Never => self.style.clone().force_styling(false),
            Coloring::AutoStdout => self.style.clone().for_stdout(),
            Coloring::AutoStderr => self.style.clone().for_stderr(),
        }
    }



    /// Runs code while the style is fixed.
    ///
    /// This is anagolous to calling [`Formatter::fix_style()`], run the given code, then call
    /// [`Formatter::unfix_style()`] when it completes.
    ///
    /// This is useful in case your function early-quits; anytime it returns, the style is unfixed,
    /// regardless of whether it quit before the end.
    ///
    /// # Arguments
    /// - `style`: A [`Style`] to fix the Formatter to.
    /// - `closure`: The [`FnOnce`] encoding the code to run with a fixed style.
    ///
    /// # Returns
    /// The result of `closure`.
    #[inline]
    pub fn with_fixed_style<R>(&mut self, style: Style, closure: impl FnOnce() -> R) -> R {
        self.fix_style(style);
        let res: R = closure();
        self.unfix_style();
        res
    }

    /// Fixes a certain style.
    ///
    /// Fixing a style means that, instead of returning a fresh [`Style`] that subsequent code can
    /// set, the given style is given instead on which calling any styling method is useless.
    ///
    /// # Arguments
    /// - `style`: A [`Style`] to fix the Formatter to.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub const fn fix_style(&mut self, style: Style) -> &mut Self {
        self.style = style.fix();
        self
    }

    /// Unfixes a certain style.
    ///
    /// This is the reverse of [`Formatter::fix_style()`]. [`Formatter::style()`] returns a fresh
    /// [`Style`] again that can be changed with its functions.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub const fn unfix_style(&mut self) -> &mut Self {
        self.style = Style::new();
        self
    }
}

// Write
impl<'w> Write for Formatter<'w> {
    #[inline]
    fn write_str(&mut self, s: &str) -> FResult {
        for c in s.chars() {
            // Always write the character
            self.writer.write_char(c)?;

            // Newlines see us writing the current indentation level
            if c == '\n' {
                for _ in 0..self.indent {
                    self.writer.write_str(INDENT)?;
                }
            }
        }
        Ok(())
    }
}
