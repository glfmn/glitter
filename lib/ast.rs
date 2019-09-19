#[cfg(test)]
use proptest::collection::vec;
#[cfg(test)]
use proptest::prelude::*;
use std::fmt;
use std::iter::{Extend, FromIterator, IntoIterator};

/// All valid expression names
///
/// Defines the "standard library" of named expressions which represent git stats
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Name {
    Branch,
    Remote,
    Ahead,
    Behind,
    Conflict,
    Added,
    Untracked,
    Modified,
    Unstaged,
    Deleted,
    DeletedStaged,
    Renamed,
    Stashed,
    Quote,
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let literal = match self {
            Name::Stashed => "h",
            Name::Branch => "b",
            Name::Remote => "B",
            Name::Ahead => "+",
            Name::Behind => "-",
            Name::Conflict => "u",
            Name::Added => "A",
            Name::Untracked => "a",
            Name::Modified => "M",
            Name::Unstaged => "m",
            Name::Deleted => "d",
            Name::DeletedStaged => "D",
            Name::Renamed => "R",
            Name::Quote => "\\\'",
        };
        write!(f, "{}", literal)
    }
}

#[cfg(test)]
pub fn arb_name() -> impl Strategy<Value = Name> {
    use self::Name::*;

    prop_oneof![
        Just(Branch),
        Just(Remote),
        Just(Ahead),
        Just(Behind),
        Just(Conflict),
        Just(Added),
        Just(Untracked),
        Just(Modified),
        Just(Unstaged),
        Just(Deleted),
        Just(DeletedStaged),
        Just(Renamed),
        Just(Stashed),
        Just(Quote),
    ]
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Color {
    /// Make text red
    Red,
    /// Make text green
    Green,
    /// Make the text yellow
    Yellow,
    /// Make the text blue
    Blue,
    /// Make the text purple
    Magenta,
    /// Make the text cyan
    Cyan,
    /// Make the text white
    White,
    /// Make the text bright black
    Black,
    /// Provide a 256 color table text color value
    RGB(u8, u8, u8),
}

/// All valid style markers
///
/// Defines the range of possible styles
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Style {
    /// Reset text to plain terminal style; ANSI code 00 equivalent
    Reset,
    /// Bold text in the terminal; ANSI code 01 equivalent
    Bold,
    /// Underline text in the terminal; ANSI code 04 equivalent
    Underline,
    /// Italisize text in the terminal; ANSI code 03 equivalent
    Italic,
    /// Set a foreground color
    Fg(Color),
    /// Set a background color
    Bg(Color),
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Color::*;
        match self {
            Style::Reset => write!(f, "~")?,
            Style::Bold => write!(f, "*")?,
            Style::Underline => write!(f, "_")?,
            Style::Italic => write!(f, "i")?,
            Style::Fg(Red) => write!(f, "r")?,
            Style::Bg(Red) => write!(f, "R")?,
            Style::Fg(Green) => write!(f, "g")?,
            Style::Bg(Green) => write!(f, "G")?,
            Style::Fg(Yellow) => write!(f, "y")?,
            Style::Bg(Yellow) => write!(f, "Y")?,
            Style::Fg(Blue) => write!(f, "b")?,
            Style::Bg(Blue) => write!(f, "B")?,
            Style::Fg(Magenta) => write!(f, "m")?,
            Style::Bg(Magenta) => write!(f, "M")?,
            Style::Fg(Cyan) => write!(f, "c")?,
            Style::Bg(Cyan) => write!(f, "C")?,
            Style::Fg(White) => write!(f, "w")?,
            Style::Bg(White) => write!(f, "W")?,
            Style::Fg(Black) => write!(f, "k")?,
            Style::Bg(Black) => write!(f, "K")?,
            &Style::Fg(RGB(r, g, b)) => write!(f, "[{},{},{}]", r, g, b)?,
            &Style::Bg(RGB(r, g, b)) => write!(f, "{{{},{},{}}}", r, g, b)?,
        };
        Ok(())
    }
}

#[cfg(test)]
pub fn arb_style() -> impl Strategy<Value = Style> {
    use self::Color::*;
    use self::Style::*;

    prop_oneof![
        Just(Reset),
        Just(Bold),
        Just(Underline),
        Just(Italic),
        Just(Fg(Red)),
        Just(Bg(Red)),
        Just(Fg(Green)),
        Just(Bg(Green)),
        Just(Fg(Yellow)),
        Just(Bg(Yellow)),
        Just(Fg(Blue)),
        Just(Bg(Blue)),
        Just(Fg(Magenta)),
        Just(Bg(Magenta)),
        Just(Fg(Cyan)),
        Just(Bg(Cyan)),
        Just(Fg(White)),
        Just(Bg(White)),
        Just(Fg(Black)),
        Just(Bg(Black)),
        any::<(u8, u8, u8)>().prop_map(|(r, g, b)| Fg(RGB(r, g, b))),
        any::<(u8, u8, u8)>().prop_map(|(r, g, b)| Bg(RGB(r, g, b))),
    ]
}

/// An aggregate unit which describes the sub total of a set of styles
///
/// ```
/// use glitter_lang::ast::{Style, CompleteStyle, Color};
/// use Style::*;
/// use Color::*;
/// let complete: CompleteStyle = [Fg(Green), Bold].iter().collect();
/// assert_eq!(complete, CompleteStyle {
///     fg: Some(Green),
///     bold: true,
///     ..Default::default()
/// });
/// ```
///
/// The conversion from a collection of styles is lossy:
///
/// ```
/// use glitter_lang::ast::{Style, CompleteStyle, Color};
/// use Style::*;
/// use Color::*;
///
/// // Style::Reset at the final position is the same as
/// // CompleteStyle::default()
/// let reset_to_default: CompleteStyle = [Bg(Red), Reset].iter().collect();
/// assert_eq!(reset_to_default, CompleteStyle::default());
///
/// // Information about repeated styles is lost
/// let green: CompleteStyle = [Fg(Green)].iter().collect();
/// let green_repeat: CompleteStyle = std::iter::repeat(&Fg(Green)).take(10).collect();
/// assert_eq!(green, green_repeat);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CompleteStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italics: bool,
    pub underline: bool,
}

impl CompleteStyle {
    pub fn add(&mut self, style: Style) {
        use Style::*;
        match style {
            Fg(color) => self.fg = Some(color),
            Bg(color) => self.bg = Some(color),
            Bold => self.bold = true,
            Italic => self.italics = true,
            Underline => self.underline = true,
            Reset => *self = Default::default(),
        }
    }
}

impl Default for CompleteStyle {
    fn default() -> Self {
        CompleteStyle {
            fg: None,
            bg: None,
            bold: false,
            italics: false,
            underline: false,
        }
    }
}

impl std::ops::AddAssign for CompleteStyle {
    fn add_assign(&mut self, with: Self) {
        if with == Default::default() {
            return *self = Default::default();
        }

        *self = Self {
            fg: with.fg.or(self.fg),
            bg: with.bg.or(self.bg),
            bold: with.bold || self.bold,
            italics: with.italics || self.italics,
            underline: with.underline || self.underline,
        }
    }
}

impl From<Style> for CompleteStyle {
    fn from(s: Style) -> Self {
        let mut ctx = Self::default();
        ctx.add(s);
        ctx
    }
}

impl fmt::Display for CompleteStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Style::*;

        if *self == Self::default() {
            return write!(f, "{}", Reset);
        }

        if let Some(color) = self.fg {
            write!(f, "{}", Fg(color))?;
        }
        if let Some(color) = self.bg {
            write!(f, "{}", Bg(color))?;
        }
        if self.bold {
            write!(f, "{}", Bold)?;
        }
        if self.italics {
            write!(f, "{}", Italic)?;
        }
        if self.underline {
            write!(f, "{}", Underline)?;
        }

        Ok(())
    }
}

impl<'a> Extend<&'a Style> for CompleteStyle {
    fn extend<E: IntoIterator<Item = &'a Style>>(&mut self, styles: E) {
        for style in styles {
            self.add(*style)
        }
    }
}

impl<'a> FromIterator<&'a Style> for CompleteStyle {
    fn from_iter<I: IntoIterator<Item = &'a Style>>(iter: I) -> CompleteStyle {
        let mut complete = CompleteStyle::default();
        complete.extend(iter);
        complete
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Delimiter {
    /// <>
    Angle,
    /// []
    Square,
    /// {}
    Curly,
    /// \()
    Parens,
}

impl Delimiter {
    pub fn left(&self) -> &'static str {
        use Delimiter::*;
        match self {
            Angle => "<",
            Square => "[",
            Curly => "{",
            Parens => "(",
        }
    }

    pub fn right(&self) -> &'static str {
        use Delimiter::*;
        match self {
            Angle => ">",
            Square => "]",
            Curly => "}",
            Parens => ")",
        }
    }
}

#[cfg(test)]
pub fn arb_delimiter() -> impl Strategy<Value = Delimiter> {
    use self::Delimiter::*;

    prop_oneof![Just(Angle), Just(Square), Just(Curly), Just(Parens)]
}

/// Special separator characters which can appear between expressions
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Separator {
    At,
    Bar,
    Dot,
    Comma,
    Space,
    Colon,
    Semicolon,
    Underscore,
}

impl Separator {
    pub fn as_str(&self) -> &'static str {
        use Separator::*;
        match self {
            At => "@",
            Bar => "|",
            Dot => ".",
            Comma => ",",
            Space => " ",
            Colon => ":",
            Semicolon => ";",
            Underscore => "_",
        }
    }
}

impl AsRef<str> for Separator {
    fn as_ref(&self) -> &'static str {
        self.as_str()
    }
}

impl fmt::Display for Separator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[cfg(test)]
pub fn arb_separator() -> impl Strategy<Value = Separator> {
    use Separator::*;

    prop_oneof![
        Just(At),
        Just(Bar),
        Just(Dot),
        Just(Comma),
        Just(Space),
        Just(Colon),
        Just(Semicolon),
        Just(Underscore),
    ]
}

/// The types of possible expressions which form an expression tree
///
/// The gist format has three types of valid expressions:
///
/// 1. Named expressions
/// 2. Group Expressions
/// 3. Literal Expressions
///
/// The interpreter transforms these expressions to their final output after they have been
/// parsed from the input string.
///
/// **Named expressions** take one of two forms: the plain form with no arguments, or with arguments
///
/// - `name` plain form
/// - `name(exp1exp2...exp3)` any number of expressions
///
/// **Group expressions** are set of expressions, which are not comma seperated.  There are a few
/// base group types:
///
/// - `\()` parentheses - wrap with parens
/// - `\{}` curly braces - wrap with curly braces
/// - `\[]` square brackets - wrap contents with square brackets
/// - `\<>` angle brackets - wrap contenst with angle brackets
///
/// By nesting groups of expressions, we can create an implicit tree.
///
/// A **literal expression** is any valid utf8 characters between single quites, except for single
/// quotes and backslashes.
///
/// ```txt
/// 'hello''we''are''literal''expressions''I am one including whitespace'
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    /// An expression with a name and optional arguments which represents git repository stats
    Named {
        /// Name of the expression
        name: Name,
        /// Arguments to the expression, zero or more
        sub: Tree,
    },
    /// An expression which represents terminal text formatting
    Format { style: CompleteStyle, sub: Tree },
    /// A group of sub-expressions which forms an expression tree
    Group {
        /// Group delimiter type, [], <>, {}, or \()
        d: Delimiter,
        /// A tree of sub expressions
        sub: Tree,
    },
    /// Literal characters including whitespace, surrounded by single quotes
    Literal(String),
    /// Separator between elements in a tree
    Separator(Separator),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Named { ref name, ref sub } => {
                write!(f, "{}", name)?;
                if sub.0.is_empty() {
                    Ok(())
                } else {
                    write!(f, "({})", sub)
                }
            }
            Expression::Group { ref d, ref sub } => match d {
                Delimiter::Square => write!(f, "[{}]", sub),
                Delimiter::Angle => write!(f, "<{}>", sub),
                Delimiter::Parens => write!(f, "\\({})", sub),
                Delimiter::Curly => write!(f, "{{{}}}", sub),
            },
            Expression::Format { ref style, ref sub } => {
                write!(f, "#")?;
                write!(f, "{}", style)?;
                write!(f, "({})", sub)
            }
            Expression::Literal(ref string) => write!(f, "'{}'", string),
            Expression::Separator(s) => write!(f, "{}", s),
        }
    }
}

#[cfg(test)]
pub fn arb_expression() -> impl Strategy<Value = Expression> {
    use self::Expression::*;

    let leaf = prop_oneof![
        arb_name().prop_map(|name| Named {
            name: name,
            sub: Tree::new(),
        }),
        vec(arb_style(), 1..5).prop_map(|style| Format {
            style: style.iter().collect(),
            sub: Tree::new(),
        }),
        "[^']*".prop_map(Literal),
        arb_separator().prop_map(Separator),
    ];

    leaf.prop_recursive(8, 64, 10, |inner| {
        prop_oneof![
            (arb_name(), vec(inner.clone(), 0..10)).prop_map(|(name, sub)| Named {
                name: name,
                sub: Tree(sub),
            }),
            (vec(arb_style(), 1..10), vec(inner.clone(), 0..10)).prop_map(|(style, sub)| Format {
                style: style.iter().collect(),
                sub: Tree(sub),
            }),
            (arb_delimiter(), vec(inner.clone(), 0..10)).prop_map(|(delimiter, sub)| Group {
                d: delimiter,
                sub: Tree(sub),
            }),
            arb_separator().prop_map(Separator),
        ]
    })
}

/// A collection of expressions which may recursively form an expression tree
///
/// Seperate struct, use mutual recursion between tree and expressions to make parsing easier to
/// implement.  May combine them in the future.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tree(pub Vec<Expression>);

impl Tree {
    /// Create an empty tree
    pub fn new() -> Tree {
        Tree(Vec::new())
    }
}

impl Default for Tree {
    fn default() -> Self {
        Tree(Vec::new())
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for exp in &self.0 {
            write!(f, "{}", exp)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub fn arb_tree(n: usize) -> impl Strategy<Value = Tree> {
    vec(arb_expression(), 0..n).prop_map(Tree)
}
