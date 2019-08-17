use crate::ast::{Color, Style};

use std::io;
use std::iter::{Extend, FromIterator, IntoIterator};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct StyleContext {
    fg: Option<Color>,
    bg: Option<Color>,
    bold: bool,
    italics: bool,
    underline: bool,
}

macro_rules! e {
    ($c:tt, $($cn:expr),*) => {
        concat!["\x1B[", $c, $(";", $cn,)* "m"]
    };
    ($c:tt) => {
        e!($c,)
    };
    () => {
        e!("0")
    };
}

impl StyleContext {
    pub fn write_to<W: io::Write>(&self, w: &mut W, bash_prompt: bool) -> io::Result<()> {
        use Color::*;

        if bash_prompt {
            write!(w, "\u{01}")?;
        }

        if self != &Default::default() {
            if let Some(fg) = self.fg {
                match fg {
                    Black => write!(w, e!("30"))?,
                    Red => write!(w, e!("31"))?,
                    Green => write!(w, e!("32"))?,
                    Yellow => write!(w, e!("33"))?,
                    Blue => write!(w, e!("34"))?,
                    Magenta => write!(w, e!("35"))?,
                    Cyan => write!(w, e!("36"))?,
                    White => write!(w, e!("37"))?,
                    RGB(r, g, b) => write!(w, e!("38", "2", "{};{};{}"), r, g, b)?,
                }
            }

            if let Some(bg) = self.bg {
                match bg {
                    Black => write!(w, e!("40"))?,
                    Red => write!(w, e!("41"))?,
                    Green => write!(w, e!("42"))?,
                    Yellow => write!(w, e!("43"))?,
                    Blue => write!(w, e!("44"))?,
                    Magenta => write!(w, e!("45"))?,
                    Cyan => write!(w, e!("46"))?,
                    White => write!(w, e!("47"))?,
                    RGB(r, g, b) => write!(w, e!("48", "2", "{};{};{}"), r, g, b)?,
                }
            }

            if self.bold {
                write!(w, e!("1"))?;
            }

            if self.italics {
                write!(w, e!("3"))?;
            }

            if self.underline {
                write!(w, e!("4"))?;
            }
        } else {
            write!(w, e!())?;
        }

        if bash_prompt {
            write!(w, "\u{02}")?;
        }

        Ok(())
    }

    pub fn write_difference<W: io::Write>(
        &self,
        w: &mut W,
        prev: &StyleContext,
        bash_prompt: bool,
    ) -> io::Result<()> {
        match Difference::between(&prev, &self) {
            Difference::Add(style) => style.write_to(w, bash_prompt)?,
            Difference::Reset => {
                if bash_prompt {
                    write!(w, concat!["\u{01}", e!()])?;
                    self.write_to(w, false)?;
                    write!(w, "\u{02}")?;
                } else {
                    write!(w, e!())?;
                    self.write_to(w, false)?;
                }
            }
            Difference::None => { /* Do nothing! */ }
        };

        Ok(())
    }

    pub fn add(&mut self, style: Style) {
        use Style::*;
        match style {
            Fg(color) => self.fg = Some(color),
            Bg(color) => self.bg = Some(color),
            Bold => self.bold = true,
            Italic => self.italics = true,
            Underline => self.underline = true,
            Number(_) => (),
            Reset => *self = Default::default(),
        }
    }
}

impl Default for StyleContext {
    fn default() -> Self {
        StyleContext {
            fg: None,
            bg: None,
            bold: false,
            italics: false,
            underline: false,
        }
    }
}

impl<'a> Extend<&'a Style> for StyleContext {
    fn extend<E: IntoIterator<Item = &'a Style>>(&mut self, styles: E) {
        for style in styles {
            self.add(*style)
        }
    }
}

impl<'a> FromIterator<&'a Style> for StyleContext {
    fn from_iter<I: IntoIterator<Item = &'a Style>>(iter: I) -> StyleContext {
        let mut context = StyleContext::default();

        for style in iter {
            context.add(*style);
        }

        context
    }
}

pub(crate) enum Difference {
    None,
    Add(StyleContext),
    Reset,
}

impl Difference {
    pub fn between(prev: &StyleContext, next: &StyleContext) -> Self {
        if prev == next {
            return Difference::None;
        }

        if (prev.fg.is_some() && next.fg.is_none())
            || (prev.bg.is_some() && next.bg.is_none())
            || (prev.bold && !next.bold)
            || (prev.italics && !next.italics)
            || (prev.underline && !next.underline)
        {
            return Difference::Reset;
        }

        Difference::Add(StyleContext {
            fg: if next.fg != prev.fg { next.fg } else { None },
            bg: if next.bg != prev.bg { next.bg } else { None },
            bold: !prev.bold && next.bold,
            italics: !prev.italics && next.italics,
            underline: !prev.underline && next.underline,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct StyledString {
    style: StyleContext,
    result: String,
}

impl<'a> StyledString {
    pub fn new(style: StyleContext, result: String) -> StyledString {
        StyledString { style, result }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.result.is_empty()
    }
}

impl Into<(StyleContext, String)> for StyledString {
    fn into(self) -> (StyleContext, String) {
        (self.style, self.result)
    }
}
