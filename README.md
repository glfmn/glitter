# Gist

A utility for pretty-printing meta information about `git` repositories, intended to facilitate making custom git prompts.

A format specifier is also called a gist expression.  Gist expressions come in three types:

1. Named expressions
2. Group Expressions
3. Literal Expressions

**Named expressions** take one of two forms: the plain form with no arguments, or with a list of arguments, comma seperated.

- `\name` plain form
- `\name(exp1,exp2,...,expn)` with expressions as arguments, comma separated.

**Group expressions** are set of expressions, which are not comma separated.  There are a few base group types:

- `\()` parentheses - wrap with parentheses
- `\{}` curly braces - wrap with curly braces
- `\[]` square brackets - wrap contents with square brackets
- `\<>` angle brackets - wrap contents with angle brackets
- `\g()` bare group - do not wrap contents with anything

The base of all gist expressions is an implicit bare group.  Thus, the following is a valid gist expression even though expressions are next to each-other without an explicit bare group.

```txt
\(\*(\b\B)\+\-)\[\A\M\D\R]\{\h('@')}'~'
```

By nesting groups of expressions, we can create an implicit tree of expressions.

A **literal expression** is any valid utf8 characters between single quites, except for single quotes and backslashes.

```txt
'hello''we''are''literal''expressions''I am one including whitespace''日本語で書いてもいい'
```
