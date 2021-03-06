//! Format parser, determines the syntax for pretty formats

mod combinator;

use crate::ast::{Color::*, CompleteStyle, Delimiter, Expression, Name, Separator, Style, Tree};
use std::fmt::{self, Display};
use std::str;

use combinator::{delimited_many0, map_err, map_fail};
use nom::{error, IResult};

/// Parse a format
pub fn parse<'a>(input: &'a str) -> Result<Tree, ParseError<'a>> {
    use nom::combinator::all_consuming;
    use nom::Err;

    all_consuming(expression_tree)(input.as_ref())
        .map(|(_, tree)| tree)
        .map_err(|e| match e {
            Err::Error(e) => e,
            Err::Failure(e) => e,
            _ => unreachable!("Parser failed to complete"),
        })
}

pub fn expression_tree<'a>(input: &'a str) -> IResult<&str, Tree, ParseError<'a>> {
    use nom::combinator::map;
    use nom::multi::many0;

    map(many0(expression), Tree)(input)
}

/// Parse a single expression, expanding nested expressions
pub fn expression<'a>(input: &'a str) -> IResult<&str, Expression, ParseError<'a>> {
    use nom::branch::alt;
    use nom::error::context;

    alt((
        context("group", group_expression),
        context("string", literal_expression),
        context("format", format_expression),
        separator_expression,
        named_expression,
    ))(input)
}

fn sub_tree<'a>(input: &'a str) -> IResult<&str, Tree, ParseError<'a>> {
    use nom::character::complete::char;
    use nom::combinator::map;
    // use nom::sequence::delimited;

    let items = delimited_many0(
        char('('),
        expression,
        map_err(char(')'), |_, e| {
            ParseError::missing_delimiter(input, e, ')')
        }),
    );

    map(items, Tree)(input)
}

pub fn named_expression<'a>(input: &'a str) -> IResult<&str, Expression, ParseError<'a>> {
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::char;
    use nom::combinator::{map, opt};

    // sub-parsers for each type of name, this defines what
    // literal values are translated to what names; must match the
    // fmt::Display implementation
    use Name::*;
    let name = alt((
        map(char('h'), |_| Stashed),
        map(char('b'), |_| Branch),
        map(char('B'), |_| Remote),
        map(char('+'), |_| Ahead),
        map(char('-'), |_| Behind),
        map(char('u'), |_| Conflict),
        map(char('A'), |_| Added),
        map(char('a'), |_| Untracked),
        map(char('M'), |_| Modified),
        map(char('m'), |_| Unstaged),
        map(char('d'), |_| Deleted),
        map(char('D'), |_| DeletedStaged),
        map(char('R'), |_| Renamed),
        map(tag("\\\'"), |_| Quote),
    ));

    let name = map_err(name, ParseError::missing_name);

    // Optional argument sub_tree
    let prefix = map_err(opt(sub_tree), |_, e| {
        error::ParseError::add_context(input, "expression", e)
    });

    // First, read name from input and then read the arguments.
    name(input).and_then(|(input, name)| {
        map(prefix, |args| Expression::Named {
            name,
            sub: args.unwrap_or_else(|| Tree::new()),
        })(input)
    })
}

fn u8_from_bytes<'a>(input: &'a str) -> u8 {
    input
        .parse()
        .expect("attempted to parse a value that was not a number")
}

fn digit<'a>(input: &'a str) -> IResult<&str, u8, ParseError<'a>> {
    use nom::bytes::complete::take_while1;
    use nom::character::is_digit;
    use nom::combinator::map;

    map(take_while1(|c| is_digit(c as u8)), u8_from_bytes)(input)
}

fn u8_triple<'a>(input: &'a str) -> IResult<&str, (u8, u8, u8), ParseError<'a>> {
    use nom::character::complete::char;
    use nom::sequence::{terminated, tuple};

    tuple((
        terminated(digit, char(',')),
        terminated(digit, char(',')),
        digit,
    ))(input)
}

fn style_token<'a>(input: &'a str) -> IResult<&str, Style, ParseError<'a>> {
    use nom::branch::alt;
    use nom::character::complete::char;
    use nom::combinator::{complete, map};
    use nom::sequence::delimited;

    // create sub-parsers for each style type
    use Style::*;
    macro_rules! style {
        ($tag:expr, $type:expr) => {
            map(char($tag), |_| $type)
        };
    }

    // sub-parsers for each type of style, this defines what
    // literals translate to what Style Tokens; must match the
    // fmt::Display implementation
    let styles = alt((
        style!('~', Reset),
        style!('*', Bold),
        style!('_', Underline),
        style!('i', Italic),
        style!('r', Fg(Red)),
        style!('R', Bg(Red)),
        style!('g', Fg(Green)),
        style!('G', Bg(Green)),
        style!('y', Fg(Yellow)),
        style!('Y', Bg(Yellow)),
        style!('b', Fg(Blue)),
        style!('B', Bg(Blue)),
        style!('m', Fg(Magenta)),
        style!('M', Bg(Magenta)),
        style!('c', Fg(Cyan)),
        style!('C', Bg(Cyan)),
        style!('w', Fg(White)),
        style!('W', Bg(White)),
        style!('k', Fg(Black)),
        style!('K', Bg(Black)),
    ));

    // more complicated sub-parsers for RGB/Indexed Color styles
    let fg_rgb = map(
        complete(delimited(
            char('['),
            map_fail(u8_triple, ParseError::invalid_rgb),
            map_fail(char(']'), ParseError::char_to_delimiter),
        )),
        |(r, g, b)| Fg(RGB(r, g, b)),
    );
    let bg_rgb = map(
        complete(delimited(
            char('{'),
            map_fail(u8_triple, ParseError::invalid_rgb),
            map_fail(char('}'), ParseError::char_to_delimiter),
        )),
        |(r, g, b)| Bg(RGB(r, g, b)),
    );

    alt((fg_rgb, bg_rgb, map_err(styles, ParseError::missing_style)))(input)
}

pub fn format_expression<'a>(input: &'a str) -> IResult<&str, Expression, ParseError<'a>> {
    use nom::bytes::complete::tag;
    use nom::combinator::{cut, map};
    use nom::multi::fold_many1;
    use nom::sequence::preceded;

    let tokens = fold_many1(
        style_token,
        CompleteStyle::default(),
        |mut complete, style| {
            complete.add(style);
            complete
        },
    );

    let style = preceded(
        tag("#"),
        map_fail(tokens, |i, e| {
            if let ParseErrorKind::Other(_) = e.error.1 {
                ParseError::missing_style(i, e)
            } else {
                e
            }
        }),
    );

    let arguments = cut(sub_tree);

    style(input).and_then(|(input, style)| {
        map(arguments, |sub_tree| Expression::Format {
            style,
            sub: sub_tree,
        })(input)
    })
}

pub fn group_expression<'a>(input: &'a str) -> IResult<&str, Expression, ParseError<'a>> {
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::char;
    use nom::combinator::map;

    macro_rules! group {
        ($l:tt, $r:tt, $type:expr) => {
            map(
                delimited_many0(
                    tag($l),
                    expression,
                    map_err(char($r), |_, e| ParseError::char_to_delimiter(input, e)),
                ),
                |sub| Expression::Group {
                    d: $type,
                    sub: Tree(sub),
                },
            )
        };
    }

    alt((
        group!("<", '>', Delimiter::Angle),
        group!("[", ']', Delimiter::Square),
        group!("{", '}', Delimiter::Curly),
        group!("\\(", ')', Delimiter::Parens),
    ))(input)
}

pub fn literal_expression<'a>(input: &'a str) -> IResult<&str, Expression, ParseError<'a>> {
    use nom::bytes::complete::take_until;
    use nom::character::complete::char;
    use nom::combinator::map;
    use nom::sequence::delimited;

    let contents = map(
        map_fail(take_until("\'"), |i, mut e: ParseError<'a>| {
            e.error = (i, UnclosedString);
            e
        }),
        str::to_owned,
    );

    use ParseErrorKind::UnclosedString;

    map(
        delimited(char('\''), contents, char('\'')),
        Expression::Literal,
    )(input)
}

pub fn separator_expression<'a>(input: &'a str) -> IResult<&str, Expression, ParseError<'a>> {
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

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError<'a> {
    error: (&'a str, ParseErrorKind),
    context: Option<(&'a str, &'static str)>,
    top: Option<(&'a str, &'static str)>,
}

#[derive(Debug, PartialEq, Clone)]
enum ParseErrorKind {
    UnclosedString,
    MissingDelimiter(char),
    MissingChar(char),
    UnrecognizedName,
    UnrecognizedStyle,
    InvalidRGB,
    Other(error::ErrorKind),
}

/// Indirect fmt::Display in order to configure whether to use color
#[derive(Debug, Clone)]
pub struct PrettyPrinter<'a> {
    error: ParseError<'a>,
    use_color: bool,
}

impl<'a> ParseError<'a> {
    fn missing_delimiter(input: &'a str, mut other: Self, delimiter: char) -> Self {
        other.error = (input, ParseErrorKind::MissingDelimiter(delimiter));
        other
    }

    fn missing_name(input: &'a str, mut other: Self) -> Self {
        other.error = (input, ParseErrorKind::UnrecognizedName);
        other
    }

    fn missing_style(input: &'a str, mut other: Self) -> Self {
        use ParseErrorKind::UnrecognizedStyle;
        other.error = (input, UnrecognizedStyle);
        other
    }

    fn char_to_delimiter(input: &'a str, mut other: Self) -> Self {
        use ParseErrorKind::{MissingChar, MissingDelimiter};
        if let MissingChar(c) = other.error.1 {
            other.error = (input, MissingDelimiter(c));
        }
        other
    }

    fn invalid_rgb(input: &'a str, mut other: Self) -> Self {
        other.error = (input, ParseErrorKind::InvalidRGB);
        other
    }

    pub fn pretty_print(&self, use_color: bool) -> PrettyPrinter<'a> {
        PrettyPrinter {
            use_color,
            error: self.clone(),
        }
    }
}

impl<'a> error::ParseError<&'a str> for ParseError<'a> {
    fn from_error_kind(input: &'a str, kind: error::ErrorKind) -> Self {
        ParseError {
            error: (input, ParseErrorKind::Other(kind)),
            context: None,
            top: None,
        }
    }

    fn append(_input: &'a str, _kind: error::ErrorKind, other: Self) -> Self {
        other
    }

    fn add_context(input: &'a str, context: &'static str, mut other: Self) -> Self {
        other.context = other.context.or(Some((input, context)));
        other.top = Some((input, context));
        other
    }

    fn from_char(input: &'a str, missing: char) -> Self {
        ParseError {
            error: (input, ParseErrorKind::MissingChar(missing)),
            context: None,
            top: None,
        }
    }
}

impl<'a> PrettyPrinter<'a> {
    pub fn pretty_print(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseErrorKind::*;
        match &self.error.error.1 {
            UnclosedString => self.error_message(self.error.error.0.len(), f, |f, bold| {
                writeln!(f, "missing closing quote ({})", bold.paint("\'"))
            }),
            MissingDelimiter(d) => self.error_message(1, f, |f, bold| {
                writeln!(f, "reached end without finding matching {}", bold.paint(d))
            }),
            MissingChar(c) => {
                let found: &str = &self.error.error.0.get(0..1).unwrap_or("");
                self.error_message(1, f, |f, bold| {
                    writeln!(
                        f,
                        "expected \"{}\" here, found \"{}\"",
                        bold.paint(c),
                        bold.paint(found)
                    )
                })
            }
            UnrecognizedName | Other(error::ErrorKind::Eof) => {
                let found = self.error.error.0.get(0..1).unwrap_or("");
                self.error_message(found.len().max(1), f, |f, _| {
                    if found == "]" || found == ")" || found == ">" || found == "}" {
                        writeln!(f, "improper close delimiter")
                    } else {
                        writeln!(f, "not recognized as a valid expression")
                    }
                })
            }
            UnrecognizedStyle => {
                let found: &str = &self.error.error.0.get(0..1).unwrap_or("");
                self.error_message(1, f, |f, bold| {
                    writeln!(f, "found \"{}\" which is not a style", bold.paint(found))
                })
            }
            InvalidRGB => {
                // find a potential matching brace and show interest up to that region
                let found = self
                    .error
                    .error
                    .0
                    .find(|c| c == ']' || c == '}')
                    .unwrap_or(1);
                self.error_message(found.min(5).max(1), f, |f, bold| {
                    writeln!(f, "RGB must be in the form \"{}\"", bold.paint("0,0,0"))
                })
            }
            Other(e) => self.error_message(1, f, |f, _| writeln!(f, "{:?}", e)),
        }
    }

    fn error_message<F>(&self, error_size: usize, f: &mut fmt::Formatter, message: F) -> fmt::Result
    where
        F: Fn(&mut fmt::Formatter, yansi::Style) -> fmt::Result,
    {
        use yansi::{Color, Style};

        let bold = if self.use_color {
            Style::new(Color::Unset).bold()
        } else {
            Style::new(Color::Unset)
        };

        let error = if self.use_color {
            Style::new(Color::Red).bold()
        } else {
            Style::new(Color::Unset)
        };

        let dim = if self.use_color {
            Style::new(Color::Unset).dimmed()
        } else {
            Style::new(Color::Unset)
        };

        if let Some((input, context)) = self.error.context {
            writeln!(f, "{}: unable to parse {}", error.paint("error"), context)?;
            writeln!(f, " {}", bold.paint("│"))?;
            writeln!(f, " {}    {}", bold.paint("│"), input)?;
            write!(f, " {}    ", bold.paint("│"))?;
            if let Some(i) = input.rfind(self.error.error.0) {
                for _ in 0..i {
                    write!(f, " ")?;
                }
            }
        } else {
            writeln!(f, "{}: unable to parse", error.paint("error"))?;
            writeln!(f, " {}    ", bold.paint("│"))?;
            writeln!(f, " {}    {}", bold.paint("│"), self.error.error.0)?;
            write!(f, " {}    ", bold.paint("│"))?;
        }

        for _ in 0..error_size {
            write!(f, "{}", error.paint("^"))?;
        }
        write!(f, " ")?;
        message(f, bold)?;

        writeln!(f, " {}", bold.paint("│"))?;
        if let (Some((top_input, top)), Some((input, _))) = (self.error.top, self.error.context) {
            let (pre, input) = top_input.split_at(top_input.rfind(input).unwrap_or(0));
            let (input, er) = input.split_at(input.rfind(self.error.error.0).unwrap_or(0));
            let (er, post) = er.split_at(error_size.min(er.len()));
            write!(
                f,
                " = in {}: {}{}{}{}",
                top,
                dim.paint(pre),
                input,
                error.paint(er),
                dim.paint(post)
            )?;
        }

        Ok(())
    }
}

impl<'a> Display for PrettyPrinter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.pretty_print(f)
    }
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.pretty_print(false))
    }
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
