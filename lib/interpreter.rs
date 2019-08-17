//! Interpreter which transforms expressions into the desired output

use crate::ast::{self, Expression, Name, Style, Tree};
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

type State = Result<Vec<StyledString>, InterpreterErr>;

/// The interpreter which transforms a gist expression using the provided stats
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Interpreter {
    stats: Stats,
    allow_color: bool,
    bash_prompt: bool,
}

impl Interpreter {
    /// Create a new Interpreter with the given stats
    pub fn new(stats: Stats, allow_color: bool, bash_prompt: bool) -> Interpreter {
        Interpreter {
            stats,
            allow_color,
            bash_prompt,
        }
    }

    /// Evaluate an expression tree and return the resulting formatted `String`
    pub fn evaluate<W: io::Write>(&self, exps: &Tree, w: &mut W) -> Result<(), InterpreterErr> {
        let mut prev_style = StyleContext::default();
        for chunk in self.interpret_tree(&exps, StyleContext::default())? {
            let (style, result) = chunk.into();
            if self.allow_color {
                style.write_difference(w, &prev_style, self.bash_prompt)?;
                prev_style = style;
            }

            w.write_all(result.as_bytes())?;
        }

        let reset = "\x1B[0m";
        if self.allow_color {
            if self.bash_prompt {
                write!(w, "\u{01}{}\u{02}", reset)?;
            } else {
                w.write_all(reset.as_bytes())?;
            }
        }

        Ok(())
    }

    fn interpret_tree(&self, exps: &Tree, context: StyleContext) -> State {
        let mut res = Vec::new();
        for e in exps.clone().0 {
            res.extend(self.interpret(&e, context)?);
        }
        Ok(res)
    }

    fn interpret(&self, exp: &Expression, ctx: StyleContext) -> State {
        use ast::Expression::{Format, Group, Literal, Named};

        let val = match exp {
            Named { ref name, ref sub } => self.interpret_named(*name, sub, ctx)?,
            Group {
                ref l,
                ref r,
                ref sub,
            } => {
                let sub = self.interpret_tree(&sub, ctx)?;
                if sub.is_empty() {
                    vec![]
                } else {
                    let mut res = Vec::with_capacity(sub.len() + 2);
                    res.push(StyledString::new(ctx, l.to_string()));
                    res.extend(sub);
                    res.push(StyledString::new(ctx, r.to_string()));
                    res
                }
            }
            Literal(ref literal) => vec![StyledString::new(ctx, literal.to_string())],
            Format { ref style, ref sub } => self.interpret_format(style, sub, ctx)?,
        };

        Ok(val)
    }

    #[inline(always)]
    fn optional_prefix<V1: fmt::Display + Empty, V2: fmt::Display>(
        &self,
        sub: &Tree,
        val: V1,
        prefix: V2,
        ctx: StyleContext,
    ) -> State {
        if val.is_empty() {
            return Ok(Vec::new());
        };
        match sub.0.len() {
            0 => Ok(vec![StyledString::new(ctx, format!("{}{}", prefix, val))]),
            _ => {
                let mut res = Vec::with_capacity(sub.0.len() + 1);
                res.extend(self.interpret_tree(sub, ctx)?);
                res.push(StyledString::new(ctx, val.to_string()));
                Ok(res)
            }
        }
    }

    #[inline(always)]
    fn interpret_literal(
        &self,
        sub: &Tree,
        literal: &str,
        context: StyleContext,
    ) -> Result<StyledString, InterpreterErr> {
        match sub.0.len() {
            0 => Ok(StyledString::new(context, literal.to_string())),
            _ => Err(InterpreterErr::UnexpectedArgs {
                exp: Expression::Named {
                    name: Name::Quote,
                    sub: sub.clone(),
                },
            }),
        }
    }

    #[inline(always)]
    fn interpret_named(&self, name: Name, sub: &Tree, ctx: StyleContext) -> State {
        use ast::Name::*;
        match name {
            Branch => self.optional_prefix(sub, self.stats.branch.clone(), "", ctx),
            Remote => self.optional_prefix(sub, self.stats.remote.clone(), "", ctx),
            Ahead => self.optional_prefix(sub, self.stats.ahead, "+", ctx),
            Behind => self.optional_prefix(sub, self.stats.behind, "-", ctx),
            Conflict => self.optional_prefix(sub, self.stats.conflicts, "U", ctx),
            Added => self.optional_prefix(sub, self.stats.added_staged, "A", ctx),
            Untracked => self.optional_prefix(sub, self.stats.untracked, "?", ctx),
            Modified => self.optional_prefix(sub, self.stats.modified_staged, "M", ctx),
            Unstaged => self.optional_prefix(sub, self.stats.modified, "M", ctx),
            Deleted => self.optional_prefix(sub, self.stats.deleted, "D", ctx),
            DeletedStaged => self.optional_prefix(sub, self.stats.deleted_staged, "D", ctx),
            Renamed => self.optional_prefix(sub, self.stats.renamed, "R", ctx),
            Stashed => self.optional_prefix(sub, self.stats.stashes, "H", ctx),
            Backslash => Ok(vec![self.interpret_literal(sub, "\\", ctx)?]),
            Quote => Ok(vec![self.interpret_literal(sub, "'", ctx)?]),
        }
    }

    fn interpret_format(&self, style: &[Style], sub: &Tree, mut context: StyleContext) -> State {
        context.extend(style);
        self.interpret_tree(sub, context)
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

impl Empty for StyledString {
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
    use ast::{Expression, Name, Tree};
    use proptest::strategy::Strategy;

    proptest! {
        #[test]
        fn empty_stats_empty_result(
            name in ast::arb_name()
                .prop_filter("Backslash is never empty".to_owned(),
                             |n| *n != Name::Backslash)
                .prop_filter("Quote is never empty".to_owned(),
                             |n| *n != Name::Quote)
        ) {

            let stats: Stats = Default::default();

            let interpreter = Interpreter::new(stats, false, false);

            let exp = Expression::Named { name, sub: Tree::new() };

            let mut output = Vec::new();
            match interpreter.evaluate(&Tree(vec![exp.clone()]), &mut output) {
                Ok(()) => {
                    println!("interpreted {} as {} ({:?})", exp, String::from_utf8_lossy(&output), output);
                    assert!(output.is_empty())
                },
                Err(e) => {
                    println!("{:?}", e);
                    ()
                }
            }
        }
    }
}
