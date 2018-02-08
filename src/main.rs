//! glitter, a git repository status pretty-printer
//!
//! An expression based, ergonomic language for writing the status of your git repository into
//! your shell prompt.
//!
//! For example :`"\<\b\(\+\-)>\[\M\A\R\D':'\m\a\u\d]\{\h('@')}':'"` results in something that
//! might look like `<master(+1)>[M1:D3]{@5}:` where
//!
//! - `master` is the name of the current branch.
//! - `+1`: means we are 1 commit ahead of the remote branch
//! - `M1`: the number of staged modifications
//! - `D3`: is the number of unstaged deleted files
//! - `@5`: is the number of stashes
//!
//! `glit` expressions also support inline format expressions to do things like making text red,
//! or bold, or using ANSI terminal escape sequences, or setting RGB colors for your git
//! information.
//!
//! # Grammar
//!
//! `glit` expressions have four basic types of expressions:
//!
//! 1. Named expressions
//! 2. Format expressions
//! 3. Group expressions
//! 4. Literals
//!
//! ## Literals
//!
//! Any characters between single quotes literal, except for backslashes and single quotes.
//! Literals are left untouched.  For example, `'literal'` outputs `literal`.
//!
//! ## Named expressions
//!
//! Named expressions represent information about your git repository.
//!
//! | Name  | Meaning                        | Example         |
//! |:------|:-------------------------------|:----------------|
//! | `\b`  | branch name or head commit id  | `master`        |
//! | `\B`  | remote name                    | `origin/master` |
//! | `\+`  | # of commits ahead remote      | `+1`            |
//! | `\-`  | # of commits behind remote     | `-1`            |
//! | `\m`  | # of unstaged modified files   | `M1`            |
//! | `\a`  | # of untracked files           | `?1`            |
//! | `\d`  | # of unstaged deleted files    | `D1`            |
//! | `\u`  | # of merge conflicts           | `U1`            |
//! | `\M`  | # of staged modified files     | `M1`            |
//! | `\A`  | # of added files               | `A1`            |
//! | `\R`  | # of renamed files             | `R1`            |
//! | `\D`  | # of staged deleted files      | `D1`            |
//! | `\h`  | # of stashed files             | `H1`            |
//!
//! You can provide other expressions as arguments to expressions which replace the default prefix
//! which appears before the result or file count.  For example, `\h('@')` will output `@3`
//! instead of `H3` if your repository has 3 stashed files.  You can provide an arbitrary number
//! of valid expressions as a prefix to another named expression.
//!
//! ## Group Expressions
//!
//! Glitter will surround grouped expressions with parentheses or brackets, and will print nothing
//! if the group is empty.
//!
//! | Macro       | Result                           |
//! |:------------|:---------------------------------|
//! | `\[]`       | empty                            |
//! | `\()`       | empty                            |
//! | `\<>`       | empty                            |
//! | `\{}`       | empty                            |
//! | `\{\b}`     | `{master}`                       |
//! | `\<\+\->`   | `<+1-1>`                         |
//! | `\[\M\A\R]` | `[M1A3]` where `\R` is empty     |
//! | `\[\r\(\a)]`| empty, when `\r`, `\a` are empty |
//!
//! ## Format Expressions
//!
//! Glitter expressions support ANSI terminal formatting through the following styles:
//!
//! | Format               | Meaning                                       |
//! |:---------------------|:----------------------------------------------|
//! | `#~(`...`)`          | reset                                         |
//! | `#_(`...`)`          | underline                                     |
//! | `#i(`...`)`          | italic text                                   |
//! | `#*(`...`)`          | bold text                                     |
//! | `#r(`...`)`          | red text                                      |
//! | `#g(`...`)`          | green text                                    |
//! | `#b(`...`)`          | blue text                                     |
//! | `#m(`...`)`          | magenta/purple text                           |
//! | `#y(`...`)`          | yellow text                                   |
//! | `#w(`...`)`          | white text                                    |
//! | `#k(`...`)`          | bright black text                             |
//! | `#[01,02,03](`...`)` | 24 bit rgb text color                         |
//! | `#R(`...`)`          | red background                                |
//! | `#G(`...`)`          | green background                              |
//! | `#B(`...`)`          | blue background                               |
//! | `#M(`...`)`          | magenta/purple background                     |
//! | `#Y(`...`)`          | yellow background                             |
//! | `#W(`...`)`          | white background                              |
//! | `#K(`...`)`          | bright black background                       |
//! | `#{01,02,03}(`...`)` | 24 bit rgb background color                   |
//! | `#01(`...`)`         | Fixed terminal color                          |
//!
//! Format styles can be combined in a single expression by separating them with semicolons:
//!
//! | Format         | Meaning                        |
//! |:---------------|:-------------------------------|
//! | `#w;K(`...`)`  | white text, black background   |
//! | `#r;*(`...`)`  | red bold text                  |
//! | `#42(`...`)`   | a forest greenish color        |
//! | `#_;*(`...`)`  | underline bold text            |

extern crate ansi_term;
#[macro_use]
extern crate clap;
extern crate git2;
#[macro_use]
extern crate nom;
extern crate rand;
#[macro_use]
extern crate rand_derive;

#[cfg(test)] #[macro_use]
#[cfg(test)] extern crate quickcheck;

mod ast;
mod git;
mod interpreter;
mod parser;

use clap::ArgMatches;
use git2::Repository;

const DESC: &'static str = "Glitter is a git repository status pretty-printing utility, useful for
making custom prompts which incorporate information about the current
git repository, such as the branch name, number of unstaged changes,
and more.";

/// Program operation mode, retreived from Args
#[derive(Debug, PartialEq, Eq)]
enum Mode<'a> {
    /// Tell if we are inside a git repository or not at the desired path
    IsRepo(&'a str),
    /// Parse pretty-printing format and insert git stats
    Glitter {
        /// Path of the git repository to check
        path: &'a str,
        /// Format string to parse
        format: &'a str
    },
    Verify {
        /// Format string to parse
        format: &'a str
    },
}

impl<'a> Mode<'a> {
    fn from_matches(matches: &'a ArgMatches) -> Self {
        if let Some(matches) = matches.subcommand_matches("isrepo") {
            return Mode::IsRepo(matches.value_of("path").unwrap_or("."))
        };
        if let Some(matches) = matches.subcommand_matches("verify") {
            Mode::Verify {
                format: matches.value_of("FORMAT").unwrap()
            }
        } else {
            Mode::Glitter {
                path: matches.value_of("path").unwrap_or("."),
                format: matches.value_of("FORMAT").unwrap(),
            }
        }
    }
}

/// Program exit conditions, allows for smoother cleanup and operation of the main program
#[derive(Debug, PartialEq, Eq)]
enum Exit {
    Failure(i32),
    Success,
}

/// Error types for program operation
#[derive(Debug, PartialEq, Eq)]
enum ProgramErr<'a> {
    BadPath(Box<&'a str>),
    BadFormat(Box<&'a str>),
    BadParse(Box<&'a str>, String),
}

fn main() {
    let exit = {
        // Read and parse command-line arguments
        let matches = clap_app!(glit =>
            (version: crate_version!())
            (author: crate_authors!())
            (about: crate_description!())
            (after_help: DESC)
            (@arg FORMAT: +required "pretty-printing format specification")
            (@arg path: -p --path +takes_value "path to test [default \".\"]")
            (@setting ArgsNegateSubcommands)
            (@setting SubcommandsNegateReqs)
            (@subcommand isrepo =>
                (about: "Determine if given path is a git repository")
                (@arg path: -p --path +takes_value "path to test [default \".\"]")
            )
            (@subcommand verify =>
                (about: "Determine if FORMAT parses correctly")
                (@arg FORMAT: +required "pretty-printing format specification")
            )
        ).get_matches();

        use ProgramErr::{BadFormat, BadPath, BadParse};

        // Carry out primary program operation
        let error: Result<(), ProgramErr> = match Mode::from_matches(&matches) {
            // Determine whether the given path is a git repository
            Mode::IsRepo(path) => {
                match Repository::discover(path) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(BadPath(Box::new(path))),
                }
            },
            // Parse pretty format and insert git status
            Mode::Glitter{ path, format } => {
                match Repository::discover(path) {
                    Ok(mut repo) => {
                        let parse = parser::expression_tree(format.as_bytes()).to_result();
                        match parse {
                            Err(_) => Err(BadFormat(Box::new(format))),
                            Ok(parsed) => {
                                let stats = git::Stats::new(&mut repo);
                                let interpreter = interpreter::Interpreter::new(stats.unwrap());
                                match interpreter.evaluate(&parsed) {
                                    Ok(result) => {
                                        println!("{}", result);
                                        Ok(())
                                    },
                                    Err(_) => Err(BadFormat(Box::new(format))),
                                }
                            },
                        }
                    },
                    Err(_) => Err(BadPath(Box::new(path))),
                }
            },
            Mode::Verify { format } => {
                let parse = parser::expression_tree(format.as_bytes());
                if parse.is_incomplete() {
                    Err(BadFormat(Box::new(format)))
                } else {
                    match parse.to_result() {
                        Err(_) => Err(BadFormat(Box::new(format))),
                        Ok(parsed) => {
                            if format!("{}", parsed) != format {
                                Err(BadParse(Box::new(format), format!("{}",parsed)))
                            } else {
                                Ok(())
                            }
                        },
                    }
                }
            }
        };

        // Handle errors and instruct program what exit code to use
        match error {
            Ok(()) => Exit::Success,
            Err(BadPath(path)) => {
                eprintln!("\"{}\" is not a git repository", path);
                Exit::Failure(1)
            },
            Err(BadFormat(format)) => {
                eprintln!("unable to parse format specifier \"{}\"", format);
                Exit::Failure(1)
            },
            Err(BadParse(format, parsed)) => {
                eprintln!("parsed \"{}\" does not match provided \"{}\"", parsed, format);
                Exit::Failure(1)
            },
        }
    };

    // Exit with desiered exit code, done outside of the scope of the main program so most values
    // have a chance to clean up and exit.
    match exit {
        Exit::Failure(code) => std::process::exit(code),
        _ => (),
    };
}
