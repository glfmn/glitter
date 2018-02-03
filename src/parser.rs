//! Format parser, determines the syntax for pretty formats

use ast::{Expression, Tree, Name, Style};
use nom::{IResult, digit};
use std::str::{self, Utf8Error};

named! {
    backslash<&[u8], Name>,
    do_parse!(tag!("\\") >> (Name::Backslash))
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


named! {
    reset<&[u8], Style>,
    do_parse!(tag!("~") >> (Style::Reset))
}


named! {
    bold<&[u8], Style>,
    do_parse!(tag!("*") >> (Style::Bold))
}


named! {
    underline<&[u8], Style>,
    do_parse!(tag!("_") >> (Style::Underline))
}


named! {
    italic<&[u8], Style>,
    do_parse!(tag!("i") >> (Style::Italic))
}


named! {
    fg_red<&[u8], Style>,
    do_parse!(tag!("r") >> (Style::FgRed))
}


named! {
    bg_red<&[u8], Style>,
    do_parse!(tag!("R") >> (Style::BgRed))
}


named! {
    fg_green<&[u8], Style>,
    do_parse!(tag!("g") >> (Style::FgGreen))
}


named! {
    bg_green<&[u8], Style>,
    do_parse!(tag!("G") >> (Style::BgGreen))
}


named! {
    fg_yellow<&[u8], Style>,
    do_parse!(tag!("y") >> (Style::FgYellow))
}


named! {
    bg_yellow<&[u8], Style>,
    do_parse!(tag!("Y") >> (Style::BgYellow))
}


named! {
    fg_blue<&[u8], Style>,
    do_parse!(tag!("b") >> (Style::FgBlue))
}


named! {
    bg_blue<&[u8], Style>,
    do_parse!(tag!("B") >> (Style::BgBlue))
}


named! {
    fg_magenta<&[u8], Style>,
    do_parse!(tag!("m") >> (Style::FgMagenta))
}


named! {
    bg_magenta<&[u8], Style>,
    do_parse!(tag!("M") >> (Style::BgMagenta))
}


named! {
    fg_cyan<&[u8], Style>,
    do_parse!(tag!("c") >> (Style::FgCyan))
}


named! {
    bg_cyan<&[u8], Style>,
    do_parse!(tag!("C") >> (Style::BgCyan))
}


named! {
    fg_white<&[u8], Style>,
    do_parse!(tag!("w") >> (Style::FgWhite))
}


named! {
    bg_white<&[u8], Style>,
    do_parse!(tag!("W") >> (Style::BgWhite))
}


named! {
    fg_black<&[u8], Style>,
    do_parse!(tag!("k") >> (Style::FgBlack))
}


named! {
    bg_black<&[u8], Style>,
    do_parse!(tag!("K") >> (Style::BgBlack))
}


fn u8_from_bytes(input: &[u8]) -> u8 {
    let raw = str::from_utf8(input).expect("invalid UTF-8");
    raw.parse().expect("attempted to parse a value that was not a number")
}


named! {
    ansi_num <&[u8], u8>,
    map!(digit, u8_from_bytes)
}


named! {
    fg_rgb<&[u8], Style>,
    do_parse!(
        tag!("[") >>
        r: ansi_num >>
        tag!(",") >>
        g: ansi_num >>
        tag!(",") >>
        b: ansi_num >>
        tag!("]") >>
        (Style::FgRGB(r,g,b))
    )
}


named! {
    bg_rgb<&[u8], Style>,
    do_parse!(
        tag!("{") >>
        r: ansi_num >>
        tag!(",") >>
        g: ansi_num >>
        tag!(",") >>
        b: ansi_num >>
        tag!("}") >>
        (Style::BgRGB(r,g,b))
    )
}


named! {
    style<&[u8], Style>,
    alt!(
        reset |
        bold |
        underline |
        italic |
        fg_red |
        bg_red |
        fg_green |
        bg_green |
        fg_yellow |
        bg_yellow |
        fg_blue |
        bg_blue |
        fg_magenta |
        bg_magenta |
        fg_cyan |
        bg_cyan |
        fg_white |
        bg_white |
        fg_black |
        bg_black |
        fg_rgb |
        bg_rgb |
        do_parse!(n: ansi_num >> (Style::Number(n)))
    )
}


named! {
    styles<&[u8], Vec<Style>>,
    separated_list!(tag!(";"), style)
}


fn format_expression(input: &[u8]) -> IResult<&[u8], Expression> {
    do_parse!(input,
        tag!("#") >>
        s: styles >>
        sub: delimited!(tag!("("), expression_tree, tag!(")")) >>
        (Expression::Format {
            style: s,
            sub: sub,
        })
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


/// Parse a valid named expression
fn named_expression(input: &[u8]) -> IResult<&[u8], Expression> {
    do_parse!(input,
        tag!("\\") >>
        n: expression_name >>
        a: opt!(complete!(delimited!(tag!("("), expression_tree, tag!(")")))) >>
        (match a {
            Some(a) => Expression::Named { name: n, sub: a },
            None => Expression::Named { name: n, sub: Tree::new() },
        })
    )
}


/// Parse a valid group expression
fn group_expression(input: &[u8]) -> IResult<&[u8], Expression> {
    alt!(input,
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
        named_expression |
        format_expression
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
        let test = "'日本語は綺麗なのです'\\['試験'#*('テスト')]".as_bytes();
        let expect = Tree(vec![
            Expression::Literal("日本語は綺麗なのです".to_string()),
            Expression::Group{ l: "[".to_string(), r: "]".to_string(), sub: Tree(vec![
                Expression::Literal("試験".to_string()),
                Expression::Format {
                    style: vec![Style::Bold],
                    sub: Tree(vec![
                        Expression::Literal("テスト".to_string())
                    ]),
                },
            ])},
        ]);
        let parse = expression_tree(test).unwrap().1;
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn named_expression_no_args() {
        let test = b"\\h";
        let expect = Expression::Named {
            name: Name::Stashed,
            sub: Tree::new(),
        };
        let parse = named_expression(test).unwrap().1;
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn named_expression_empty_args() {
        let test = b"\\b()";
        let expect = Expression::Named {
            name: Name::Branch,
            sub: Tree::new(),
        };
        let parse = match named_expression(test) {
            IResult::Done(_, exp) => exp,
            fail @ _ => panic!("Failed to parse with result {:?}", fail),
        };
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn named_expression_1_arg() {
        let test = b"\\b(\\+)";
        let expect = Expression::Named {
            name: Name::Branch,
            sub: Tree(vec![
                Expression::Named { name: Name::Ahead, sub: Tree::new() },
            ]),
        };
        let parse = match named_expression(test) {
            IResult::Done(_, exp) => exp,
            fail @ _ => panic!("Failed to parse with result {:?}", fail),
        };
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn named_expression_2_arg() {
        let test = b"\\b(\\+\\-)";
        let expect = Expression::Named {
            name: Name::Branch,
            sub: Tree(vec![
                Expression::Named { name: Name::Ahead, sub: Tree::new()},
                Expression::Named { name: Name::Behind, sub: Tree::new()},
            ]),
        };
        let parse = match named_expression(test) {
            IResult::Done(_, exp) => exp,
            fail @ _ => panic!("Failed to parse with result {:?}", fail),
        };
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn format_number() {
        let test = b"#1;42(\\b\\B)";
        let expect = Expression::Format {
            style: vec![Style::Number(1), Style::Number(42)],
            sub: Tree(vec![
                Expression::Named { name: Name::Branch, sub: Tree::new()},
                Expression::Named { name: Name::Remote, sub: Tree::new()},
            ])
        };
        let parse = match format_expression(test) {
            IResult::Done(_, exp) => exp,
            fail @ _ => panic!("Failed to parse with result {:?}", fail),
        };
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn format_rgb() {
        let test = b"#[42,0,0];{0,0,42}(\\b\\B)";
        let expect = Expression::Format {
            style: vec![Style::FgRGB(42,0,0), Style::BgRGB(0,0,42)],
            sub: Tree(vec![
                Expression::Named { name: Name::Branch, sub: Tree::new()},
                Expression::Named { name: Name::Remote, sub: Tree::new()},
            ])
        };
        let parse = match format_expression(test) {
            IResult::Done(_, exp) => exp,
            fail @ _ => panic!("Failed to parse with result {:?}", fail),
        };
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn empty_group_expression() {
        let test = b"\\{}\\()\\[]\\<>";
        let expect = Tree(vec![
            Expression::Group {l: "{".to_string(), r: "}".to_string(), sub: Tree::new()},
            Expression::Group {l: "(".to_string(), r: ")".to_string(), sub: Tree::new()},
            Expression::Group {l: "[".to_string(), r: "]".to_string(), sub: Tree::new()},
            Expression::Group {l: "<".to_string(), r: ">".to_string(), sub: Tree::new()},
        ]);
        let parse = match expression_tree(test) {
            IResult::Done(_, exp) => exp,
            fail @ _ => panic!("Failed to parse with result {:?}", fail),
        };
        assert!(parse == expect, "{:?} != {:?}", parse, expect);
    }

    #[test]
    fn disp() {
        let test = b"\\('quoted literal'#*(\\b\\B))";
        let expect = str::from_utf8(test).expect("Invalid utf-8");
        let parse = match expression_tree(test) {
            IResult::Done(_, exp) => exp,
            fail @ _ => panic!("Failed to parse with result {:?}", fail),
        };
        assert!(format!("{}", parse) == expect, "{} == {}\n\tparsed {:?}", parse, expect, parse);

        let test = b"#b(\\b\\B)";
        let expect = str::from_utf8(test).unwrap();
        let parse = expression_tree(test).unwrap().1;
        assert!(format!("{}", parse) == expect, "{} == {}\n\tparsed {:?}", parse, expect, parse);
    }
}
