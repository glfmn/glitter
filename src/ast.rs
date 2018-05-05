use std::fmt;
use std::str;

#[cfg(test)]
use rand::{Rand, Rng};
#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

/// All valid expression names
///
/// Defines the "standard library" of named expressions which represent git stats
#[cfg_attr(test, derive(Rand))]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Name {
    Backslash,
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
            &Name::Stashed => "h",
            &Name::Branch => "b",
            &Name::Remote => "B",
            &Name::Ahead => "+",
            &Name::Behind => "-",
            &Name::Conflict => "u",
            &Name::Added => "A",
            &Name::Untracked => "a",
            &Name::Modified => "M",
            &Name::Unstaged => "m",
            &Name::Deleted => "d",
            &Name::DeletedStaged => "D",
            &Name::Renamed => "R",
            &Name::Backslash => "\\",
            &Name::Quote => "\'",
        };
        write!(f, "{}", literal)
    }
}


#[cfg(test)]
impl Arbitrary for Name {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        g.gen::<Name>()
    }
}


/// All valid style markers
///
/// Defines the range of possible styles
#[cfg_attr(test, derive(Rand))]
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
    /// Make text red; ANSI foreground code 31 equivalent
    FgRed,
    /// Make text background red ANSI background code 41 equivalent
    BgRed,
    /// Make text green; ANSI background code 32 equivalent
    FgGreen,
    /// Make text background green; ANSI code 42 equivalent
    BgGreen,
    /// Make the text yellow; ANSI code 33 equivalent
    FgYellow,
    /// Make the text background yellow; ANSI code 43 equivalent
    BgYellow,
    /// Make the text blue; ANSI code 34 equivalent
    FgBlue,
    /// Make the text background blue; ANSI code 44 equivalent
    BgBlue,
    /// Make the text magenta or purple; ANSI code 35 equivalent
    FgMagenta,
    /// Make the text background magenta or purple; ANSI code 45 equivalent
    BgMagenta,
    /// Make the text cyan; ANSI code 36 equivalent
    FgCyan,
    /// Make the text background cyan; ANSI code 46 equivalent
    BgCyan,
    /// Make the text white; ANSI code 37 equivalent
    FgWhite,
    /// Make the text background white; ANSI code 47 equivalent
    BgWhite,
    /// Provide a 256 color table text color value; ANSI code 38 equivalent
    FgRGB(u8,u8,u8),
    /// Provide a 256 color table text background color value; ANSI code 48 equivalent
    BgRGB(u8,u8,u8),
    /// Make the text bright black; ANSI code 90 equivalent
    FgBlack,
    /// Make the text background bright black; ANSI code 100 equivalent
    BgBlack,
    /// Provide a raw number escape code to represent terminal formatting
    Number(u8),
}


impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let literal = match self {
            &Style::Reset => "~".to_string(),
            &Style::Bold => "*".to_string(),
            &Style::Underline => "_".to_string(),
            &Style::Italic => "i".to_string(),
            &Style::FgRed => "r".to_string(),
            &Style::BgRed => "R".to_string(),
            &Style::FgGreen => "g".to_string(),
            &Style::BgGreen => "G".to_string(),
            &Style::FgYellow => "y".to_string(),
            &Style::BgYellow => "Y".to_string(),
            &Style::FgBlue => "b".to_string(),
            &Style::BgBlue => "B".to_string(),
            &Style::FgMagenta => "m".to_string(),
            &Style::BgMagenta => "M".to_string(),
            &Style::FgCyan => "c".to_string(),
            &Style::BgCyan => "C".to_string(),
            &Style::FgWhite => "w".to_string(),
            &Style::BgWhite => "W".to_string(),
            &Style::FgRGB(r,g,b) => format!( "[{},{},{}]",  r,g,b),
            &Style::BgRGB(r,g,b) => format!("{{{},{},{}}}", r,g,b),
            &Style::FgBlack => "k".to_string(),
            &Style::BgBlack => "K".to_string(),
            &Style::Number(n) => n.to_string(),
        };
        write!(f, "{}", literal)
    }
}


#[cfg(test)]
impl Arbitrary for Style {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        g.gen::<Style>()
    }
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
/// - `\name` plain form
/// - `\name(\exp1\exp2...\expn)` any number of expressions, no separaters
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
    Format {
        style: Vec<Style>,
        sub: Tree,
    },
    /// A group of sub-expressions which forms an expression tree
    Group {
        /// Left delimiter
        l: String,
        /// Right delimiter
        r: String,
        /// A tree of sub expressions
        sub: Tree,
    },
    /// Literal characters including whitespace, surrounded by single quotes
    Literal(String),
}


#[cfg(test)]
impl Rand for Expression {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        use self::Expression::{ Named, Literal, Group, Format };
        match rng.gen_range(0, 5) {
            0 => {
                let mut sub = Vec::new();
                while let Some(e) = Option::<Expression>::rand(rng) {
                    sub.push(e);
                }
                Named { name: Name::rand(rng), sub: Tree(sub) }
            },
            1 => Named { name: Name::rand(rng), sub: Tree(Vec::new()) },
            2 => {
                let mut s = Vec::new();
                while let Some(c) = Option::<u8>::rand(rng) {
                    s.push(c)
                }
                let s = str::from_utf8(&s).unwrap_or("#");
                let s = str::replace(&s, "'", "");
                let s = str::replace(&s, "\\", "");
                Literal(s.to_string())
            },
            3 => {
                let mut style = vec![Style::rand(rng)];
                while let Some(s) = Option::<Style>::rand(rng) {
                    style.push(s);
                }
                let mut sub = Vec::new();
                while let Some(e) = Option::<Expression>::rand(rng) {
                    sub.push(e);
                }
                Format { style: style, sub: Tree(sub) }
            }
            _ => {
                let mut sub = Vec::new();
                while let Some(e) = Option::<Expression>::rand(rng) {
                    sub.push(e);
                }
                match rng.gen_range(0, 4) {
                    0 => Group {l: "{".to_string(), r: "}".to_string(), sub: Tree(sub)},
                    1 => Group {l: "(".to_string(), r: ")".to_string(), sub: Tree(sub)},
                    2 => Group {l: "[".to_string(), r: "]".to_string(), sub: Tree(sub)},
                    _ => Group {l: "<".to_string(), r: ">".to_string(), sub: Tree(sub)},
                }
            }
        }
    }
}


#[cfg(test)]
impl Arbitrary for Expression {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        g.gen::<Expression>()
    }
}


impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Expression::Named { ref name, ref sub } => {
                write!(f, "\\{}", name)?;
                if sub.0.is_empty() {
                    Ok(())
                } else {
                    write!(f, "({})", sub)?;
                    Ok(())
                }
            },
            &Expression::Group { ref l, ref r, ref sub } => {
                write!(f, "\\{}{}{}", l, sub, r)
            },
            &Expression::Format { ref style, ref sub } => {
                write!(f, "#")?;
                if let Some((first, ss)) = style.split_first() {
                    write!(f, "{}", first)?;
                    for s in ss {
                        write!(f, ";{}", s)?;
                    }
                }
                write!(f, "({})", sub)
            }
            &Expression::Literal(ref string) => write!(f, "'{}'", string),
        }
    }
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


#[cfg(test)]
impl Rand for Tree {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        let mut sub = vec![];
        while let Some(e) = Option::<Expression>::rand(rng) {
            sub.push(e);
        }
        Tree(sub)
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
