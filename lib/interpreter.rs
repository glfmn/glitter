//! Interpreter which transforms expressions into the desired output

use crate::ast::{self, CompleteStyle, Expression, Name, Tree};
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

type State = Result<(CompleteStyle, bool), InterpreterErr>;

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

    /// Evaluate an expression tree and return the resulting formatted `String`
    pub fn evaluate<W: io::Write>(&mut self, exps: &Tree, w: &mut W) -> Result<(), InterpreterErr> {
        if self.allow_color {
            if self.bash_prompt {
                self.command_queue
                    .push(WriteCommand::WriteStr("\u{01}\x1B[0m\u{02}"));
            } else {
                self.command_queue.push(WriteCommand::WriteStr("\x1B[0m"));
            }
        }

        let (_, wrote) = self.interpret_tree(w, &exps, CompleteStyle::default())?;

        if wrote && self.allow_color {
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
    fn write_queue<W: io::Write>(&mut self, w: &mut W) -> Result<(), InterpreterErr> {
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
    ) -> State {
        let mut wrote = false;
        let mut separator_count = 0;
        for e in &exps.0 {
            use Expression::*;
            match e {
                Separator(s) => {
                    // Queue all separators if anything has been written in this
                    // tree so far
                    if wrote {
                        separator_count += 1;
                        self.command_queue.push(WriteCommand::WriteStr(s.as_str()));
                    }
                }
                e => {
                    let (_, wrote_now) = self.interpret(w, &e, context)?;
                    // Clear separators between previous expression and the current
                    // one which was not written, to prevent accumulating separators
                    // between elements which were not supposed to have them
                    if !wrote_now {
                        self.drain_queue(separator_count);
                        separator_count = 0;
                    }
                    wrote |= wrote_now;
                }
            }
        }
        self.drain_queue(separator_count);
        Ok((context, wrote))
    }

    fn interpret<W: io::Write>(
        &mut self,
        w: &mut W,
        exp: &Expression,
        ctx: CompleteStyle,
    ) -> State {
        use ast::Expression::*;

        match exp {
            Named { ref name, ref sub } => self.interpret_named(w, *name, sub, ctx),
            Group { ref d, ref sub } => {
                if sub.0.len() > 0 {
                    let len = self.command_queue.len();
                    self.command_queue.push(WriteCommand::WriteStr(d.left()));
                    if let (_, true) = self.interpret_tree(w, &sub, ctx)? {
                        write!(w, "{}", d.right())?;
                        Ok((ctx, true))
                    } else {
                        while self.command_queue.len() > len {
                            self.command_queue.pop();
                        }
                        Ok((ctx, false))
                    }
                } else {
                    Ok((ctx, false))
                }
            }
            Literal(ref literal) => {
                self.write_queue(w)?;
                write!(w, "{}", literal)?;
                Ok((ctx, true))
            }
            Format { ref style, ref sub } => self.interpret_format(w, *style, sub, ctx),
            Separator(_) => unreachable!("Separator must be handled in tree interpreter"),
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
    ) -> State {
        if val.is_empty() {
            return Ok((ctx, false));
        }

        self.write_queue(w)?;

        match sub.0.len() {
            0 => write!(w, "{}{}", prefix, val)?,
            _ => {
                let (_, wrote) = self.interpret_tree(w, sub, ctx)?;
                if wrote {
                    write!(w, "{}", val)?;
                } else {
                    write!(w, "{}{}", prefix, val)?;
                }
            }
        }
        Ok((ctx, true))
    }

    #[inline(always)]
    fn interpret_literal<W: io::Write>(
        &mut self,
        w: &mut W,
        sub: &Tree,
        literal: &str,
        context: CompleteStyle,
    ) -> State {
        match sub.0.len() {
            0 => {
                write!(w, "{}", literal)?;
                Ok((context, true))
            }
            _ => Err(InterpreterErr::UnexpectedArgs {
                exp: Expression::Named {
                    name: Name::Quote,
                    sub: sub.clone(),
                },
            }),
        }
    }

    #[inline(always)]
    fn interpret_named<W: io::Write>(
        &mut self,
        w: &mut W,
        name: Name,
        sub: &Tree,
        ctx: CompleteStyle,
    ) -> State {
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
            Quote => self.interpret_literal(w, sub, "'", ctx),
        }
    }

    fn interpret_format<W: io::Write>(
        &mut self,
        w: &mut W,
        style: CompleteStyle,
        sub: &Tree,
        mut context: CompleteStyle,
    ) -> State {
        let prev = context;
        let len = self.command_queue.len();

        context += style;
        self.command_queue.push(WriteCommand::WriteContext(context));
        if let (_, true) = self.interpret_tree(w, sub, context)? {
            prev.write_difference(w, &context, self.bash_prompt)?;
            Ok((context, true))
        } else {
            while self.command_queue.len() > len {
                self.command_queue.pop();
            }
            Ok((context, false))
        }
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
