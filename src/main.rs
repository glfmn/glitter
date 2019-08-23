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
    Glitter(glitter_lang::Error),
}

impl From<git2::Error> for Error {
    fn from(e: git2::Error) -> Self {
        Error::Git(e)
    }
}

impl From<glitter_lang::Error> for Error {
    fn from(e: glitter_lang::Error) -> Self {
        Error::Glitter(e)
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
            Glitter(e) => write!(f, "Error with format: {:?}", e),
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

    glitter(stats, format, color, opt.bash_escapes, &mut out)?;

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
