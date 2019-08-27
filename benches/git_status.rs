#[macro_use]
extern crate criterion;
extern crate git2;
extern crate glitter_lang;
extern crate tempfile;

use git2::Repository;
use glitter_lang::git::Stats;
use std::fs::File;
use tempfile::{tempdir, TempDir};

use criterion::{Bencher, Criterion};

fn repo() -> (TempDir, Repository) {
    let dir = tempdir().expect("Unable to make temp dir");

    let repo = Repository::init(dir.path()).unwrap();
    {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "name").unwrap();
        config.set_str("user.email", "email").unwrap();

        let mut index = repo.index().unwrap();
        let id = index.write_tree().unwrap();

        let tree = repo.find_tree(id).unwrap();
        let sig = repo.signature().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();
    }

    (dir, repo)
}

macro_rules! powers_of {
    ($n:tt from $acc:ident take $m:expr) => {{
        [()].iter()
            .cycle()
            .map(|_| {
                $acc *= $n;
                $acc
            })
            .take($m)
    }};
}

fn untracked_files(c: &mut Criterion) {
    // Generate n untracked files and gather their status information
    let fun = |b: &mut Bencher, n: &u16| {
        let (dir, mut repo) = repo();
        for f in 0..*n {
            let file_path = dir.path().join(format!("file-{}.txt", f));
            drop(File::create(file_path).unwrap());
        }

        // Ensure we are benching what we want to bench
        assert_eq!(Stats::new(&mut repo).untracked, *n as u16);
        b.iter(|| {
            Stats::new(&mut repo);
        })
    };

    // Bench over 10 powers of n starting with 1
    let mut n = 1;
    let xs = powers_of!(2 from n take 11);
    c.bench_function_over_inputs("Number of Untracked Files", fun, xs);
}

fn added_files(c: &mut Criterion) {
    use git2::IndexAddOption;

    // Generate n added files and gather their status information
    let fun = |b: &mut Bencher, n: &u16| {
        let (dir, mut repo) = repo();
        for f in 0..*n {
            let file_path = dir.path().join(format!("file-{}.txt", f));
            drop(File::create(file_path).unwrap());
        }
        repo.index()
            .unwrap()
            .add_all(&["*"], IndexAddOption::DEFAULT, None)
            .unwrap();
        // Ensure we are benching what we want to bench
        assert_eq!(Stats::new(&mut repo).added_staged, *n);

        b.iter(|| {
            Stats::new(&mut repo);
        })
    };

    // Bench over 10 powers of n starting with n^2
    let mut n = 1;
    let xs = powers_of!(2 from n take 10);
    c.bench_function_over_inputs("Number of Added Files", fun, xs);
}

criterion_group!(index, added_files, untracked_files);

fn discover_repo(c: &mut Criterion) {
    let fun = |b: &mut Bencher| {
        let (dir, _) = repo();
        let path = dir.path();

        b.iter(|| {
            Repository::discover(path).unwrap();
        })
    };

    c.bench_function("Discover git repository", fun);
}

criterion_group!(repository, discover_repo);

criterion_main!(repository, index);
