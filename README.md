
# ✨ glitter

![Glit is an informative shell prompt](https://raw.githubusercontent.com/glfmn/glitter/develop/img/glit-demo.gif)

**Git status summary with custom formats, perfect for your shell prompt**

[![Crates.io](https://img.shields.io/crates/v/glit.svg)](https://crates.io/crates/glit)[![Build Status](https://travis-ci.org/glfmn/glitter.svg?branch=master)](https://travis-ci.org/glfmn/glitter)

Glitter is a cross-platform command-line tool and format language for making informative git prompts.  Glitter's interpreter, `glit` will:

- Read status information from your git repository through the git api
- Parse and Interpret the provided format
- Output your format with the requested information to `stdout`

Glitter is a binary tool which affords blazing speed, offers maximum flexibility, and painless installation.  This makes Glitter an ideal alternative to alternative to tools like [`bash-git-prompt`](https://github.com/magicmonty/bash-git-prompt), [``zsh-git-prompt`](https://github.com/olivierverdier/zsh-git-prompt), and [`pos-git`](https://github.com/dahlbyk/posh-git).

Glitter has been tested on Windows, Mac, and Ubuntu; it works in Powershell, zsh, bash, and theoretically any shell environment that supports a prompt command.

# Installation

Go to the [release](https://github.com/glfmn/glitter/releases) page and download a binary for your platform.

To make sure Glitter is installed:

```
$ glit "'hello from git'" -e "'hello'"
```

It will output `hello from git` if the current directory is a git directory and `hello` if it is not.

### Build from source

Install the [rust toolchain](https://rustup.rs), `cmake` and `openssl` first, and then:

```
$ cargo install glit
```

## Setting up your shell

Once Glitter is installed, you need to set it to update your prompt.

### Bash

Add the following snippet to your `~/.bashrc`:

```bash
# Format to use inside of git repositories or their sub-folders
export GIT_FMT="\[#g;*(\b)#r(\B(#~(' ⇒ ')))#w(\(#~;*(\+('↑')\-('↓')))\<#g(\M\A\R\D)#r(\m\a\u\d)>\{#m;*;_(\h('@'))})]' '#b;*('\w')'\n '"

# Format to use outside of git repositories
export PS1_FMT="#g(#*('\u')'@\h')':'#b;*('\w')'\$ '"

__set_prompt() {
    PS1="$(glit "$GIT_FMT" -b -e "$PS1_FMT")"
}

export PROMPT_COMMAND=__set_prompt
```

### Powershell

Add the following snippet to your `$PROFILE`:

```ps
# Format to use inside of git repositories
$GIT_FMT="#y(\[#c;*(\b)#c(\B(#~(' ')))#w(\(#~;*(\+\-))\[#g(\M\A\R\D)#r(\m\a\u\d)]\{#m;*;_(\h('@'))})])"

function prompt {
    $path = $(get-location)
    glit "'$path'$GIT_FMT'> '" -e "'$path> '"
}
```

### zsh

Add the following snippet to your `~/.zshrc` file:

```sh
# Format used in a git repository
export GIT_FMT="\[#g;*(\b)#r(\B(#~(' ⇒ ')))#w(\(#~;*(\+('↑')\-('↓')))\<#g(\M\A\R\D)#r(\m\a\u\d)>\{#m;*;_(\h('@'))})]' '#b;*('%~')"

# Fallback format used outside of git repositories
export PS1_FMT="#g;*('%m')#b;*('%~')"

precmd() { print -rP "$(glit "$GIT_FMT" -b -e "$PS1_FMT")" }
PROMPT="%# "
```

# Customizing your format

Glitter provides a flexible expression language which is easy to use and easy to prototype with:

![`glit` is easy to experiment with.](img/glit-command.gif)

| Example `fmt`                                                                                                | Result                                                |
| :----------------------------------------------------------------------------------------------------------- | :---------------------------------------------------- |
| `"\<#m;*(\b)#m(\B(#~('..')))\(#g(\+)#r(\-))>\[#g;*(\M\A\R\D)#r;*(\m\a\u\d)]\{#m;*;_(\h('@'))}"`              | ![long example glitter](img/example-1.png)            |
| `"\(#m;*(\b)#g(\+)#r(\-))\[#g(\M\A\R\D)#r(\m\a\u\d)]\{#m;_(\h('@'))}':'"`                                    | ![short example glitter](img/example-2.png)           |
| `"#g;*(\b)#y(\B(#~('..')))\[#g(\+(#~('ahead ')))]\[#r(\-(#~('behind ')))]' '#g;_(\M\A\R\D)#r;_(\m\a\u\d)"`   | ![`git status sb` example glitter](img/example-3.png) |

A glitter format is made of 4 types of expressions:

- Informational expressions
- Group expressions
- Literals
- Format expressions

### Git Information

| Expression | Meaning                        | Example         |
|:------|:-------------------------------|:----------------|
| `\b`  | branch name or head commit id  | `master`        |
| `\B`  | remote name                    | `origin/master` |
| `\+`  | # of commits ahead remote      | `+1`            |
| `\-`  | # of commits behind remote     | `-1`            |
| `\m`  | # of unstaged modified files   | `M1`            |
| `\a`  | # of untracked files           | `?1`            |
| `\d`  | # of unstaged deleted files    | `D1`            |
| `\u`  | # of merge conflicts           | `U1`            |
| `\M`  | # of staged modified files     | `M1`            |
| `\A`  | # of added files               | `A1`            |
| `\R`  | # of renamed files             | `R1`            |
| `\D`  | # of staged deleted files      | `D1`            |
| `\h`  | # of stashed changes           | `H1`            |

You can provide other expressions as arguments to expressions which replace the default prefix which appears before the result or file count.  For example, `\h('@')` will output `@3`
instead of `H3` if your repository has 3 stashed files.  You can provide an arbitrary number of valid expressions as arguments to any of these expressions.

```
$ glit "\b"
$ glit "\b('on branch ')"
```

Expressions generally only render any output if their corresponding values aren't empty; in other words, if there are no added files, `glit` will not produce `A0` as the output of `\A`, but instead will output an empty string.

### Grouping

Glitter will surround grouped expressions with parentheses or brackets, and will print nothing if the group is empty.

| Macro       | Result                       |
|:------------|:-----------------------------|
| `\[]`       | empty                        |
| `\()`       | empty                        |
| `\<>`       | empty                        |
| `\{}`       | empty                        |
| `\{\b}`     | `{master}`                   |
| `\<\+\->`   | `<+1-1>`                     |
| `\[\M\A\R]` | `[M1A3]` where `\R` is 0     |
| `\[\r\(\a)]`| empty, when `\r`, `\a` are 0 |

```
$ glit "\b\<\M>"
```

### Literals

Any characters between single quotes are literals. Literals are left untouched.  For example, `'literal'` outputs `literal`.

```
$ glit "'hello world'"
$ glit "'\n\w\n\u'"
$ glit "'separate'' ''words'"
```

### Formatting text

Glitter expressions support ANSI terminal formatting through the following styles:

| Format                   | Meaning                     |
|:-------------------------|:----------------------------|
| `#~('...')`          | reset                       |
| `#_('...')`          | underline                   |
| `#i('...')`          | italic text                 |
| `#*('...')`          | bold text                   |
| `#r('...')`          | red text                    |
| `#g('...')`          | green text                  |
| `#b('...')`          | blue text                   |
| `#m('...')`          | magenta/purple text         |
| `#y('...')`          | yellow text                 |
| `#w('...')`          | white text                  |
| `#k('...')`          | bright black text           |
| `#[01,02,03]('...')` | 24 bit RGB text color       |
| `#R('...')`          | red background              |
| `#G('...')`          | green background            |
| `#B('...')`          | blue background             |
| `#M('...')`          | magenta/purple background   |
| `#Y('...')`          | yellow background           |
| `#W('...')`          | white background            |
| `#K('...')`          | bright black background     |
| `#{01,02,03}('...')` | 24 bit RGB background color |
| `#01('...')`         | Fixed terminal color        |

Format styles can be combined in a single expression by separating them with semicolons:

| Format             | Meaning                        |
|:-------------------|:-------------------------------|
| `#w;K('...')`  | white text, black background   |
| `#r;*('...')`  | red bold text                  |
| `#42('...')`   | a forest greenish color        |
| `#_;*('...')`  | underline bold text            |

```
$ glit "#r;*('hello world')"
$ glit "#g;*(\b)"
$ glit "#[255,175,52]('orange text')"
$ glit "#G('green background')"
```

`glit` can understand and respects complicated nested styles, providing maximum flexibility.

```
$ glit "#g('green text with some '#*('bold')' green text')"
$ glit "#g;*(\b(#~('on branch ')))"
```
