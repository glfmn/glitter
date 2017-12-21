//! gist, a git repository status pretty-printer

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

mod parser;

use clap::ArgMatches;
use git2::Repository;

const DESC: &'static str = "Gist is a git repository status pretty-printing utility, useful for \
making custom prompts which incorporate information about the current git repository, such as the \
branch name, number of unstaged changes, and more.";

/// Program operation mode, retreived from Args
#[derive(Debug, PartialEq, Eq)]
enum Mode<'a> {
    /// Tell if we are inside a git repository or not at the desired path
    IsRepo(&'a str),
    /// Parse pretty-printing format and insert git stats
    Gist {
        /// Path of the git repository to check
        path: &'a str,
        /// Format string to parse
        format: &'a str
    },
}

impl<'a> Mode<'a> {
    fn from_matches(matches: &'a ArgMatches) -> Self {
        if let Some(matches) = matches.subcommand_matches("isrepo") {
            Mode::IsRepo(matches.value_of("path").unwrap_or("."))
        } else {
            Mode::Gist {
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
}

fn main() {
    let exit = {
        // Read and parse command-line arguments
        let matches = clap_app!(gist =>
            (version: crate_version!())
            (author: crate_authors!())
            (about: crate_description!())
            (after_help: DESC)
            (@arg FORMAT: +required "pretty-printing format specification")
            (@arg path: -p --path +takes_value "path to test")
            (@setting ArgsNegateSubcommands)
            // (@setting SubcommandsNegateReqs)
            (@subcommand isrepo =>
                (about: "Determine if given path is a git repository")
                (@arg path: -p --path +takes_value "path to test [default \".\"]")
            )
        ).get_matches();

        // Carry out primary program operation
        let error: Result<(), ProgramErr> = match Mode::from_matches(&matches) {
            // Determine whether the given path is a git repository
            Mode::IsRepo(path) => {
                match Repository::open(path) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(ProgramErr::BadPath(Box::new(path))),
                }
            },
            // Parse pretty format and insert git status
            Mode::Gist{ path, format } => {
                match Repository::open(path) {
                    Ok(_) => {
                        let _ = parser::expression_tree(format.as_bytes());
                        Err(ProgramErr::BadFormat(Box::new(format)))
                    },
                    Err(_) => Err(ProgramErr::BadPath(Box::new(path))),
                }
            },
        };

        // Handle errors and instruct program what exit code to use
        match error {
            Ok(()) => Exit::Success,
            Err(ProgramErr::BadPath(path)) => {
                eprintln!("{} is not a git repository", path);
                Exit::Failure(1)
            },
            Err(ProgramErr::BadFormat(format)) => {
                eprintln!("unable to parse format specifier \"{}\"", format);
                Exit::Failure(1)
            }
        }
    };

    // Exit with desiered exit code, done outside of the scope of the main program so most values
    // have a chance to clean up and exit.
    match exit {
        Exit::Failure(code) => std::process::exit(code),
        _ => (),
    };
}
