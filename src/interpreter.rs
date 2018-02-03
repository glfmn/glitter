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
    ArgErr {
        exp: Expression,
        reason: ArgErr,
    },
}


pub enum ArgErr {
    UnexpectedArgs,
    UnexpectedCount {
        expected: u8,
        found: u8,
    },
    InvalidArg(Expression),
}

type InterpretResult = Result<String, InterpreterErr>;

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
            &Named { ref name, ref args } => {
                match name {
                    &Backslash => self.interpret_backslash(args)?,
                    &Quote => self.interpret_quote(args)?,
                    name @ &Branch => self.optional_prefix(args, *name, self.stats.branch.clone(), "")?,
                    name @ &Remote => self.optional_prefix(args, *name, self.stats.remote.clone(), "")?,
                    name @ &Ahead => self.optional_prefix(args, *name, self.stats.ahead, "+")?,
                    name @ &Behind => self.optional_prefix(args, *name, self.stats.behind, "-")?,
                    name @ &Conflict => self.optional_prefix(args, *name, self.stats.conflicts, "U")?,
                    name @ &Added => self.optional_prefix(args, *name, self.stats.added_staged, "A")?,
                    name @ &Untracked => self.optional_prefix(args, *name, self.stats.untracked, "?")?,
                    name @ &Modified => self.optional_prefix(args, *name, self.stats.modified_staged, "M")?,
                    name @ &Unstaged => self.optional_prefix(args, *name, self.stats.modified, "M")?,
                    name @ &Deleted => self.optional_prefix(args, *name, self.stats.deleted, "D")?,
                    name @ &DeletedStaged => self.optional_prefix(args, *name, self.stats.deleted_staged, "D")?,
                    name @ &Renamed => self.optional_prefix(args, *name, self.stats.renamed, "R")?,
                    name @ &RenamedStaged => self.optional_prefix(args, *name, self.stats.renamed, "R")?,
                    name @ &Stashed => self.optional_prefix(args, *name, self.stats.stashes, "H")?,
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
            &Format { ref style, ref sub } => {
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
        args: &Option<Vec<Expression>>,
        name: Name,
        val: V1,
        prefix: V2,
     ) -> InterpretResult {
        if val.is_empty() { return Ok(String::new()) };
        match args {
            &None => Ok(format!("{}{}", prefix, val)),
            &Some(ref args) if args.len() == 1 => {
                Ok(format!("{}{}", self.interpret(&args[0])?, val))
            },
            &Some(ref args) => Err(InterpreterErr::ArgErr {
                exp: Expression::Named {
                    name: name,
                    args: Some(args.to_vec())
                },
                reason: ArgErr::UnexpectedCount{ expected: 1, found: args.len() as u8 },
            }),
        }
    }

    #[inline(always)]
    fn interpret_backslash(&self, args: &Option<Vec<Expression>>) -> InterpretResult {
        match args {
            &None => Ok("\\".to_string()),
            &Some(_) => Err(InterpreterErr::ArgErr {
                exp: Expression::Named {
                    name: Name::Backslash,
                    args: args.clone(),
                },
                reason: ArgErr::UnexpectedArgs,
            }),
        }
    }

    #[inline(always)]
    fn interpret_quote(&self, args: &Option<Vec<Expression>>) -> InterpretResult {
        match args {
            &None => Ok("'".to_string()),
            &Some(_) => Err(InterpreterErr::ArgErr {
                exp: Expression::Named {
                    name: Name::Quote,
                    args: args.clone(),
                },
                reason: ArgErr::UnexpectedArgs,
            }),
        }
    }

    #[inline(always)]
    fn interpret_newline(&self, args: &Option<Vec<Expression>>) -> InterpretResult {
        match args {
            &None => Ok("\n".to_string()),
            &Some(_) => Err(InterpreterErr::ArgErr {
                exp: Expression::Named {
                    name: Name::Quote,
                    args: args.clone(),
                },
                reason: ArgErr::UnexpectedArgs,
            }),
        }
    }

    #[inline(always)]
    fn interpret_tab(&self, args: &Option<Vec<Expression>>) -> InterpretResult {
        match args {
            &None => Ok("\t".to_string()),
            &Some(_) => Err(InterpreterErr::ArgErr {
                exp: Expression::Named {
                    name: Name::Quote,
                    args: args.clone(),
                },
                reason: ArgErr::UnexpectedArgs,
            }),
        }
    }
}


#[cfg(test)]
mod test {

    use super::*;
    use ast::{Name, Expression, Tree, Style};
    use quickcheck::TestResult;

    quickcheck! {

        fn empty_stats_empty_result(name: Name) -> TestResult {
            let stats: ::git::Stats = Default::default();

            let interpreter = Interpreter::new(stats);

            // Create valid expressions with empty arguments if arguments are necessary, and
            // replace expressions which always return a value with empty literals
            let exp = match name {
                Name::Quote | Name::Backslash => return TestResult::discard(),
                name @ _ => Expression::Named { name, args: None },
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
