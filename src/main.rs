//! Glitter
//!
//! # Usage
//!
//! Basic usage for `glit` is:
//!
//! ```
//! $ glit <FORMAT>
//! ```
//!
//! Learn more and get help with:
//!
//! ```
//! $ glit help
//! ```
//!
//! ## Setting your shell to use `glit`
//!
//! Too add a glitter format to your shell prompt if you are in a bash shell, add the
//! following snippet to your `~/.bashrc`:
//!
//! ```bash
//! # Use environment variables to store formats if you want to be able to easily
//! # change them from your shell by just doing:
//! #
//! #   $ export PS1_FMT="#r;*('TODO')"
//!
//! # Format to use inside of git repositories or their sub-folders
//! export PS1_FMT="\<#m;*(\b)#m(\B(#~('..')))\(#g(\+)#r(\-))>\[#g;*(\M\A\R\D)#r;*(\m\a\u\d)]\{#m;*;_(\h('@'))}':'#y;*('\w')'\n\$ '"
//!
//! # Format to use outside of git repositories
//! export PS1_ELSE_FMT="#g(#*('\u')'@\h')':'#b;*('\w')'\$ '"
//!
//! # Prompt command which is used to set the prompt, includes some extra useful
//! # functionality such as showing the last exit code
//! __set_prompt() {
//!     local EXIT="$?"
//!     # Capture last command exit flag first
//!
//!     # Clear out prompt
//!     PS1=""
//!
//!     # If the last command didn't exit 0, display the exit code
//!     [ "$EXIT" -ne "0" ] && PS1+="$EXIT "
//!
//!     # identify debian chroot, if one exists
//!     if [ -z "${debian_chroot:-}" ] && [ -r /etc/debian_chroot ]; then
//!       PS1+="${debian_chroot:+($(cat /etc/debian_chroot))}"
//!     fi
//!
//!     # Render the appropriate format depending on whether we are in a git repo
//!     PS1+="$(glit "$PS1_FMT" -e "$PS1_ELSE_FMT")"
//! }
//!
//! export PROMPT_COMMAND=__set_prompt
//! ```
//!
//! Where the variable **PS1_FMT** contains your glitter format.  Here are a few
//! examples you might want to try out on your system.
//!
//! | Example `fmt`                                                                                              | Result                                                |
//! |:-----------------------------------------------------------------------------------------------------------|:------------------------------------------------------|
//! | `"\<#m;*(\b)#m(\B(#~('..')))\(#g(\+)#r(\-))>\[#g;*(\M\A\R\D)#r;*(\m\a\u\d)]\{#m;*;_(\h('@'))}"`            | ![long example glitter](img/example-1.png)            |
//! | `"\(#m;*(\b)#g(\+)#r(\-))\[#g(\M\A\R\D)#r(\m\a\u\d)]\{#m;_(\h('@'))}':'"`                                  | ![short example glitter](img/example-2.png)           |
//! | `"#g;*(\b)#y(\B(#~('..')))\[#g(\+(#~('ahead ')))]\[#r(\-(#~('behind ')))]' '#g;_(\M\A\R\D)#r;_(\m\a\u\d)"` | ![`git status sb` example glitter](img/example-3.png) |
//!
//! # Background
//!
//! Most shells provide the ability to customize the shell prompt which appears before every command.
//! On my system, the default looks like:
//!
//! ```
//! gwen@tpy12:~/Documents/dev/util/glitter$
//! ```
//!
//! Its intended to provide useful information about your shell.  However, it normally does not
//! include information about git repositories, requiring the near constant use of `git status`
//! to understand the state of the repository.  The solution is to set a prompt command and
//! dynamically update your shell with the information you want.  `glit` is made for precisely
//! this purpose: you can provide a format, and glitter will interpret it, inserting the information
//! in the format you want.

#[macro_use]
extern crate clap;
extern crate git2;
extern crate glitter_lang;

use clap::ArgMatches;
use git2::Repository;
use glitter_lang::git;
use glitter_lang::interpreter;
use glitter_lang::parser;

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
        format: &'a str,
        else_format: Option<&'a str>,
    },
    Verify {
        /// Format string to parse
        format: &'a str,
    },
}

impl<'a> Mode<'a> {
    fn from_matches(matches: &'a ArgMatches) -> Self {
        if let Some(matches) = matches.subcommand_matches("isrepo") {
            return Mode::IsRepo(matches.value_of("path").unwrap_or("."));
        };
        if let Some(matches) = matches.subcommand_matches("verify") {
            Mode::Verify {
                format: matches.value_of("FORMAT").unwrap(),
            }
        } else {
            Mode::Glitter {
                path: matches.value_of("path").unwrap_or("."),
                format: matches.value_of("FORMAT").unwrap(),
                else_format: matches.value_of("else"),
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
            (@arg path: -p --path +takes_value "path to repository [default \".\"]")
            (@arg else: -e --else +takes_value "format to use outside of a repository")
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
        )
        .get_matches();

        use ProgramErr::{BadFormat, BadParse, BadPath};

        // Carry out primary program operation
        let error: Result<(), ProgramErr> = match Mode::from_matches(&matches) {
            // Determine whether the given path is a git repository
            Mode::IsRepo(path) => match Repository::discover(path) {
                Ok(_) => Ok(()),
                Err(_) => Err(BadPath(Box::new(path))),
            },
            // Parse pretty format and insert git status
            Mode::Glitter {
                path,
                format,
                else_format,
            } => match Repository::discover(path) {
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
                                }
                                Err(_) => Err(BadFormat(Box::new(format))),
                            }
                        }
                    }
                }
                Err(_) => match else_format {
                    Some(fmt) => match parser::expression_tree(fmt.as_bytes()).to_result() {
                        Err(_) => Err(BadFormat(Box::new(format))),
                        Ok(parsed) => {
                            let int = interpreter::Interpreter::new(Default::default());
                            match int.evaluate(&parsed) {
                                Ok(result) => {
                                    println!("{}", result);
                                    Ok(())
                                }
                                Err(_) => Err(BadFormat(Box::new(format))),
                            }
                        }
                    },
                    None => Err(BadPath(Box::new(path))),
                },
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
                                Err(BadParse(Box::new(format), format!("{}", parsed)))
                            } else {
                                Ok(())
                            }
                        }
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
            }
            Err(BadFormat(format)) => {
                eprintln!("unable to parse format specifier \"{}\"", format);
                Exit::Failure(1)
            }
            Err(BadParse(format, parsed)) => {
                eprintln!(
                    "parsed \"{}\" does not match provided \"{}\"",
                    parsed, format
                );
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
