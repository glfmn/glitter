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
