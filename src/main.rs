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

fn main() {
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
    match args.mode() {
        // Emit version imformation
        Mode::Version => {
            println!("{}", VERSION);
        },
        // Emit help information by way of usage string
        Mode::Help => {
            println!("{}", USAGE);
        },
        // Determine whether the given path is a git repository
        Mode::IsRepo => {
            match Repository::open(path) {
                Ok(_) => println!("yes"),
                Err(_) => println!("no"),
            };
        },
        // Parse pretty format and insert git status
        Mode::Gist => {
            let _ = match Repository::open(path) {
                Ok(repo) => {
                    println!("main program");
                    repo
                },
                Err(_) => panic!("not a git repository"),
            };
        },
    }
}
