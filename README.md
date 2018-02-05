# Gist

![](img/gist-demo.gif)

**A domain-specific language for printing git stats in custom formats.**

# Installation

## Quick Start

As long as you have the rust tool-chain set up, installing is as easy as:

```
$ cargo install gist-i
```

You can download the rust toolchain at [rustup.rs](http://rustup.rs/).

Basic usage for `gist-i` is:

```
$ gist-i <FORMAT>
```

Learn more and get help with:

```
$ gist-i help
```

Too add a gist format to your shell prompt if you are in a bash shell, add the following snippet to your `~/.bashrc`:

```bash
__is_git_repo() {
    # return 128 if not in git repository, return 0 if in repo
    git status -sb --porcelain &>/dev/null
    echo "$?"
}

__set_prompt() {
    local EXIT="$?"
    # Capture last command exit flag first

    # Your gist format
    local fmt="\<#m;*(\b)#m(\B(#~('..')))\(#g(\+)#r(\-))>\[#g;*(\M\A\R\D)#r;*(\m\a\u\d)]\{#m;*;_(\h('@'))}"

    # If color support exists, set color values, otherwise set them as empty
    if [ -x /usr/bin/tput ] && tput setaf 1 >&/dev/null; then
      # We have color support; assume it's compliant with Ecma-48
      # (ISO/IEC-6429). (Lack of such support is extremely rare, and such
      # a case would tend to support setf rather than setaf.)
      local nc="\[\033[0m\]"
      local red="\[\033[00;31m\]"
      local grn="\[\033[00;32m\]"
      local ylw="\[\033[00;33m\]"
      local blu="\[\033[00;34m\]"
      local bgrn="\[\033[01;32m\]"
      local bylw="\[\033[01;33m\]"
      local bblu="\[\033[01;34m\]"
      local bpur="\[\033[01;35m\]"
    fi

    # Clear out prompt
    PS1=""

    # If the last command didn't exit 0, display the exit code
    [ "$EXIT" -ne "0" ] && PS1+="$red$EXIT$nc "

    # identify debian chroot, if one exists
    if [ -z "${debian_chroot:-}" ] && [ -r /etc/debian_chroot ]; then
      PS1+="${debian_chroot:+($(cat /etc/debian_chroot))}"
    fi

    if [ "$(__is_git_repo)" -eq "0" ]; then
      local stats="$(gist-i $fmt)"
      PS1+="$stats:$bylw\w$nc\n\$ "
    else
      PS1+="$bgrn\u$grn@\h$nc:$bblu\w$nc\$ "
    fi
}

export PROMPT_COMMAND=__set_prompt
```

Where the variable **fmt** contains your gist format.  Here are a few examples you might want to try out on your system.

| Example `fmt`                                                                                              | Result                                             |
|:-----------------------------------------------------------------------------------------------------------|:---------------------------------------------------|
| `"\<#m;*(\b)#m(\B(#~('..')))\(#g(\+)#r(\-))>\[#g;*(\M\A\R\D)#r;*(\m\a\u\d)]\{#m;*;_(\h('@'))}"`            | ![long example gist](img/example-1.png)            |
| `"\(#m;*(\b)#g(\+)#r(\-))\[#g(\M\A\R\D)#r(\m\a\u\d)]\{#m;_(\h('@'))}':'"`                                  | ![short example gist](img/example-2.png)           |
| `"#g;*(\b)#y(\B(#~('..')))\[#g(\+(#~('ahead ')))]\[#r(\-(#~('behind ')))]' '#g;_(\M\A\R\D)#r;_(\m\a\u\d)"` | ![`git status sb` example gist](img/example-3.png) |

## Background

Most shells provide the ability to customize the shell prompt which appears before every command.  On my system, the default looks like:

```
gwen@tpy12:~/Documents/dev/util/gist$
```

Its intended to provide useful information about your shell.  However, it normally does not include information about git repositories, requiring the near constant use of `git status` to understand the state of the repository.  The solution is to set a prompt command and dynamically update your shell with the information you want.  `gist` is made for precisely this purpose: you can provide a format, and gist will interpret it, inserting the information in the format you want.

## Making your own gist format

An example format looks like:`"\<\b\(\+\-)>\[\M\A\R\D':'\m\a\u\d]\{\h('@')}':'"` results in something that might look like `<master(+1)>[M1:D3]{@5}:` where

- `master` is the name of the current branch.
- `+1`: means we are 1 commit ahead of the remote branch
- `M1`: the number of staged modifications
- `D3`: is the number of unstaged deleted files
- `@5`: is the number of stashes

`gist` expressions also support inline format expressions to do things like making text red, or bold, or using ANSI terminal escape sequences, or setting RGB colors for your git information.

`gist-i` will only accept your format string if your current directory is a **git repository**.

`gist` expressions have four components:

1. Named expressions
2. Format expressions
3. Group expressions
4. Literals

### Literals

Any characters between single quotes literal, except for backslashes and single quotes. Literals are left untouched.  For example, `'literal'` outputs `literal`.

```
$ gist-i "'hello world'"
```

### Named expressions

Named expressions represent information about your git repository.

| Name  | Meaning                        | Example         |
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
| `\h`  | # of stashed files             | `H1`            |

You can provide other expressions as arguments to expressions which replace the default prefix which appears before the result or file count.  For example, `\h('@')` will output `@3`
instead of `H3` if your repository has 3 stashed files.  You can provide an arbitrary number of valid expressions as a prefix to another named expression.

```
$ gist-i "\b"
$ gist-i "\b('on branch ')"
```

Expressions generally only render any output if their corresponding values aren't empty; in other words, if there are no added files, `gist-i` will not produce `A0` as the output of `\A`.

### Group Expressions

Gist will surround grouped expressions with parentheses or brackets, and will print nothing if the group is empty.

| Macro       | Result                           |
|:------------|:---------------------------------|
| `\[]`       | empty                            |
| `\()`       | empty                            |
| `\<>`       | empty                            |
| `\{}`       | empty                            |
| `\{\b}`     | `{master}`                       |
| `\<\+\->`   | `<+1-1>`                         |
| `\[\M\A\R]` | `[M1A3]` where `\R` is empty     |
| `\[\r\(\a)]`| empty, when `\r`, `\a` are empty |

```
$ gist-i "\b\<\M>"
```

### Format Expressions

Gist expressions support ANSI terminal formatting through the following styles:

| Format               | Meaning                     |
|:---------------------|:----------------------------|
| `#~(`...`)`          | reset                       |
| `#_(`...`)`          | underline                   |
| `#i(`...`)`          | italic text                 |
| `#*(`...`)`          | bold text                   |
| `#r(`...`)`          | red text                    |
| `#g(`...`)`          | green text                  |
| `#b(`...`)`          | blue text                   |
| `#m(`...`)`          | magenta/purple text         |
| `#y(`...`)`          | yellow text                 |
| `#w(`...`)`          | white text                  |
| `#k(`...`)`          | bright black text           |
| `#[01,02,03](`...`)` | 24 bit RGB text color       |
| `#R(`...`)`          | red background              |
| `#G(`...`)`          | green background            |
| `#B(`...`)`          | blue background             |
| `#M(`...`)`          | magenta/purple background   |
| `#Y(`...`)`          | yellow background           |
| `#W(`...`)`          | white background            |
| `#K(`...`)`          | bright black background     |
| `#{01,02,03}(`...`)` | 24 bit RGB background color |
| `#01(`...`)`         | Fixed terminal color        |

Format styles can be combined in a single expression by separating them with semicolons:

| Format         | Meaning                        |
|:---------------|:-------------------------------|
| `#w;K(`...`)`  | white text, black background   |
| `#r;*(`...`)`  | red bold text                  |
| `#42(`...`)`   | a forest greenish color        |
| `#_;*(`...`)`  | underline bold text            |

```
$ gist-i "#r;*('hello world')"
$ gist-i "#g;*(\b)"
$ gist-i "#[255,175,52]('orange text')"
$ gist-i "#G('green background')"
```

`gist` can understand and respects complicated nested styles, providing maximum flexibility.

```
$ gist-i "#g('green text with some '#*('bold')' green text')"
$ gist-i "#g;*(\b(#~('on branch ')))"
```
