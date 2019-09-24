//! glitter, a git repository status pretty-printer
//!
//!
//! Glitter is a cross-platform command-line tool and format language for making informative git prompts.  Glitter's interpreter, `glit` will:
//!
//! - Read status information from your git repository through the git api
//! - Parse and Interpret the provided format
//! - Output your format with the requested information to `stdout`
//!
//! For a detailed guide, visit the [GitHub repository](https://github.com/glfmn/glitter)l

extern crate git2;
extern crate nom;
#[cfg_attr(test, macro_use)]
#[cfg(test)]
extern crate proptest;

pub mod ast;
mod color;
pub mod git;
pub mod interpreter;
pub mod parser;

pub use git::Stats;
use std::fmt::{self, Display};
use std::io;

#[derive(Debug)]
pub enum Error<'a> {
    InterpreterError(interpreter::InterpreterErr),
    ParseError(parser::ParseError<'a>),
}

impl<'a> Error<'a> {
    pub fn pretty_print(&self, use_color: bool) -> String {
        match self {
            Error::InterpreterError(e) => format!("{:?}", e),
            Error::ParseError(e) => format!("{}", e.pretty_print(use_color)),
        }
    }
}

impl<'a> From<interpreter::InterpreterErr> for Error<'a> {
    fn from(e: interpreter::InterpreterErr) -> Self {
        Error::InterpreterError(e)
    }
}

impl<'a> From<parser::ParseError<'a>> for Error<'a> {
    fn from(e: parser::ParseError<'a>) -> Self {
        Error::ParseError(e)
    }
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            InterpreterError(e) => write!(f, "{:?}", e),
            ParseError(e) => write!(f, "{:?}", e.pretty_print(false)),
        }
    }
}

pub fn glitter<'a, W: io::Write>(
    stats: Stats,
    format: &'a str,
    allow_color: bool,
    bash_prompt: bool,
    w: &mut W,
) -> Result<(), Error<'a>> {
    let tree = parser::parse(format)?;
    interpreter::Interpreter::new(stats, allow_color, bash_prompt).evaluate(&tree, w)?;
    Ok(())
}
