/// Benchmarks for interpreter

#[macro_use]
extern crate criterion;
extern crate glitter_lang;

use glitter_lang::ast::{Expression, Name, Style, Tree};
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
        l: "[".to_string(),
        r: "]".to_string(),
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
    let interpreter = Interpreter::new(empty);

    c.bench_function("default stats \"\\[\\M\\A\\R\\D\\]\"", move |b| {
        b.iter(|| interpreter.evaluate(&expression))
    });
}

fn nested_named(c: &mut Criterion) {
    use Expression::*;
    use Name::*;

    fn depth_1(b: &mut Bencher, s: &Stats) {
        let interpreter = Interpreter::new(s.clone());
        let e = Tree(vec![Named {
            name: Modified,
            sub: Tree::new(),
        }]);
        b.iter(|| interpreter.evaluate(&e));
    }
    fn depth_2(b: &mut Bencher, s: &Stats) {
        let interpreter = Interpreter::new(s.clone());
        let e = Tree(vec![Named {
            name: Modified,
            sub: Tree(vec![Named {
                name: Added,
                sub: Tree::new(),
            }]),
        }]);
        b.iter(|| interpreter.evaluate(&e));
    }
    fn depth_3(b: &mut Bencher, s: &Stats) {
        let interpreter = Interpreter::new(s.clone());
        let e = Tree(vec![Named {
            name: Modified,
            sub: Tree(vec![Named {
                name: Added,
                sub: Tree(vec![Named {
                    name: Renamed,
                    sub: Tree::new(),
                }]),
            }]),
        }]);
        b.iter(|| interpreter.evaluate(&e));
    }
    fn depth_4(b: &mut Bencher, s: &Stats) {
        let interpreter = Interpreter::new(s.clone());
        let e = Tree(vec![Named {
            name: Modified,
            sub: Tree(vec![Named {
                name: Added,
                sub: Tree(vec![Named {
                    name: Renamed,
                    sub: Tree(vec![Named {
                        name: Deleted,
                        sub: Tree::new(),
                    }]),
                }]),
            }]),
        }]);
        b.iter(|| interpreter.evaluate(&e));
    }

    c.bench_functions(
        "nested named",
        vec![
            Fun::new("depth 1", depth_1),
            Fun::new("depth 2", depth_2),
            Fun::new("depth 3", depth_3),
            Fun::new("depth 4", depth_4),
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
                let tree = Tree(
                    std::iter::repeat(Named {
                        name: Deleted,
                        sub: Tree::new(),
                    }).take($n)
                        .collect(),
                );
                let i = Interpreter::new(s.clone());
                b.iter(|| i.evaluate(&tree))
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

fn style_length(c: &mut Criterion) {}

criterion_group!(interpreter, empty_stats, nested_named, tree_length);
criterion_main!(interpreter);
