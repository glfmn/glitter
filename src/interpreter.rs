//! Interpreter which transforms expressions into the desired output

use git::Stats;
use ast::{Tree, Expression, Name};
use std::fmt;


/// The interpreter which transforms a gist expression using the provided stats
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Interpreter {
    stats: Stats,
}


pub enum InterpreterErr {
    UnexpectedArgs {
        exp: Expression,
    },
}

type InterpretResult = Result<String, InterpreterErr>;


/// Trait which determines what an empty result should be
trait Empty {
    fn is_empty(&self) -> bool;
}

impl Empty for u16 {
    fn is_empty(&self) -> bool { *self == 0 }
}

impl Empty for str {
    fn is_empty(&self) -> bool { self.is_empty() }
}

impl Empty for String {
    fn is_empty(&self) -> bool { self.is_empty() }
}

impl Interpreter {
    /// Create a new Interpreter with the given stats
    pub fn new(stats: Stats) -> Interpreter {
        Interpreter {
            stats: stats,
        }
    }

    ///
    pub fn evaluate(&self, exps: &Tree) -> InterpretResult {
        let mut res = String::new();
        for ref exp in exps.clone().0 {
            res.push_str(&self.interpret(exp)?);
        }
        Ok(res)
    }

    fn interpret(&self, exp: &Expression) -> InterpretResult {
        use ast::Expression::{Named, Group, Literal, Format};
        use ast::Name::*;

        let val = match exp {
            &Named { ref name, ref sub } => {
                match name {
                    &Backslash => self.interpret_literal(sub, "\\")?,
                    &Quote => self.interpret_literal(sub, "'")?,
                    &Branch => self.optional_prefix(sub, self.stats.branch.clone(), "")?,
                    &Remote => self.optional_prefix(sub, self.stats.remote.clone(), "")?,
                    &Ahead => self.optional_prefix(sub, self.stats.ahead, "+")?,
                    &Behind => self.optional_prefix(sub, self.stats.behind, "-")?,
                    &Conflict => self.optional_prefix(sub, self.stats.conflicts, "U")?,
                    &Added => self.optional_prefix(sub, self.stats.added_staged, "A")?,
                    &Untracked => self.optional_prefix(sub, self.stats.untracked, "?")?,
                    &Modified => self.optional_prefix(sub, self.stats.modified_staged, "M")?,
                    &Unstaged => self.optional_prefix(sub, self.stats.modified, "M")?,
                    &Deleted => self.optional_prefix(sub, self.stats.deleted, "D")?,
                    &DeletedStaged => self.optional_prefix(sub, self.stats.deleted_staged, "D")?,
                    &Renamed => self.optional_prefix(sub, self.stats.renamed, "R")?,
                    &Stashed => self.optional_prefix(sub, self.stats.stashes, "H")?,
                }
            },
            &Group { ref l, ref r, ref sub } if l == "g(" && r == ")" => {
                let sub = self.evaluate(sub)?;
                if sub.is_empty() {
                    String::new()
                } else {
                    format!("{}", sub)
                }
            }
            &Group { ref l, ref r, ref sub } => {
                let sub = self.evaluate(&sub)?;
                if sub.is_empty() {
                    String::new()
                } else {
                    format!("{}{}{}",l, sub, r)
                }
            },
            &Literal(ref literal) => literal.to_string(),
            &Format { style: _, ref sub } => {
                let sub = self.evaluate(&sub)?;
                if sub.is_empty() {
                    String::new()
                } else {
                    sub
                }
            },
        };

        Ok(val)
    }

    #[inline(always)]
    fn optional_prefix<V1: fmt::Display + Empty, V2: fmt::Display>(&self,
        sub: &Tree,
        val: V1,
        prefix: V2,
     ) -> InterpretResult {
        if val.is_empty() { return Ok(String::new()) };
        match sub.0.len() {
            0 => Ok(format!("{}{}", prefix, val)),
            _ => {
                Ok(format!("{}{}", self.evaluate(sub)?, val))
            }
        }
    }

    #[inline(always)]
    fn interpret_literal(&self, sub: &Tree, literal: &str) -> InterpretResult {
        match sub.0.len() {
            0 => Ok(literal.to_string()),
            _ => Err(InterpreterErr::UnexpectedArgs {
                exp: Expression::Named {
                    name: Name::Quote,
                    sub: sub.clone(),
                },
            }),
        }
    }
}


#[cfg(test)]
mod test {

    use super::*;
    use ast::{Name, Expression, Tree};
    use git::Stats;
    use quickcheck::TestResult;

    quickcheck! {
        fn empty_stats_empty_result(name: Name) -> TestResult {
            let stats: Stats = Default::default();

            let interpreter = Interpreter::new(stats);

            // Create valid expressions with empty arguments if arguments are necessary, discard
            // tests which represent illegal literal characters since they produe an output
            let exp = match name {
                Name::Quote | Name::Backslash => return TestResult::discard(),
                name @ _ => Expression::Named { name, sub: Tree::new() },
            };

            match interpreter.evaluate(&Tree(vec![exp.clone()])) {
                Ok(res) => {
                    println!("interpreted {} as {}", exp, res);
                    TestResult::from_bool(res.is_empty())
                },
                Err(_) => {
                    TestResult::discard()
                }
            }
        }
    }
}
