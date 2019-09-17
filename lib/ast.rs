#[cfg(test)]
use proptest::collection::vec;
#[cfg(test)]
use proptest::prelude::*;
use std::fmt;

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
    /// Provide Raw ANSI escape
    Number(u8),
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
            &Style::Number(n) => write!(f, "{}", n)?,
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
        any::<u8>().prop_map(|n| Number(n)),
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
    Format { style: Vec<Style>, sub: Tree },
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

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Named { ref name, ref sub } => {
                write!(f, "{}", name)?;
                if sub.0.is_empty() {
                    Ok(())
                } else {
                    write!(f, "({})", sub)?;
                    Ok(())
                }
            }
            Expression::Group {
                ref l,
                ref r,
                ref sub,
            } if *l == "(".to_string() => write!(f, "\\({}{}", sub, r),
            Expression::Group {
                ref l,
                ref r,
                ref sub,
            } => write!(f, "{}{}{}", l, sub, r),
            Expression::Format { ref style, ref sub } => {
                write!(f, "#")?;
                if let Some((first, ss)) = style.split_first() {
                    write!(f, "{}", first)?;
                    for s in ss {
                        write!(f, ";{}", s)?;
                    }
                }
                write!(f, "({})", sub)
            }
            Expression::Literal(ref string) => write!(f, "'{}'", string),
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
            style: style,
            sub: Tree::new(),
        }),
        "[^']*".prop_map(Literal),
    ];

    leaf.prop_recursive(8, 64, 10, |inner| {
        prop_oneof![
            (arb_name(), vec(inner.clone(), 0..10)).prop_map(|(name, sub)| Named {
                name: name,
                sub: Tree(sub)
            }),
            (vec(arb_style(), 1..10), vec(inner.clone(), 0..10)).prop_map(|(style, sub)| Format {
                style: style,
                sub: Tree(sub)
            }),
            vec(inner.clone(), 0..10).prop_map(|sub| Group {
                l: "{".to_string(),
                r: "}".to_string(),
                sub: Tree(sub)
            }),
            vec(inner.clone(), 0..10).prop_map(|sub| Group {
                l: "(".to_string(),
                r: ")".to_string(),
                sub: Tree(sub)
            }),
            vec(inner.clone(), 0..10).prop_map(|sub| Group {
                l: "<".to_string(),
                r: ">".to_string(),
                sub: Tree(sub)
            }),
            vec(inner.clone(), 0..10).prop_map(|sub| Group {
                l: "[".to_string(),
                r: "]".to_string(),
                sub: Tree(sub)
            }),
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
