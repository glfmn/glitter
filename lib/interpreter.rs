//! Interpreter which transforms expressions into the desired output

use crate::ast::{self, CompleteStyle, Delimiter, Expression, Name, Tree};
use crate::color::*;
use crate::git::Stats;

use std::{fmt, io};

/// Various types of Interpreter errors
#[derive(Debug)]
pub enum InterpreterErr {
    UnexpectedArgs { exp: Expression },
    WriteError(io::Error),
}

impl From<io::Error> for InterpreterErr {
    fn from(e: io::Error) -> Self {
        InterpreterErr::WriteError(e)
    }
}

type Result<T = bool> = std::result::Result<T, InterpreterErr>;

/// The interpreter which transforms a gist expression using the provided stats
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Interpreter {
    stats: Stats,
    allow_color: bool,
    bash_prompt: bool,
    command_queue: Vec<WriteCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WriteCommand {
    WriteContext(CompleteStyle),
    WriteStr(&'static str),
    #[allow(unused)] // unused variant left in case of extension
    WriteString(String),
}

impl Interpreter {
    /// Create a new Interpreter with the given stats
    pub fn new(stats: Stats, allow_color: bool, bash_prompt: bool) -> Interpreter {
        Interpreter {
            stats,
            allow_color,
            bash_prompt,
            command_queue: Vec::with_capacity(32),
        }
    }

    fn drain_queue(&mut self, i: usize) {
        self.command_queue.truncate(self.command_queue.len() - i);
    }

    fn queue_str(&mut self, s: &'static str) {
        self.command_queue.push(WriteCommand::WriteStr(s));
    }

    #[inline(always)]
    fn with_command<F, C, W>(&mut self, w: &mut W, c: WriteCommand, execute: F, close: C) -> Result
    where
        W: io::Write,
        F: Fn(&mut Self, &mut W) -> Result,
        C: Fn(&mut Self, &mut W) -> Result<()>,
    {
        self.command_queue.push(c);
        execute(self, w).and_then(|wrote| {
            if wrote {
                close(self, w).map(|_| true)
            } else {
                self.command_queue.pop();
                Ok(false)
            }
        })
    }

    /// Evaluate an expression tree and return the resulting formatted `String`
    pub fn evaluate<W: io::Write>(&mut self, exps: &Tree, w: &mut W) -> Result<()> {
        if self.allow_color {
            self.queue_str(if self.bash_prompt {
                "\u{01}\x1B[0m\u{02}"
            } else {
                "\x1B[0m"
            });
        }

        if self.interpret_tree(w, &exps, CompleteStyle::default())? && self.allow_color {
            if self.bash_prompt {
                write!(w, "\u{01}\x1B[0m\u{02}")?;
            } else {
                write!(w, "\x1B[0m")?;
            }
        }

        self.command_queue.clear();

        Ok(())
    }

    #[inline(always)]
    fn write_queue<W: io::Write>(&mut self, w: &mut W) -> Result<()> {
        for command in self.command_queue.drain(..) {
            use WriteCommand::*;
            match command {
                WriteString(s) => write!(w, "{}", s)?,
                WriteContext(c) => c.write_to(w, self.bash_prompt)?,
                WriteStr(s) => write!(w, "{}", s)?,
            }
        }

        Ok(())
    }

    fn interpret_tree<W: io::Write>(
        &mut self,
        w: &mut W,
        exps: &Tree,
        context: CompleteStyle,
    ) -> Result {
        use Expression::*;
        let mut wrote = false;
        let mut separator_count = 0;
        for e in &exps.0 {
            match e {
                Separator(s) => {
                    // Queue all separators if anything has been written in this
                    // tree so far
                    if wrote {
                        separator_count += 1;
                        self.queue_str(s.as_str());
                    }
                }
                e => {
                    // Clear separators between previous expression and the current
                    // one which was not written, to prevent accumulating separators
                    // between elements which were not supposed to have them
                    if self.interpret(w, &e, context)? {
                        wrote = true;
                    } else {
                        self.drain_queue(separator_count);
                        separator_count = 0;
                    }
                }
            }
        }
        self.drain_queue(separator_count);
        Ok(wrote)
    }

    fn interpret<W: io::Write>(
        &mut self,
        w: &mut W,
        exp: &Expression,
        ctx: CompleteStyle,
    ) -> Result {
        use ast::Expression::*;

        match exp {
            Named { name, ref sub } => self.interpret_named(w, *name, sub, ctx),
            Group { d, ref sub } => self.interpret_group(w, *d, sub, ctx),
            Format { ref style, ref sub } => self.interpret_format(w, *style, sub, ctx),
            Literal(ref literal) => {
                self.write_queue(w)?;
                write!(w, "{}", literal)?;
                Ok(true)
            }
            Separator(_) => unreachable!("Separator must be handled in tree interpreter"),
        }
    }

    fn interpret_group<W: io::Write>(
        &mut self,
        w: &mut W,
        d: Delimiter,
        sub: &Tree,
        style: CompleteStyle,
    ) -> Result {
        if sub.0.len() > 0 {
            self.with_command(
                w,
                WriteCommand::WriteStr(d.left()),
                |i, w| i.interpret_tree(w, &sub, style),
                |_, w| write!(w, "{}", d.right()).map_err(|e| e.into()),
            )
        } else {
            Ok(false)
        }
    }

    #[inline(always)]
    fn optional_prefix<W: io::Write, V1: fmt::Display + Empty, V2: fmt::Display>(
        &mut self,
        w: &mut W,
        sub: &Tree,
        val: V1,
        prefix: V2,
        ctx: CompleteStyle,
    ) -> Result {
        if val.is_empty() {
            return Ok(false);
        }

        self.write_queue(w)?;

        if sub.0.is_empty() {
            write!(w, "{}{}", prefix, val)?;
            return Ok(true);
        }

        if self.interpret_tree(w, sub, ctx)? {
            write!(w, "{}", val)?;
        } else {
            write!(w, "{}{}", prefix, val)?;
        }

        Ok(true)
    }

    #[inline(always)]
    fn interpret_literal<W: io::Write>(&mut self, w: &mut W, sub: &Tree, s: &str) -> Result {
        if sub.0.is_empty() {
            write!(w, "{}", s)?;
            Ok(true)
        } else {
            Err(InterpreterErr::UnexpectedArgs {
                exp: Expression::Named {
                    name: Name::Quote,
                    sub: sub.clone(),
                },
            })
        }
    }

    #[inline(always)]
    fn interpret_named<W: io::Write>(
        &mut self,
        w: &mut W,
        name: Name,
        sub: &Tree,
        ctx: CompleteStyle,
    ) -> Result {
        use ast::Name::*;
        match name {
            Branch => self.optional_prefix(w, sub, self.stats.branch.clone(), "", ctx),
            Remote => self.optional_prefix(w, sub, self.stats.remote.clone(), "", ctx),
            Ahead => self.optional_prefix(w, sub, self.stats.ahead, "+", ctx),
            Behind => self.optional_prefix(w, sub, self.stats.behind, "-", ctx),
            Conflict => self.optional_prefix(w, sub, self.stats.conflicts, "U", ctx),
            Added => self.optional_prefix(w, sub, self.stats.added_staged, "A", ctx),
            Untracked => self.optional_prefix(w, sub, self.stats.untracked, "?", ctx),
            Modified => self.optional_prefix(w, sub, self.stats.modified_staged, "M", ctx),
            Unstaged => self.optional_prefix(w, sub, self.stats.modified, "M", ctx),
            Deleted => self.optional_prefix(w, sub, self.stats.deleted, "D", ctx),
            DeletedStaged => self.optional_prefix(w, sub, self.stats.deleted_staged, "D", ctx),
            Renamed => self.optional_prefix(w, sub, self.stats.renamed, "R", ctx),
            Stashed => self.optional_prefix(w, sub, self.stats.stashes, "H", ctx),
            Quote => self.interpret_literal(w, sub, "'"),
        }
    }

    fn interpret_format<W: io::Write>(
        &mut self,
        w: &mut W,
        style: CompleteStyle,
        sub: &Tree,
        mut context: CompleteStyle,
    ) -> Result {
        let prev = context;
        context += style;

        self.with_command(
            w,
            WriteCommand::WriteContext(context),
            |i, w| i.interpret_tree(w, sub, context),
            |i, w| {
                prev.write_difference(w, &context, i.bash_prompt)
                    .map_err(|e| e.into())
            },
        )
    }
}

/// Trait which determines what is empty in the eyes of the Interpreter
///
/// The interpreter simply ignores the macros which correspond to "empty" values.
trait Empty {
    fn is_empty(&self) -> bool;
}

impl Empty for u16 {
    fn is_empty(&self) -> bool {
        *self == 0
    }
}

impl Empty for str {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl Empty for String {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<T> Empty for Vec<T> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::git::Stats;
    use ast;
    use ast::{Delimiter, Expression, Name, Tree};
    use proptest::arbitrary::any;
    use proptest::collection::vec;
    use proptest::strategy::Strategy;

    proptest! {
        #[test]
        fn empty_stats_empty_result(
            name in ast::arb_name()
                .prop_filter("Quote is never empty".to_owned(),
                             |n| *n != Name::Quote)
        ) {

            let stats: Stats = Default::default();

            let mut interpreter = Interpreter::new(stats, false, false);

            let exp = Expression::Named { name, sub: Tree::new() };

            let mut output = Vec::new();
            match interpreter.evaluate(&Tree(vec![exp.clone()]), &mut output) {
                Ok(()) => {
                    println!("interpreted {} as {} ({:?})", exp, String::from_utf8_lossy(&output), output);
                    assert!(output.is_empty())
                },
                Err(e) => {
                    println!("{:?}", e);
                    panic!("Error in proptest")
                }
            }
        }

        #[test]
        fn empty_group_empty_result(
            name in ast::arb_name()
                .prop_filter("Quote is never empty".to_owned(),
                             |n| *n != Name::Quote)
        ) {
            let stats = Stats::default();
            let interior = Expression::Named { name, sub: Tree::new(), };
            let exp = Expression::Group {
                d: Delimiter::Curly,
                sub: Tree(vec![interior]),
            };

            let mut interpreter = Interpreter::new(stats, false, false);

            let mut output = Vec::with_capacity(32);
            match interpreter.evaluate(&Tree(vec![exp.clone()]), &mut output) {
                Ok(()) => {
                    println!(
                        "interpreted {} as \"{}\" ({:?})",
                        exp,
                        String::from_utf8(output.clone()).unwrap(),
                        output
                    );
                    prop_assert!(output.is_empty());
                }
                Err(e) => {
                    println!("{:?} printing {}", e,  String::from_utf8(output).unwrap());
                    prop_assert!(false, "Failed to interpret tree");
                }
            }
        }

        #[test]
        fn empty_format_empty_result(
            name in ast::arb_name()
                .prop_filter("Quote is never empty".to_owned(),
                             |n| *n != Name::Quote),
            style in vec(ast::arb_style(), 1..10),
            bash_prompt in any::<bool>()
        ) {
            let stats = Stats::default();
            let interior = Expression::Named { name, sub: Tree::new(), };
            let exp = Expression::Format {
                style: style.iter().collect(),
                sub: Tree(vec![interior]),
            };

            let mut interpreter = Interpreter::new(stats, true, bash_prompt);
            let mut output = Vec::with_capacity(32);
            match interpreter.evaluate(&Tree(vec![exp.clone()]), &mut output) {
                Ok(()) => {
                    println!(
                        "interpreted {} as {} ({:?})",
                        exp,
                        String::from_utf8(output.clone()).unwrap(),
                        output
                    );
                    prop_assert!(output.is_empty());
                }
                Err(e) => {
                    println!("{:?} printing {}", e,  String::from_utf8(output.clone()).unwrap());
                    prop_assert!(false, "Failed to interpret tree");
                }
            }
        }
    }
}
