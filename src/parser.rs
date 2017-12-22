//! Format parser, determines the syntax for pretty formats

use nom::IResult;
use std::fmt;
use std::str::{self, Utf8Error};
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


/// The types of possible expressions which form an expression tree
///
/// The gist format has three types of valid expressions:
///
/// 1. Named expressions
/// 2. Group Expressions
/// 3. Literal Expressions
///
/// The interpreter transforms these expressions to their final output.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    /// An expression with a name and optional arguments
    Named {
        /// Name of the macro
        name: Name,
        /// Arguments to the macro, zero or more
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tree(pub Vec<Expression>);


impl Rand for Tree {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        let mut sub = vec![Expression::rand(rng)];
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

named! {
    backslash<&[u8], Name>,
    do_parse!(tag!("\\") >> (Name::Backslash))
}

named! {
    color<&[u8], Name>,
    do_parse!(tag!("c") >> (Name::Color))
}

named! {
    unstaged<&[u8], Name>,
    do_parse!(tag!("m") >> (Name::Unstaged))
}

named! {
    modified<&[u8], Name>,
    do_parse!(tag!("M") >> (Name::Modified))
}

named! {
    untracked<&[u8], Name>,
    do_parse!(tag!("a") >> (Name::Untracked))
}

named! {
    added<&[u8], Name>,
    do_parse!(tag!("A") >> (Name::Added))
}

named! {
    conflict<&[u8], Name>,
    do_parse!(tag!("u") >> (Name::Conflict))
}

named! {
    ahead<&[u8], Name>,
    do_parse!(tag!("+") >> (Name::Ahead))
}

named! { behind<&[u8], Name>,
    do_parse!(tag!("-") >> (Name::Behind))
}

named! {
    deleted<&[u8], Name>,
    do_parse!(tag!("d") >> (Name::Deleted))
}

named! {
    deleted_staged<&[u8], Name>,
    do_parse!(tag!("D") >> (Name::DeletedStaged))
}

named! {
    renamed<&[u8], Name>,
    do_parse!(tag!("r") >> (Name::Renamed))
}

named! {
    renamed_staged<&[u8], Name>,
    do_parse!(tag!("R") >> (Name::RenamedStaged))
}


named! {
    branch<&[u8], Name>,
    do_parse!(tag!("b") >> (Name::Branch))
}


named! {
    bold<&[u8], Name>,
    do_parse!(tag!("*") >> (Name::Bold))
}


named! {
    underline<&[u8], Name>,
    do_parse!(tag!("_") >> (Name::Underline))
}


named! {
    remote<&[u8], Name>,
    do_parse!(tag!("B") >> (Name::Remote))
}

named! {
    quote<&[u8], Name>,
    do_parse!(tag!("\'") >> (Name::Quote))
}


named! {
    stashed<&[u8], Name>,
    do_parse!(tag!("h") >> (Name::Stashed))
}


named! {
    expression_name<&[u8], Name>,
    alt!(
        backslash |
        color |
        bold |
        underline |
        unstaged |
        modified |
        untracked |
        added |
        ahead |
        behind |
        conflict |
        deleted |
        deleted_staged |
        renamed |
        renamed_staged |
        branch |
        remote |
        quote |
        stashed
    )
}


/// Take string contents with valid escapes
///
/// https://github.com/Rydgel/monkey-rust/blob/master/lib/lexer/mod.rs
fn string_contents(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (i1, c1) = try_parse!(input, take!(1));
    match c1 {
        b"\'" => IResult::Done(input, vec![]),
        b"\\" => {
            let (i2, c2) = try_parse!(i1, take!(1));
            string_contents(i2).map(|done| concat_slice_vec(c2, done))
        }
        c => string_contents(i1).map(|done| concat_slice_vec(c, done)),
    }
}


/// Extend a slice with a vector, and return as a vector
///
/// https://github.com/Rydgel/monkey-rust/blob/master/lib/lexer/mod.rs
fn concat_slice_vec(c: &[u8], done: Vec<u8>) -> Vec<u8> {
    let mut new_vec = c.to_vec();
    new_vec.extend(&done);
    new_vec
}


/// Convert a vector of u8 values to a string
///
/// https://github.com/Rydgel/monkey-rust/blob/master/lib/lexer/mod.rs
fn convert_vec_utf8(v: Vec<u8>) -> Result<String, Utf8Error> {
    let slice = v.as_slice();
    str::from_utf8(slice).map(|s| s.to_owned())
}


/// Parse a string from slice
///
/// https://github.com/Rydgel/monkey-rust/blob/master/lib/lexer/mod.rs
named! {
    string<String>,
    delimited!(
        tag!("\'"),
        map_res!(string_contents, convert_vec_utf8),
        tag!("\'")
    )
}

fn literal(input: &[u8]) -> IResult<&[u8], Expression> {
    do_parse!(input, s: string >> (Expression::Literal(s)))
}

named! {
    expression_args<&[u8], Vec<Expression> >,
    ws!(
        delimited!(
            tag!("("),
            separated_list!(tag!(","), expression),
            tag!(")")
        )
    )
}

/// Parse a valid named expression
fn named_expression(input: &[u8]) -> IResult<&[u8], Expression> {
    do_parse!(input,
        tag!("\\") >>
        n: expression_name >>
        a: opt!(complete!(expression_args)) >>
        (Expression::Named {
            name: n,
            args: a,
        })
    )
}


/// Parse a valid group expression
fn group_expression(input: &[u8]) -> IResult<&[u8], Expression> {
    alt!(input,
        delimited!(tag!("\\g("), expression_tree ,tag!(")")) => {
            |sub: Tree| Expression::Group {
                l: "g(".to_string(),
                r: ")".to_string(),
                sub: sub
            }
        } |
        delimited!(tag!("\\("), expression_tree ,tag!(")")) => {
            |sub: Tree| Expression::Group {
                l: "(".to_string(),
                r: ")".to_string(),
                sub: sub
            }
        } |
        delimited!(tag!("\\{"), expression_tree ,tag!("}")) => {
            |sub: Tree| Expression::Group {
                l: "{".to_string(),
                r: "}".to_string(),
                sub: sub
            }
        } |
        delimited!(tag!("\\<"), expression_tree ,tag!(">")) => {
            |sub: Tree| Expression::Group {
                l: "<".to_string(),
                r: ">".to_string(),
                sub: sub
            }
        } |
        delimited!(tag!("\\["), expression_tree ,tag!("]")) => {
            |sub: Tree| Expression::Group {
                l: "[".to_string(),
                r: "]".to_string(),
                sub: sub
            }
        }
    )
}


/// Parse a tree of expressions
pub fn expression_tree(input: &[u8]) -> IResult<&[u8], Tree> {
    do_parse! {
        input,
        sub: many0!(expression) >> (Tree(sub))
    }
}


/// Parse a single expression, expanding nested expressions
pub fn expression(input: &[u8]) -> IResult<&[u8],Expression> {
    alt!(input,
        group_expression |
        literal |
        named_expression
    )
}


#[cfg(test)]
mod test {
    use super::*;

    quickcheck! {
        fn disp_parse_idempotent(expect: Expression) -> bool {
            let test = format!("{}", expect);
            println!("{} from {:?}", test, expect);
            let parse = expression(test.as_bytes());
            println!("\t parsed => {:?}", parse);
            let parse = parse.unwrap().1;
            println!("expect {} == {}\n", expect, parse);
            parse == expect
        }
    }

    #[test]
    fn japanese_text() {
        let test = "'日本語は綺麗なのです'\\g('試験'\\*('テスト'))".as_bytes();
        let expect = Tree(vec![
            Expression::Literal("日本語は綺麗なのです".to_string()),
            Expression::Group{ l: "g(".to_string(), r: ")".to_string(), sub: Tree(vec![
                Expression::Literal("試験".to_string()),
                Expression::Named { name: Name::Bold, args: Some(vec![
                    Expression::Literal("テスト".to_string())],
                )},
            ])},
        ]);
        let parse = expression_tree(test).unwrap().1;
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn named_expression_no_args() {
        let test = b"\\*";
        let expect = Expression::Named {
            name: Name::Bold,
            args: None,
        };
        let parse = named_expression(test).unwrap().1;
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn named_expression_empty_args() {
        let test = b"\\*()";
        let expect = Expression::Named {
            name: Name::Bold,
            args: Some(vec![]),
        };
        let parse = named_expression(test).unwrap().1;
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn named_expression_1_arg() {
        let test = b"\\*(\\_)";
        let expect = Expression::Named {
            name: Name::Bold,
            args: Some(vec![
                Expression::Named { name: Name::Underline, args: None },
            ]),
        };
        let parse = named_expression(test).unwrap().1;
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn named_expression_2_arg() {
        let test = b"\\c('blue','..')";
        let expect = Expression::Named {
            name: Name::Color,
            args: Some(vec![
                Expression::Literal("blue".to_string()),
                Expression::Literal("..".to_string()),
            ])
        };
        let parse = named_expression(test).unwrap().1;
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn empty_group_expression() {
        let test = b"\\{}\\()\\[]\\g()\\<>";
        let expect = Tree(vec![
            Expression::Group {l: "{".to_string(), r: "}".to_string(), sub: Tree(vec![])},
            Expression::Group {l: "(".to_string(), r: ")".to_string(), sub: Tree(vec![])},
            Expression::Group {l: "[".to_string(), r: "]".to_string(), sub: Tree(vec![])},
            Expression::Group {l: "g(".to_string(), r: ")".to_string(), sub: Tree(vec![])},
            Expression::Group {l: "<".to_string(), r: ">".to_string(), sub: Tree(vec![])},
        ]);
        let parse = expression_tree(test).unwrap().1;
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn disp() {
        let test = b"\\('quoted literal'\\*(\\g(\\b\\B)))";
        let expect = str::from_utf8(test).unwrap();
        let parse = expression_tree(test).unwrap().1;
        assert!(format!("{}", parse) == expect, "{} == {}\n\tparsed {:?}", parse, expect, parse);

        let test = b"\\c('blue',\\g(\\b\\B),'')";
        let expect = str::from_utf8(test).unwrap();
        let parse = expression_tree(test).unwrap().1;
        assert!(format!("{}", parse) == expect, "{} == {}\n\tparsed {:?}", parse, expect, parse);
    }
}
