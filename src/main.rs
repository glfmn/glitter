//! gist, a git repository status pretty-printer

extern crate docopt;
extern crate git2;
#[macro_use]
extern crate serde_derive;

use docopt::Docopt;
use git2::Repository;

/// Usage specification for program which determines how to parse arguments
const USAGE: &'static str = "
gist, git repository status pretty-printer

Usage:
    gist (--help | --version)
    gist is-repo [--path DIR] [--quiet]
    gist <format> [--path DIR] [--quiet]

Options:
    -h --help        Show this screen
    -v --version     Show version
    -p --path DIR    File path to git repository
    -q --quiet       Hide error messages

gist, a git repository status pretty-printing utility, useful for
making custom prompts which incorporate information about the current
git repository, such as the branch name, number of unstaged changes,
and more.
";

/// Version, output as version information.
const VERSION: &'static str = "0.1.0"; // HACK: find better way to handle this

/// Program operation mode, retreived from Args
#[derive(Debug, PartialEq, Eq)]
enum Mode {
    /// Emit version infromation
    Version,
    /// Emit help information
    Help,
    /// Tell if we are inside a git repository or not
    IsRepo,
    /// Parse pretty-printing format and insert git stats
    Gist,
}

/// Arguments parsed from command-line according to usage string
#[derive(Debug, Deserialize)]
struct Args {
    cmd_is_repo: bool,
    arg_format: String,
    flag_help: bool,
    flag_version: bool,
    flag_quiet: bool,
    flag_path: String,
}

impl Args {
    /// Get execution mode
    fn mode(&self) -> Mode {
        if self.flag_version {
            Mode::Version
        } else if self.flag_help {
            Mode::Help
        } else if self.cmd_is_repo {
            Mode::IsRepo
        } else {
            Mode::Gist
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
        let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

        // If no path is provided, set a sensible default for the program to use, currently "."
        let path = if args.flag_path.is_empty() {
            "."
        } else {
            &args.flag_path
        };

        // Carry out primary program operation
        let error: Result<(), ProgramErr> = match args.mode() {
            // Emit version imformation
            Mode::Version => {
                println!("{}", VERSION);
                Ok(())
            },
            // Emit help information by way of usage string
            Mode::Help => {
                println!("{}", USAGE);
                Ok(())
            },
            // Determine whether the given path is a git repository
            Mode::IsRepo => {
                match Repository::open(path) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(ProgramErr::BadPath(Box::new(path))),
                }
            },
            // Parse pretty format and insert git status
            Mode::Gist => {
                match Repository::open(path) {
                    Ok(_) => {
                        Err(ProgramErr::BadFormat(Box::new(&args.arg_format)))
                    },
                    Err(_) => Err(ProgramErr::BadPath(Box::new(path))),
                }
            },
        };

        // Handle errors and instruct program what exit code to use
        match error {
            Ok(()) => Exit::Success,
            Err(ProgramErr::BadPath(path)) => {
                if !args.flag_quiet {
                    eprintln!("{} is not a git repository", path);
                }
                Exit::Failure(1)
            },
            Err(ProgramErr::BadFormat(format)) => {
                if !args.flag_quiet {
                    eprintln!("unable to parse format specifier \"{}\"", format);
                }
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
