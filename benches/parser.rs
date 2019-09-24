extern crate criterion;
extern crate glitter_lang;
use self::criterion::*;

use glitter_lang::ast::{Color, CompleteStyle, Delimiter, Expression, Name, Separator, Tree};
use glitter_lang::parser::{self, parse};

criterion_main!(parser);
criterion_group!(
    parser,
    parse_group,
    parse_named,
    parse_string,
    parse_separator,
    real_world
);

fn parse_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("group");

    use parser::group_expression;
    use Delimiter::*;

    macro_rules! linear {
        ($($name:expr),*) => {{
            Tree(vec![
                $(
                    Expression::Group {
                        d: $name,
                        sub: Tree(vec![]),
                    }
                ),*
            ])
        }}
    }

    let inputs = [
        linear![Parens],
        linear![Square, Square, Square],
        linear![Parens, Square, Angle, Curly, Parens],
    ];

    for test in inputs.iter() {
        let input = format!("{}", test);
        group.bench_with_input(
            BenchmarkId::new("linear", input.clone()),
            &input.as_ref(),
            |b, i| b.iter(|| parse(i)),
        );
    }

    /// Recursively create tree structure for tests
    macro_rules! depth {
        ($expr:tt, $($tail:tt),*) => {{
            Tree(vec![Expression::Group {
                d: $expr,
                sub: depth![$($tail),*]
            }])
        }};
        ($expr:tt) => {{
            Tree(vec![Expression::Group {
                d: $expr,
                sub: Tree::default(),
            }])
        }};
    }

    let inputs = [
        depth![Parens],
        depth![Square, Square, Square],
        depth![Parens, Square, Angle, Curly, Parens],
    ];

    for test in inputs.iter() {
        let input = format!("{}", test);
        group.bench_with_input(
            BenchmarkId::new("nested", input.clone()),
            &input.as_ref(),
            |b, i| b.iter(|| parse(i)),
        );
    }

    let input = format!("{}", depth![Curly]);
    group.bench_with_input(
        BenchmarkId::new("isolated sub-parser", input.clone()),
        &input.as_ref(),
        |b, i| b.iter(|| group_expression(i)),
    );

    group.finish();
}

fn parse_named(c: &mut Criterion) {
    macro_rules! linear {
        ($($name:expr),*) => {{
            Tree(vec![
                $(Expression::Named {
                    name: $name,
                    sub: Tree(vec![]),
                }),*
            ])
        }}
    }

    use parser::named_expression;
    use Name::*;

    let inputs = [
        linear![Modified],
        linear![Modified, Added],
        linear![Modified, Added, Untracked],
        linear![Branch, Remote, Ahead, Behind, Stashed],
        linear![Branch, Remote, Untracked, Added, Ahead, Renamed, Deleted],
    ];

    let mut group = c.benchmark_group("named");
    for test in inputs.iter() {
        let input = format!("{}", test);
        group.bench_with_input(
            BenchmarkId::new("linear", input.clone()),
            &input.as_ref(),
            |b, i| b.iter(|| parse(i)),
        );
    }

    macro_rules! depth {
        ($expr:tt, $($tail:tt),*) => {{
            Tree(vec![Expression::Named {
                name: $expr,
                sub: depth![$($tail),*]
            }])
        }};
        ($expr:tt) => {{
            Tree(vec![Expression::Named {
                name: $expr,
                sub: Tree::default(),
            }])
        }};
    }

    let inputs = [
        depth![Modified],
        depth![Modified, Added],
        depth![Modified, Added, Untracked],
        depth![Branch, Remote, Ahead, Behind, Stashed],
        depth![Branch, Remote, Untracked, Added, Ahead, Renamed, Deleted],
    ];

    for test in inputs.iter() {
        let input = format!("{}", test);
        group.bench_with_input(
            BenchmarkId::new("nested", input.clone()),
            &input.as_ref(),
            |b, i| b.iter(|| parse(i)),
        );
    }

    let input = format!("{}", depth![Modified]);
    group.bench_with_input(
        BenchmarkId::new("isolated sub-parser", input.clone()),
        &input.as_ref(),
        |b, i| b.iter(|| named_expression(i)),
    );

    group.finish();
}

fn parse_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("string");

    let test = r"1234567890!@#$%^&*()_+-=[]{};:\\||.,<>?//~`".to_string();

    use parser::literal_expression;

    let input = format!("{}", Expression::Literal(test));
    group.bench_with_input(
        BenchmarkId::new("isolated sub-parser", input.clone()),
        &input.as_ref(),
        |b, i| b.iter(|| literal_expression(i)),
    );
}

fn parse_separator(c: &mut Criterion) {
    let mut group = c.benchmark_group("separator");

    macro_rules! linear {
        ($($sep:expr),*) => {{
            Tree(vec![
                $(Expression::Separator($sep)),*
            ])
        }}
    }

    use parser::separator_expression;
    use Separator::*;

    let inputs = [
        linear![At],
        linear![At, Dot, Space],
        linear![At, Dot, Space, Colon, Semicolon],
        linear![At, Dot, Space, Colon, Semicolon, Bar, Comma],
    ];

    for test in inputs.iter() {
        let input = format!("{}", test);
        group.bench_with_input(
            BenchmarkId::new("simple", input.clone()),
            &input.as_ref(),
            |b, i| b.iter(|| parse(i)),
        );
    }

    let input = format!("{}", At);
    group.bench_with_input(
        BenchmarkId::new("isolated sub-parser", input.clone()),
        &input.as_ref(),
        |b, i| b.iter(|| separator_expression(i)),
    );

    group.finish();
}

fn real_world(c: &mut Criterion) {
    use Color::*;
    use Delimiter::*;
    use Name::*;
    use Separator::*;

    // A real-world example of a very complicated glitter format
    let test = Tree(vec![
        Expression::Format {
            style: CompleteStyle {
                fg: Some(Yellow),
                bg: None,
                bold: false,
                italics: false,
                underline: false,
            },
            sub: Tree(vec![
                Expression::Format {
                    style: CompleteStyle {
                        fg: Some(Cyan),
                        bg: None,
                        bold: true,
                        italics: false,
                        underline: false,
                    },
                    sub: Tree(vec![Expression::Literal("~".to_owned())]),
                },
                Expression::Literal("/C/u/glitter".to_owned()),
            ]),
        },
        Expression::Separator(Space),
        Expression::Group {
            d: Square,
            sub: Tree(vec![
                Expression::Format {
                    style: CompleteStyle {
                        fg: Some(Blue),
                        bg: None,
                        bold: true,
                        italics: false,
                        underline: false,
                    },
                    sub: Tree(vec![Expression::Named {
                        name: Branch,
                        sub: Tree(vec![]),
                    }]),
                },
                Expression::Separator(At),
                Expression::Format {
                    style: CompleteStyle {
                        fg: Some(Blue),
                        bg: None,
                        bold: false,
                        italics: false,
                        underline: false,
                    },
                    sub: Tree(vec![Expression::Named {
                        name: Remote,
                        sub: Tree(vec![]),
                    }]),
                },
                Expression::Separator(Colon),
                Expression::Group {
                    d: Curly,
                    sub: Tree(vec![
                        Expression::Named {
                            name: Ahead,
                            sub: Tree(vec![Expression::Format {
                                style: CompleteStyle {
                                    fg: Some(Green),
                                    bg: None,
                                    bold: false,
                                    italics: false,
                                    underline: false,
                                },
                                sub: Tree(vec![Expression::Literal("↑".to_owned())]),
                            }]),
                        },
                        Expression::Separator(Comma),
                        Expression::Named {
                            name: Behind,
                            sub: Tree(vec![Expression::Format {
                                style: CompleteStyle {
                                    fg: Some(Red),
                                    bg: None,
                                    bold: false,
                                    italics: false,
                                    underline: false,
                                },
                                sub: Tree(vec![Expression::Literal("↓".to_owned())]),
                            }]),
                        },
                    ]),
                },
                Expression::Separator(Space),
                Expression::Separator(Bar),
                Expression::Separator(Space),
                Expression::Format {
                    style: CompleteStyle {
                        fg: None,
                        bg: None,
                        bold: false,
                        italics: false,
                        underline: false,
                    },
                    sub: Tree(vec![
                        Expression::Format {
                            style: CompleteStyle {
                                fg: Some(Green),
                                bg: None,
                                bold: false,
                                italics: false,
                                underline: false,
                            },
                            sub: Tree(vec![
                                Expression::Named {
                                    name: Modified,
                                    sub: Tree(vec![]),
                                },
                                Expression::Named {
                                    name: Added,
                                    sub: Tree(vec![]),
                                },
                                Expression::Named {
                                    name: Renamed,
                                    sub: Tree(vec![]),
                                },
                                Expression::Named {
                                    name: DeletedStaged,
                                    sub: Tree(vec![]),
                                },
                            ]),
                        },
                        Expression::Separator(Colon),
                        Expression::Format {
                            style: CompleteStyle {
                                fg: Some(Red),
                                bg: None,
                                bold: false,
                                italics: false,
                                underline: false,
                            },
                            sub: Tree(vec![
                                Expression::Named {
                                    name: Unstaged,
                                    sub: Tree(vec![]),
                                },
                                Expression::Named {
                                    name: Untracked,
                                    sub: Tree(vec![]),
                                },
                                Expression::Named {
                                    name: Conflict,
                                    sub: Tree(vec![]),
                                },
                                Expression::Named {
                                    name: Deleted,
                                    sub: Tree(vec![]),
                                },
                            ]),
                        },
                        Expression::Separator(Colon),
                        Expression::Named {
                            name: Stashed,
                            sub: Tree(vec![Expression::Format {
                                style: CompleteStyle {
                                    fg: Some(RGB(153, 0, 51)),
                                    bg: None,
                                    bold: false,
                                    italics: false,
                                    underline: false,
                                },
                                sub: Tree(vec![Expression::Literal("@".to_owned())]),
                            }]),
                        },
                    ]),
                },
            ]),
        },
        Expression::Literal("\\n".to_owned()),
        Expression::Literal("➟ ".to_owned()),
    ]);

    let input = format!("{}", test);

    c.bench_with_input(
        BenchmarkId::new("real world example", input.clone()),
        &input.as_ref(),
        |b, i| b.iter(|| parse(i)),
    );
}
