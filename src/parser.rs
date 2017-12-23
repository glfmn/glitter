//! Format parser, determines the syntax for pretty formats

use nom::IResult;
use std::str::{self, Utf8Error};
use ast::{Expression, Tree, Name};

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
