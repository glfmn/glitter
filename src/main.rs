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
//! Learn more and get help with `glit help` and from the detailed guide at
//! [the GitHub repository](https://github.com/glfmn/glitter).
extern crate structopt;
#[macro_use]
extern crate human_panic;

// Currently, only used to enable windows colors
#[cfg(windows)]
extern crate yansi;

use git2::Repository;
use std::fmt::{self, Display};
use std::path::PathBuf;
use structopt::StructOpt;

use glitter_lang::{git, glitter};

#[derive(StructOpt, Debug)]
#[structopt(name = "glit")]
/// Glitter is a git repository status pretty-printing utility intended
/// for making custom shell prompts which incorporate information about
/// the current git repository, such as the branch name, number of
/// unstaged changes, and more.
struct Opt {
    /// Format used in git repositories
    git_format: String,

    /// Format used outside git repositories
    #[structopt(short = "e", long = "else-format")]
    else_format: Option<String>,

    /// Ignore syntax errors
    #[structopt(long = "silent")]
    silent_mode: bool,

    /// Escape format characters for bash shell prompts
    ///
    /// Without the escapes, BASH prompt has broken line wrapping
    #[structopt(long = "bash-escapes", short)]
    bash_escapes: bool,

    /// Path to the git repository represented by the format
    #[structopt(long, short, parse(from_os_str), default_value = ".")]
    path: PathBuf,
}

#[derive(Debug)]
enum Error {
    Git(git2::Error),
    MissingFormat(PathBuf),
    Glitter(String),
}

impl From<git2::Error> for Error {
    fn from(e: git2::Error) -> Self {
        Error::Git(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Git(e) => write!(f, "Git error: {}", e),
            MissingFormat(p) => write!(
                f,
                "No git repository in `{}` and no alternate format provided",
                p.to_string_lossy()
            ),
            Glitter(e) => write!(f, "{}", e),
        }
    }
}

fn run() -> Result<(), Error> {
    #[allow(unused)]
    let mut color = true;

    #[cfg(windows)]
    {
        use yansi::Paint;
        color = Paint::enable_windows_ascii();
    }

    let opt = Opt::from_args();

    // Get a format and stats from the git repository or exit early with an error
    let (stats, format) = Repository::discover(opt.path.clone())
        .map(|mut repo| (git::Stats::new(&mut repo), opt.git_format.clone()))
        // if no repository is found, use the alt format if it exists
        .or_else(|_| {
            if let Some(format) = opt.else_format.clone() {
                Ok((git::Stats::default(), format))
            } else {
                Err(Error::MissingFormat(opt.path.clone()))
            }
        })?;

    use std::io::BufWriter;
    let mut out = BufWriter::with_capacity(128, std::io::stdout());

    glitter(stats, &format, color, opt.bash_escapes, &mut out)
        .map_err(|e| Error::Glitter(e.pretty_print(color)))?;

    out.into_inner()
        .expect("Unable to complete writing format to output");
    println!();

    Ok(())
}

fn main() {
    setup_panic!();

    match run() {
        Ok(()) => {
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
}
