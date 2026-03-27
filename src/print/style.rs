//  STYLE.rs
//    by Lut99
//
//  Description:
//!   Provides an explicit wrapper around a [`console::Style`] to allow us to fix using it.
//

use console::{Attribute, Color, StyledObject};


/***** HELPER MACROS *****/
/// Implements a transformation function on a `style` but only if it's not fixed.
macro_rules! style_impl {
    ($self:ident, |$style:ident| $($code:tt)*) => {
        if !$self.fixed {
            let Self { style: $style, fixed } = $self;
            Self { style: { $($code)* }, fixed }
        } else {
            $self
        }
    };
}





/***** LIBRARY *****/
/// A wrapper around [`console::Style`]'s styling object.
///
/// This does much the same, except that it has a special "override" mode that prevent it from
/// changing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Style {
    style: console::Style,
    fixed: bool,
}

// Constructors
impl Default for Style {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl Style {
    /// Constructor for the Style that initializes it as a neutral style.
    ///
    /// # Returns
    /// A new Style for styling.
    #[inline]
    pub const fn new() -> Self { Self { style: console::Style::new(), fixed: false } }

    /// Constructor for the Style that initializes it from a "dotted string".
    ///
    /// See the original function [`console::Style::from_dotted_str()`]'s documentation for more
    /// information on what this does.
    ///
    /// # Arguments
    /// - `s`: The dotted string to create this Style from.
    ///
    /// # Returns
    /// A new Style for styling.
    #[inline]
    pub fn from_dotted_str(s: &str) -> Self { Self { style: console::Style::from_dotted_str(s), fixed: false } }
}

// Color options
impl Style {
    /// Forces styling on or off.
    ///
    /// # Arguments
    /// - `value`: Whether to always use coloring (true) or never (false).
    ///
    /// # Returns
    /// Self but with the forced styling.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn force_styling(self, value: bool) -> Self { style_impl!(self, |style| style.force_styling(value)) }

    /// Will adapt the colouring of this style on whether stdout is a TTY.
    ///
    /// See [`console::Style::for_stdout()`] for more information.
    ///
    /// # Returns
    /// Self but with colouring optionally enabled if stdout is a TTY.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn for_stdout(self) -> Self { style_impl!(self, |style| style.for_stdout()) }

    /// Will adapt the colouring of this style on whether stderr is a TTY.
    ///
    /// See [`console::Style::for_stderr()`] for more information.
    ///
    /// # Returns
    /// Self but with colouring optionally enabled if stderr is a TTY.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn for_stderr(self) -> Self { style_impl!(self, |style| style.for_stderr()) }

    /// Fixes the styling of this Style.
    ///
    /// Once this function has been called, all of the other functions stop mutating this Style.
    ///
    /// This is used to prevent nested functions from changing it. As such, **this cannot be undone
    /// until you create a fresh Style.**
    ///
    /// # Returns
    /// Self that can never change anymore.
    #[inline]
    pub const fn fix(self) -> Self {
        let Self { style, fixed: _ } = self;
        Self { style, fixed: true }
    }
}

// Styling
impl Style {
    /// Sets an attribute for this Style.
    ///
    /// # Arguments
    /// - `attr`: An [`Attribute`] to set it to.
    ///
    /// # Returns
    /// Self but with the given `attr`ibute.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn attr(self, attr: Attribute) -> Self { style_impl!(self, |style| style.attr(attr)) }

    /// Sets the foreground color of this Style.
    ///
    /// # Arguments
    /// - `color`: A [`Color`] to set it to.
    ///
    /// # Returns
    /// Self but with the given foreground `color`.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn fg(self, color: Color) -> Self { style_impl!(self, |style| style.fg(color)) }

    /// Sets the background color of this Style.
    ///
    /// # Arguments
    /// - `color`: A [`Color`] to set it to.
    ///
    /// # Returns
    /// Self but with the given background `color`.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn bg(self, color: Color) -> Self { style_impl!(self, |style| style.bg(color)) }



    /// Sets the foreground color of this Style to ANSI black.
    ///
    /// # Returns
    /// Self but with the given foreground color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn black(self) -> Self { style_impl!(self, |style| style.black()) }

    /// Sets the foreground color of this Style to ANSI red.
    ///
    /// # Returns
    /// Self but with the given foreground color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn red(self) -> Self { style_impl!(self, |style| style.red()) }

    /// Sets the foreground color of this Style to ANSI green.
    ///
    /// # Returns
    /// Self but with the given foreground color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn green(self) -> Self { style_impl!(self, |style| style.green()) }

    /// Sets the foreground color of this Style to ANSI yellow.
    ///
    /// # Returns
    /// Self but with the given foreground color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn yellow(self) -> Self { style_impl!(self, |style| style.yellow()) }

    /// Sets the foreground color of this Style to ANSI blue.
    ///
    /// # Returns
    /// Self but with the given foreground color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn blue(self) -> Self { style_impl!(self, |style| style.blue()) }

    /// Sets the foreground color of this Style to ANSI magenta.
    ///
    /// # Returns
    /// Self but with the given foreground color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn magenta(self) -> Self { style_impl!(self, |style| style.magenta()) }

    /// Sets the foreground color of this Style to ANSI cyan.
    ///
    /// # Returns
    /// Self but with the given foreground color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn cyan(self) -> Self { style_impl!(self, |style| style.cyan()) }

    /// Sets the foreground color of this Style to ANSI white.
    ///
    /// # Returns
    /// Self but with the given foreground color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn white(self) -> Self { style_impl!(self, |style| style.white()) }

    /// Sets the foreground color of this Style to some 8-bit color.
    ///
    /// # Returns
    /// Self but with the given foreground color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn color256(self, color: u8) -> Self { style_impl!(self, |style| style.color256(color)) }

    /// Sets the foreground color of this Style to some 24-bit color.
    ///
    /// # Returns
    /// Self but with the given foreground color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn true_color(self, red: u8, green: u8, blue: u8) -> Self { style_impl!(self, |style| style.true_color(red, green, blue)) }

    /// Uses a bright version of the current foreground color, if any.
    ///
    /// # Returns
    /// Self but with a brighter foreground color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn bright(self) -> Self { style_impl!(self, |style| style.bright()) }

    /// Sets the background color of this Style to ANSI black.
    ///
    /// # Returns
    /// Self but with the given background color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn on_black(self) -> Self { style_impl!(self, |style| style.on_black()) }

    /// Sets the background color of this Style to ANSI red.
    ///
    /// # Returns
    /// Self but with the given background color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn on_red(self) -> Self { style_impl!(self, |style| style.on_red()) }

    /// Sets the background color of this Style to ANSI green.
    ///
    /// # Returns
    /// Self but with the given background color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn on_green(self) -> Self { style_impl!(self, |style| style.on_green()) }

    /// Sets the background color of this Style to ANSI yellow.
    ///
    /// # Returns
    /// Self but with the given background color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn on_yellow(self) -> Self { style_impl!(self, |style| style.on_yellow()) }

    /// Sets the background color of this Style to ANSI blue.
    ///
    /// # Returns
    /// Self but with the given background color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn on_blue(self) -> Self { style_impl!(self, |style| style.on_blue()) }

    /// Sets the background color of this Style to ANSI magenta.
    ///
    /// # Returns
    /// Self but with the given background color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn on_magenta(self) -> Self { style_impl!(self, |style| style.on_magenta()) }

    /// Sets the background color of this Style to ANSI cyan.
    ///
    /// # Returns
    /// Self but with the given background color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn on_cyan(self) -> Self { style_impl!(self, |style| style.on_cyan()) }

    /// Sets the background color of this Style to ANSI white.
    ///
    /// # Returns
    /// Self but with the given background color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn on_white(self) -> Self { style_impl!(self, |style| style.on_white()) }

    /// Sets the background color of this Style to some 8-bit color.
    ///
    /// # Returns
    /// Self but with the given background color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn on_color256(self, color: u8) -> Self { style_impl!(self, |style| style.on_color256(color)) }

    /// Sets the background color of this Style to some 24-bit color.
    ///
    /// # Returns
    /// Self but with the given background color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn on_true_color(self, red: u8, green: u8, blue: u8) -> Self { style_impl!(self, |style| style.on_true_color(red, green, blue)) }

    /// Uses a bright version of the current background color, if any.
    ///
    /// # Returns
    /// Self but with a brighter background color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn on_bright(self) -> Self { style_impl!(self, |style| style.on_bright()) }

    /// Uses a bold font.
    ///
    /// # Returns
    /// Self but with a bold font.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn bold(self) -> Self { style_impl!(self, |style| style.bold()) }

    /// Uses a dimmer foreground color, if any.
    ///
    /// # Returns
    /// Self but with a dimmer foreground color.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn dim(self) -> Self { style_impl!(self, |style| style.dim()) }

    /// Uses an italic font.
    ///
    /// # Returns
    /// Self but with an italic font.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn italic(self) -> Self { style_impl!(self, |style| style.italic()) }

    /// Uses an underlined font.
    ///
    /// # Returns
    /// Self but with an underlined font.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn underlined(self) -> Self { style_impl!(self, |style| style.underlined()) }

    /// Sets the text to blinking.
    ///
    /// # Returns
    /// Self but with blinking text.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn blink(self) -> Self { style_impl!(self, |style| style.blink()) }

    /// Sets the text to blinking but _faster._
    ///
    /// # Returns
    /// Self but with blinking text.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn blink_fast(self) -> Self { style_impl!(self, |style| style.blink_fast()) }

    /// Runs the reverse ANSI-escape code.
    ///
    /// # Returns
    /// Self but reversed.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn reverse(self) -> Self { style_impl!(self, |style| style.reverse()) }

    /// Hides the text.
    ///
    /// # Returns
    /// Self but with hidden text.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn hidden(self) -> Self { style_impl!(self, |style| style.hidden()) }

    /// Uses a strike through'd font.
    ///
    /// # Returns
    /// Self but with a strike through'd font.
    ///
    /// Note that it is actually unchanged if this style is fixed!
    #[inline]
    pub const fn strikethrough(self) -> Self { style_impl!(self, |style| style.strikethrough()) }
}

// Application
impl Style {
    /// Applies this style to some [`Display`]able object.
    ///
    /// # Arguments
    /// - `val`: Some object that will be rendered using this style's style.
    ///
    /// # Returns
    /// A [`StyledObject`] that does the actual rendering.
    #[inline]
    pub fn apply_to<D>(&self, val: D) -> StyledObject<D> { self.style.apply_to(val) }
}

// Conversion
impl AsRef<Style> for Style {
    #[inline]
    fn as_ref(&self) -> &Style { self }
}
impl AsRef<console::Style> for Style {
    #[inline]
    fn as_ref(&self) -> &console::Style { &self.style }
}
impl From<console::Style> for Style {
    #[inline]
    fn from(value: console::Style) -> Self { Self { style: value, fixed: false } }
}
impl From<Style> for console::Style {
    #[inline]
    fn from(value: Style) -> Self { value.style }
}
