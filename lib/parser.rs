//! Format parser, determines the syntax for pretty formats

use crate::ast::{Color::*, CompleteStyle, Delimiter, Expression, Name, Separator, Style, Tree};
use std::str;

use nom::IResult;

pub type ParseError = ();

fn sub_tree(input: &str) -> IResult<&str, Tree> {
    use nom::bytes::complete::tag;
    use nom::combinator::complete;
    use nom::sequence::delimited;
    complete(delimited(tag("("), expression_tree, tag(")")))(input)
}

pub fn named_expression(input: &str) -> IResult<&str, Expression> {
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::combinator::{map, opt};

    // create sub-parsers for each type of name, this defines what
    // literal values are translated to what names; must match the
    // fmt::Display implementation
    use Name::*;
    let stashed = map(tag("h"), |_| Stashed);
    let branch = map(tag("b"), |_| Branch);
    let remote = map(tag("B"), |_| Remote);
    let ahead = map(tag("+"), |_| Ahead);
    let behind = map(tag("-"), |_| Behind);
    let conflict = map(tag("u"), |_| Conflict);
    let added = map(tag("A"), |_| Added);
    let untracked = map(tag("a"), |_| Untracked);
    let modified = map(tag("M"), |_| Modified);
    let unstaged = map(tag("m"), |_| Unstaged);
    let deleted = map(tag("d"), |_| Deleted);
    let deleted_staged = map(tag("D"), |_| DeletedStaged);
    let renamed = map(tag("R"), |_| Renamed);
    let quote = map(tag("\\\'"), |_| Quote);

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

fn u8_from_bytes(input: &str) -> u8 {
    input
        .parse()
        .expect("attempted to parse a value that was not a number")
}

fn digit(input: &str) -> IResult<&str, u8> {
    use nom::bytes::complete::take_while1;
    use nom::character::is_digit;
    use nom::combinator::map;

    map(take_while1(|c| is_digit(c as u8)), u8_from_bytes)(input)
}

fn u8_triple(input: &str) -> IResult<&str, (u8, u8, u8)> {
    use nom::bytes::complete::tag;
    use nom::sequence::{terminated, tuple};

    tuple((
        terminated(digit, tag(",")),
        terminated(digit, tag(",")),
        digit,
    ))(input)
}

fn style_token(input: &str) -> IResult<&str, Style> {
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::combinator::{complete, map};
    use nom::error::context;
    use nom::sequence::delimited;

    // create sub-parsers for each style type
    use Style::*;
    macro_rules! style {
        ($tag:expr, $type:expr) => {
            map(tag($tag), |_| $type)
        };
    }
    let reset = style!("~", Reset);
    let bold = style!("*", Bold);
    let underline = style!("_", Underline);
    let italic = style!("i", Italic);
    let fg_red = style!("r", Fg(Red));
    let bg_red = style!("R", Bg(Red));
    let fg_green = style!("g", Fg(Green));
    let bg_green = style!("G", Bg(Green));
    let fg_yellow = style!("y", Fg(Yellow));
    let bg_yellow = style!("Y", Bg(Yellow));
    let fg_blue = style!("b", Fg(Blue));
    let bg_blue = style!("B", Bg(Blue));
    let fg_magenta = style!("m", Fg(Magenta));
    let bg_magenta = style!("M", Bg(Magenta));
    let fg_cyan = style!("c", Fg(Cyan));
    let bg_cyan = style!("C", Bg(Cyan));
    let fg_white = style!("w", Fg(White));
    let bg_white = style!("W", Bg(White));
    let fg_black = style!("k", Fg(Black));
    let bg_black = style!("K", Bg(Black));

    // more complicated sub-parsers for RGB/Indexed Color styles
    let fg_rgb = map(
        context(
            "rgb foreground color",
            complete(delimited(tag("["), u8_triple, tag("]"))),
        ),
        |(r, g, b)| Fg(RGB(r, g, b)),
    );
    let bg_rgb = map(
        context(
            "rgb background color",
            complete(delimited(tag("{"), u8_triple, tag("}"))),
        ),
        |(r, g, b)| Bg(RGB(r, g, b)),
    );

    alt((
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
    ))(input)
}

pub fn format_expression(input: &str) -> IResult<&str, Expression> {
    use nom::bytes::complete::tag;
    use nom::combinator::map;
    use nom::error::context;
    use nom::multi::fold_many1;
    use nom::sequence::preceded;

    let style = preceded(
        tag("#"),
        fold_many1(
            style_token,
            CompleteStyle::default(),
            |mut complete, style| {
                complete.add(style);
                complete
            },
        ),
    );

    let arguments = sub_tree;

    context("Format", style)(input).and_then(|(input, style)| {
        map(arguments, |sub_tree| Expression::Format {
            style,
            sub: sub_tree,
        })(input)
    })
}

pub fn expression_tree(input: &str) -> IResult<&str, Tree> {
    use nom::combinator::map;
    use nom::multi::many0;

    map(many0(expression), |es| Tree(es))(input)
}

pub fn group_expression(input: &str) -> IResult<&str, Expression> {
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
        group!("<", ">", Delimiter::Angle),
        group!("[", "]", Delimiter::Square),
        group!("{", "}", Delimiter::Curly),
        group!("\\(", ")", Delimiter::Parens),
    ))(input)
}

pub fn literal_expression(input: &str) -> IResult<&str, Expression> {
    use nom::bytes::complete::{tag, take_until};
    use nom::combinator::map;
    use nom::sequence::delimited;

    let contents = map(take_until("\'"), str::to_owned);

    map(
        delimited(tag("\'"), contents, tag("\'")),
        Expression::Literal,
    )(input)
}

pub fn separator_expression(input: &str) -> IResult<&str, Expression> {
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::combinator::map;

    use Separator::*;

    macro_rules! sep {
        ($sep:expr) => {
            map(tag($sep.as_str()), |_| $sep)
        };
    }

    map(
        alt((
            sep!(At),
            sep!(Bar),
            sep!(Dot),
            sep!(Comma),
            sep!(Space),
            sep!(Colon),
            sep!(Semicolon),
            sep!(Underscore),
        )),
        Expression::Separator,
    )(input)
}

/// Parse a single expression, expanding nested expressions
pub fn expression(input: &str) -> IResult<&str, Expression> {
    use nom::branch::alt;
    alt((
        named_expression,
        format_expression,
        group_expression,
        literal_expression,
        separator_expression,
    ))(input)
}

/// Parse a format
pub fn parse<I>(input: I) -> Result<Tree, ParseError>
where
    I: AsRef<str>,
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
            let parse = expression(test.as_ref());
            println!("\t parsed => {:?}", parse);
            let parse = parse.unwrap().1;
            println!("expect {} ==\nresult {}\n", expect, parse);
            assert!(parse == expect)
        }
    }

    #[test]
    fn separator() {
        use Separator::*;
        let test = "  , |  ::";
        let expect = Tree(vec![
            Expression::Separator(Space),
            Expression::Separator(Space),
            Expression::Separator(Comma),
            Expression::Separator(Space),
            Expression::Separator(Bar),
            Expression::Separator(Space),
            Expression::Separator(Space),
            Expression::Separator(Colon),
            Expression::Separator(Colon),
        ]);
        let parse = parse(test).unwrap();
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn japanese_text() {
        let test = "'日本語は綺麗なのです'['試験'#*('テスト')]";
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
        assert!(parse == expect, "{:#?} != {:#?}", parse, expect);
    }

    #[test]
    fn named_expression_no_args() {
        let test = "h";
        let expect = Expression::Named {
            name: Name::Stashed,
            sub: Tree::new(),
        };
        let parse = named_expression(test).unwrap().1;
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn named_expression_empty_args() {
        let test = "b()";
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
        let test = "b(+)";
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
        let test = "b(+-)";
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
        let test = "#[42,0,0]{0,0,42}(bB)";
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
        let test = "{}\\()[]<>";
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
        let expect = "\\('quoted literal'#*(bB))";
        let parse = match expression_tree(expect) {
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

        let expect = "#b(bB)";
        let parse = expression_tree(expect).unwrap().1;
        assert!(
            format!("{}", parse) == expect,
            "{} == {}\n\tparsed {:?}",
            parse,
            expect,
            parse
        );
    }
}
