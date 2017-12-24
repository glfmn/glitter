use std::fmt;
use std::str;
use rand::{Rand, Rng};

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

/// All valid expression names
///
/// Defines the "standard library" of named expressions
#[derive(Debug, PartialEq, Eq, Copy, Clone, Rand)]
pub enum Name {
    Backslash,
    Color,
    Bold,
    Underline,
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
    RenamedStaged,
    Quote,
    Stashed,
}


impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let literal = match self {
            &Name::Stashed => "h",
            &Name::Backslash => "\\",
            &Name::Color => "c",
            &Name::Bold => "*",
            &Name::Underline => "_",
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
            &Name::Renamed => "r",
            &Name::RenamedStaged => "R",
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
/// **Named expressions** take one of two forms: the plain form with no arguments, or with a list
/// of arguments, comma seperated.
///
/// - `\name` plain form
/// - `\name(exp1,exp2,...,expn)` with expressions as arguments, comma seperated.
///
/// **Group expressions** are set of expressions, which are not comma seperated.  There are a few
/// base group types:
///
/// - `\()` parentheses - wrap with parens
/// - `\{}` curly braces - wrap with curly braces
/// - `\[]` square brackets - wrap contents with square brackets
/// - `\<>` angle brackets - wrap contenst with angle brackets
/// - `\g()` bare group - do not wrap contents with anything
///
/// The base of all gist expressions is an implicit bare group.  Thus, the following is a valid
/// gist expression even though expressions are next to each-other without an explicit bare group.
///
/// ```txt
/// \(\*(\b\B)\+\-)\[\A\M\D\R]\{\h('@')}'~'
/// ```
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
    /// An expression with a name and optional arguments
    Named {
        /// Name of the expression
        name: Name,
        /// Arguments to the expression, zero or more
        args: Option<Vec<Expression>>,
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


impl Rand for Expression {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        use self::Expression::{ Named, Literal, Group };
        match rng.gen_range(0, 4) {
            0 => {
                let mut args = Vec::new();
                while let Some(e) = Option::<Expression>::rand(rng) {
                    args.push(e);
                }
                Named { name: Name::rand(rng), args: Some(args) }
            },
            1 => Named { name: Name::rand(rng), args: None },
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
            _ => {
                let mut sub = Vec::new();
                while let Some(e) = Option::<Expression>::rand(rng) {
                    sub.push(e);
                }
                match rng.gen_range(0, 5) {
                    0 => Group {l: "{".to_string(), r: "}".to_string(), sub: Tree(sub)},
                    1 => Group {l: "(".to_string(), r: ")".to_string(), sub: Tree(sub)},
                    2 => Group {l: "[".to_string(), r: "]".to_string(), sub: Tree(sub)},
                    3 => Group {l: "g(".to_string(), r: ")".to_string(), sub: Tree(sub)},
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
            &Expression::Named { ref name, ref args } => {
                write!(f, "\\{}", name)?;
                match args {
                    &None => Ok(()),
                    &Some(ref args) => {
                        write!(f, "(")?;
                        if let Some((last, es)) = args.split_last() {
                            for e in es {
                                write!(f, "{},", e)?;
                            }
                            write!(f, "{}", last)?;
                        }
                        write!(f, ")")?;
                        Ok(())
                    }
                }
            },
            &Expression::Group { ref l, ref r, ref sub } => {
                write!(f, "\\{}{}{}", l, sub, r)
            },
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
