//! Format parser, determines the syntax for pretty formats

use crate::ast::{Color::*, Delimiter, Expression, Name, Style, Tree};
use std::str::{self, Utf8Error};

use nom::IResult;

pub type ParseError = ();

fn u8_from_bytes(input: &[u8]) -> u8 {
    let raw = str::from_utf8(input).expect("invalid UTF-8");
    raw.parse()
        .expect("attempted to parse a value that was not a number")
}

/// Take string contents with valid escapes
///
/// https://github.com/Rydgel/monkey-rust/blob/master/lib/lexer/mod.rs
fn string_contents(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (i1, c1) = try_parse!(input, take!(1));
    match c1 {
        b"\'" => IResult::Ok((input, vec![])),
        c => string_contents(i1).map(|(i2, done)| (i2, concat_slice_vec(c, done))),
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

fn sub_tree(input: &[u8]) -> IResult<&[u8], Tree> {
    use nom::bytes::complete::tag;
    use nom::combinator::complete;
    use nom::sequence::delimited;
    complete(delimited(tag(b"("), expression_tree, tag(b")")))(input)
}

pub fn named_expression(input: &[u8]) -> IResult<&[u8], Expression> {
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::combinator::{map, opt};

    // create sub-parsers for each type of name, this defines what
    // literal values are translated to what names; must match the
    // fmt::Display implementation
    use Name::*;
    let stashed = map(tag(b"h"), |_| Stashed);
    let branch = map(tag(b"b"), |_| Branch);
    let remote = map(tag(b"B"), |_| Remote);
    let ahead = map(tag(b"+"), |_| Ahead);
    let behind = map(tag(b"-"), |_| Behind);
    let conflict = map(tag(b"u"), |_| Conflict);
    let added = map(tag(b"A"), |_| Added);
    let untracked = map(tag(b"a"), |_| Untracked);
    let modified = map(tag(b"M"), |_| Modified);
    let unstaged = map(tag(b"m"), |_| Unstaged);
    let deleted = map(tag(b"d"), |_| Deleted);
    let deleted_staged = map(tag(b"D"), |_| DeletedStaged);
    let renamed = map(tag(b"R"), |_| Renamed);
    let quote = map(tag(b"\\\'"), |_| Quote);

    // Combine sub-parsers for each name value to get a sum total
    let name = alt((
        stashed,
        branch,
        remote,
        ahead,
        behind,
        conflict,
        added,
        untracked,
        modified,
        unstaged,
        deleted,
        deleted_staged,
        renamed,
        quote,
    ));

    // Optional argument sub_tree
    let prefix = opt(sub_tree);

    // First, read name from input and then read the arguments.
    name(input).and_then(|(input, name)| {
        map(prefix, |args| Expression::Named {
            name,
            sub: args.unwrap_or_else(|| Tree::new()),
        })(input)
    })
}

pub fn digit(input: &[u8]) -> IResult<&[u8], u8> {
    unimplemented!()
}

pub fn format_expression(input: &[u8]) -> IResult<&[u8], Expression> {
    unimplemented!()
}

pub fn expression_tree(input: &[u8]) -> IResult<&[u8], Tree> {
    unimplemented!()
}

/// Parse a single expression, expanding nested expressions
pub fn expression(input: &[u8]) -> IResult<&[u8], Expression> {
    unimplemented!()
}

/// Parse a format
pub fn parse<I>(input: I) -> Result<Tree, ParseError>
where
    I: AsRef<[u8]>,
{
    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::arb_expression;

    proptest! {
        #[test]
        fn disp_parse_invariant(expect in arb_expression()) {
            let test = format!("{}", expect);
            println!("{} from {:?}", test, expect);
            let parse = expression(test.as_bytes());
            println!("\t parsed => {:?}", parse);
            let parse = parse.unwrap().1;
            println!("expect {} ==\nresult {}\n", expect, parse);
            assert!(parse == expect)
        }
    }

    #[test]
    fn japanese_text() {
        let test = "'日本語は綺麗なのです'['試験'#*('テスト')]".as_bytes();
        let expect = Tree(vec![
            Expression::Literal("日本語は綺麗なのです".to_string()),
            Expression::Group {
                d: Delimiter::Square,
                sub: Tree(vec![
                    Expression::Literal("試験".to_string()),
                    Expression::Format {
                        style: vec![Style::Bold],
                        sub: Tree(vec![Expression::Literal("テスト".to_string())]),
                    },
                ]),
            },
        ]);
        let parse = expression_tree(test).unwrap().1;
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn named_expression_no_args() {
        let test = b"h";
        let expect = Expression::Named {
            name: Name::Stashed,
            sub: Tree::new(),
        };
        let parse = named_expression(test).unwrap().1;
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn named_expression_empty_args() {
        let test = b"b()";
        let expect = Expression::Named {
            name: Name::Branch,
            sub: Tree::new(),
        };
        let parse = match named_expression(test) {
            IResult::Ok((_, exp)) => exp,
            fail @ _ => panic!("Failed to parse with result {:?}", fail),
        };
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn named_expression_1_arg() {
        let test = b"b(+)";
        let expect = Expression::Named {
            name: Name::Branch,
            sub: Tree(vec![Expression::Named {
                name: Name::Ahead,
                sub: Tree::new(),
            }]),
        };
        let parse = match named_expression(test) {
            IResult::Ok((_, exp)) => exp,
            fail @ _ => panic!("Failed to parse with result {:?}", fail),
        };
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn named_expression_2_arg() {
        let test = b"b(+-)";
        let expect = Expression::Named {
            name: Name::Branch,
            sub: Tree(vec![
                Expression::Named {
                    name: Name::Ahead,
                    sub: Tree::new(),
                },
                Expression::Named {
                    name: Name::Behind,
                    sub: Tree::new(),
                },
            ]),
        };
        let parse = match named_expression(test) {
            IResult::Ok((_, exp)) => exp,
            fail @ _ => panic!("Failed to parse with result {:?}", fail),
        };
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn format_number() {
        let test = b"#1;42(bB)";
        let expect = Expression::Format {
            style: vec![Style::Number(1), Style::Number(42)],
            sub: Tree(vec![
                Expression::Named {
                    name: Name::Branch,
                    sub: Tree::new(),
                },
                Expression::Named {
                    name: Name::Remote,
                    sub: Tree::new(),
                },
            ]),
        };
        let parse = match format_expression(test) {
            IResult::Ok((_, exp)) => exp,
            fail @ _ => panic!("Failed to parse with result {:?}", fail),
        };
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn format_rgb() {
        let test = b"#[42,0,0];{0,0,42}(bB)";
        let expect = Expression::Format {
            style: vec![Style::Fg(RGB(42, 0, 0)), Style::Bg(RGB(0, 0, 42))],
            sub: Tree(vec![
                Expression::Named {
                    name: Name::Branch,
                    sub: Tree::new(),
                },
                Expression::Named {
                    name: Name::Remote,
                    sub: Tree::new(),
                },
            ]),
        };
        let parse = match format_expression(test) {
            IResult::Ok((_, exp)) => exp,
            fail @ _ => panic!("Failed to parse with result {:?}", fail),
        };
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn empty_group_expression() {
        let test = b"{}\\()[]<>";
        let expect = Tree(vec![
            Expression::Group {
                d: Delimiter::Curly,
                sub: Tree::new(),
            },
            Expression::Group {
                d: Delimiter::Parens,
                sub: Tree::new(),
            },
            Expression::Group {
                d: Delimiter::Square,
                sub: Tree::new(),
            },
            Expression::Group {
                d: Delimiter::Angle,
                sub: Tree::new(),
            },
        ]);
        let parse = match expression_tree(test) {
            IResult::Ok((_, exp)) => exp,
            fail @ _ => panic!("Failed to parse with result {:?}", fail),
        };
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn disp() {
        let test = b"\\('quoted literal'#*(bB))";
        let expect = str::from_utf8(test).expect("Invalid utf-8");
        let parse = match expression_tree(test) {
            IResult::Ok((_, exp)) => exp,
            fail => panic!("Failed to parse with result {:?}", fail),
        };
        assert!(
            format!("{}", parse) == expect,
            "{} == {}\n\tparsed {:?}",
            parse,
            expect,
            parse
        );

        let test = b"#b(bB)";
        let expect = str::from_utf8(test).unwrap();
        let parse = expression_tree(test).unwrap().1;
        assert!(
            format!("{}", parse) == expect,
            "{} == {}\n\tparsed {:?}",
            parse,
            expect,
            parse
        );
    }
}
