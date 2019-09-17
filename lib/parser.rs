//! Format parser, determines the syntax for pretty formats

use crate::ast::{Color::*, CompleteStyle, Delimiter, Expression, Name, Style, Tree};
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
    use nom::bytes::complete::take_while1;
    use nom::character::is_digit;
    use nom::combinator::map;

    map(take_while1(is_digit), u8_from_bytes)(input)
}

pub fn format_expression(input: &[u8]) -> IResult<&[u8], Expression> {
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::combinator::{complete, map, opt};
    use nom::multi::fold_many1;
    use nom::sequence::{delimited, preceded, terminated, tuple};

    // create sub-parsers for each style type
    use Style::*;
    macro_rules! style {
        ($tag:expr, $type:expr) => {
            map(tag($tag), |_| $type)
        };
    }
    let reset = style!(b"~", Reset);
    let bold = style!(b"*", Bold);
    let underline = style!(b"_", Underline);
    let italic = style!(b"i", Italic);
    let fg_red = style!(b"r", Fg(Red));
    let bg_red = style!(b"R", Bg(Red));
    let fg_green = style!(b"g", Fg(Green));
    let bg_green = style!(b"G", Bg(Green));
    let fg_yellow = style!(b"y", Fg(Yellow));
    let bg_yellow = style!(b"Y", Bg(Yellow));
    let fg_blue = style!(b"b", Fg(Blue));
    let bg_blue = style!(b"B", Bg(Blue));
    let fg_magenta = style!(b"m", Fg(Magenta));
    let bg_magenta = style!(b"M", Bg(Magenta));
    let fg_cyan = style!(b"c", Fg(Cyan));
    let bg_cyan = style!(b"C", Bg(Cyan));
    let fg_white = style!(b"w", Fg(White));
    let bg_white = style!(b"W", Bg(White));
    let fg_black = style!(b"k", Fg(Black));
    let bg_black = style!(b"K", Bg(Black));

    // more complicated sub-parsers for RGB/Indexed Color styles
    let rgb = tuple((
        terminated(digit, tag(b",")),
        terminated(digit, tag(b",")),
        digit,
    ));
    let fg_rgb = map(
        complete(delimited(tag(b"["), &rgb, tag(b"]"))),
        |(r, g, b)| Fg(RGB(r, g, b)),
    );
    let bg_rgb = map(
        complete(delimited(tag(b"{"), &rgb, tag(b"}"))),
        |(r, g, b)| Bg(RGB(r, g, b)),
    );

    let style = alt((
        reset,
        bold,
        underline,
        italic,
        // HACK: nest alts due to size limit on tuple
        alt((
            fg_red, bg_red, fg_green, bg_green, fg_yellow, bg_yellow, fg_blue, bg_blue, fg_magenta,
            bg_magenta, fg_cyan, bg_cyan, fg_white, bg_white, fg_black, bg_black,
        )),
        fg_rgb,
        bg_rgb,
    ));

    let styles = preceded(
        tag(b"#"),
        fold_many1(style, CompleteStyle::default(), |mut complete, style| {
            complete.add(style);
            complete
        }),
    );

    let arguments = opt(sub_tree);

    styles(input).and_then(|(input, style)| {
        map(arguments, |sub_tree| Expression::Format {
            style,
            sub: sub_tree.unwrap_or_else(|| Tree::new()),
        })(input)
    })
}

pub fn expression_tree(input: &[u8]) -> IResult<&[u8], Tree> {
    use nom::combinator::map;
    use nom::multi::many0;

    map(many0(expression), |es| Tree(es))(input)
}

pub fn group_expression(input: &[u8]) -> IResult<&[u8], Expression> {
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::combinator::{complete, map};
    use nom::sequence::delimited;

    macro_rules! group {
        ($l:tt, $r:tt, $type:expr) => {
            map(
                complete(delimited(tag($l), expression_tree, tag($r))),
                |sub| Expression::Group { d: $type, sub },
            )
        };
    }

    alt((
        group!(b"<", b">", Delimiter::Angle),
        group!(b"[", b"]", Delimiter::Square),
        group!(b"{", b"}", Delimiter::Curly),
        group!(b"\\(", b")", Delimiter::Parens),
    ))(input)
}

pub fn literal_expression(input: &[u8]) -> IResult<&[u8], Expression> {
    use nom::bytes::complete::tag;
    use nom::combinator::{map, map_res};
    use nom::sequence::delimited;

    map(
        delimited(
            tag("\'"),
            map_res(string_contents, convert_vec_utf8),
            tag("\'"),
        ),
        Expression::Literal,
    )(input)
}

/// Parse a single expression, expanding nested expressions
pub fn expression(input: &[u8]) -> IResult<&[u8], Expression> {
    use nom::branch::alt;
    alt((
        named_expression,
        format_expression,
        group_expression,
        literal_expression,
    ))(input)
}

/// Parse a format
pub fn parse<I>(input: I) -> Result<Tree, ParseError>
where
    I: AsRef<[u8]>,
{
    use nom::combinator::all_consuming;
    all_consuming(expression_tree)(input.as_ref())
        .map(|(_, tree)| tree)
        .map_err(|e| {
            eprintln!("{:?}", e);
            ()
        })
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
                        style: (&[Style::Bold]).iter().collect(),
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
    fn format_rgb() {
        let test = b"#[42,0,0]{0,0,42}(bB)";
        let expect = Expression::Format {
            style: vec![Style::Fg(RGB(42, 0, 0)), Style::Bg(RGB(0, 0, 42))]
                .iter()
                .collect(),
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
