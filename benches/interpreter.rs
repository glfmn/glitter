/// Benchmarks for interpreter

#[macro_use]
extern crate criterion;
extern crate glitter_lang;

use glitter_lang::ast::{Delimiter, Expression, Name, Style, Tree};
use glitter_lang::git::Stats;
use glitter_lang::interpreter::Interpreter;

use criterion::{Bencher, Criterion, Fun};

fn stats() -> Stats {
    Stats {
        untracked: 1,
        added_staged: 1,
        modified: 1,
        modified_staged: 1,
        renamed: 1,
        deleted: 1,
        deleted_staged: 1,
        ahead: 1,
        behind: 1,
        conflicts: 1,
        stashes: 1,
        branch: "master".to_string(),
        remote: "origin/master".to_string(),
    }
}

fn empty_stats(c: &mut Criterion) {
    use Expression::*;
    use Name::*;

    let empty: Stats = Default::default();
    let expression = Tree(vec![Group {
        d: Delimiter::Square,
        sub: Tree(vec![
            Named {
                name: Modified,
                sub: Tree::new(),
            },
            Named {
                name: Added,
                sub: Tree::new(),
            },
            Named {
                name: Renamed,
                sub: Tree::new(),
            },
            Named {
                name: Deleted,
                sub: Tree::new(),
            },
        ]),
    }]);
    let mut interpreter = Interpreter::new(empty, true, true);

    c.bench_function("default stats \"[MARD]\"", move |b| {
        let mut out = Vec::with_capacity(128);
        b.iter(|| {
            out.clear();
            let _ = interpreter.evaluate(&expression, &mut out);
        })
    });
}

fn real_world(c: &mut Criterion) {
    use glitter_lang::parser::parse;

    let tree = parse(r"[#g*(b)#r(B(#~('..')))#w(\(#~*(+('↑')-('↓')))<#g(MARD)#r(maud)>{#m*_(h('@'))})]' '#b*('\w')'\n '").expect("failed to parse example");

    let mut i = Interpreter::new(stats(), true, true);
    c.bench_function("Real world \"$GIT_FMT\" example", move |b| {
        let mut out = Vec::with_capacity(256);
        b.iter(|| {
            out.clear();
            let _ = i.evaluate(&tree, &mut out);
        })
    });
}

fn nested_named(c: &mut Criterion) {
    use Expression::*;
    use Name::*;

    /// Recursively create tree structure for tests
    macro_rules! tree {
        ($expr:tt, $($tail:tt),*) => {{
            Tree(vec![Named {
                name: $expr,
                sub: tree![$($tail),*]
            }])
        }};
        ($expr:tt) => {{
            Tree(vec![Named {
                name: $expr,
                sub: Tree::default(),
            }])
        }};
    }

    macro_rules! depth {
        ($($tail:tt),+) => {{
            |b: &mut Bencher, s: &Stats| {
                let mut interpreter = Interpreter::new(s.clone(), true, true);
                // Use passed tokens as the Name type in each subtree
                let e = tree![$($tail),+];
                let mut out = Vec::with_capacity(128);
                b.iter(|| {
                    out.clear();
                    let _  = interpreter.evaluate(&e, &mut out);
                });
            }
        }};
    }

    c.bench_functions(
        "nested named",
        vec![
            Fun::new("depth 1", depth![Modified]),
            Fun::new("depth 2", depth![Modified, Added]),
            Fun::new("depth 3", depth![Modified, Added, Untracked]),
            Fun::new("depth 4", depth![Modified, Added, Untracked, Deleted]),
            Fun::new(
                "depth 5",
                depth![Modified, Added, Untracked, Deleted, Branch],
            ),
        ],
        stats(),
    );
}

fn tree_length(c: &mut Criterion) {
    use Expression::*;
    use Name::*;

    macro_rules! length_n {
        ($n:expr) => {
            |b: &mut Bencher, s: &Stats| {
                let e = Tree(
                    std::iter::repeat(Named {
                        name: Deleted,
                        sub: Tree::new(),
                    })
                    .take($n)
                    .collect(),
                );

                let mut i = Interpreter::new(s.clone(), true, true);
                let mut out = Vec::with_capacity(128);
                b.iter(|| {
                    out.clear();
                    let _ = i.evaluate(&e, &mut out);
                });
            }
        };
    }

    c.bench_functions(
        "tree length",
        vec![
            Fun::new("length 2", length_n!(2)),
            Fun::new("length 4", length_n!(4)),
            Fun::new("length 8", length_n!(8)),
            Fun::new("length 16", length_n!(16)),
            Fun::new("length 32", length_n!(32)),
        ],
        stats(),
    );
}

fn style_length(c: &mut Criterion) {
    use Expression::*;
    use Style::*;

    macro_rules! n_styles {
        ($n:expr) => {
            |b: &mut Bencher, s: &Stats| {
                let styles = Tree(vec![Format {
                    style: std::iter::repeat(&Bold).take($n).collect(),
                    sub: Tree(Vec::new()),
                }]);
                let mut i = Interpreter::new(s.clone(), true, true);
                let mut out = Vec::with_capacity(128);
                b.iter(|| {
                    out.clear();
                    let _ = i.evaluate(&styles, &mut out);
                })
            }
        };
    }

    c.bench_functions(
        "style length",
        vec![
            Fun::new("2 styles", n_styles!(2)),
            Fun::new("4 styles", n_styles!(4)),
            Fun::new("8 styles", n_styles!(8)),
            Fun::new("16 styles", n_styles!(16)),
            Fun::new("32 styles", n_styles!(32)),
        ],
        stats(),
    );
}

criterion_group!(
    interpreter,
    real_world,
    empty_stats,
    nested_named,
    tree_length,
    style_length,
);
criterion_main!(interpreter);
