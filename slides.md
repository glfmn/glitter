---
title: Gist
separator: \n----\n
verticalSeparator: \n---\n
theme: white
revealOptions:
    transition: 'fade'
---

# Gist

Gwen Lofman

----

I use git from the commandline

----

Problem: I **spam** `git status`

----

Solution: I can embed status info in my shell prompt

----

For Exmample:

![example long gist](img/gist-example-1.png)

----

## Attempt 1

----

```sh
__is_git_repo() {
  # return 128 if not in git repository, return 0 if in repo
  git status -sb --porcelain &>/dev/null
  echo "$?"
}

__formatter() {
  local count="$(git status --porcelain | grep -ce "$4")"
  local formatted="$5"
  [ "$count" -lt "1" ] && count=""
  formatted="${formatted//\%$1/$count}"
  formatted="${formatted//\%$2/$([ -z $count ] || echo $3$count)}"
  echo "$formatted"
}

__git_stats() { # (format)
  # Check if stats is in a git repository
  [ "$(__is_git_repo)" -ne 0 ] && echo "not a git repository" && return 128

  # Echo usage if no argument specified
  [ -z "$1" ] && echo -e "\033[1;37musage: __git_stats [format]\033[0m
  format parameters:
  \t%b\tbranch name
  \t%a\tnumber of added files
  \t%A\tnumber of added files with an A in front
  \t%d\tnumber of deleted files
  \t%D\tnumber of deleted files with a D in front
  \t%m\tnumber of modified files, or number of files with unstaged changes
  \t%M\tnumber of modified files with an M in front
  \t%r\tnumber of renamed files
  \t%R\tnumber of renamed files with an R in front
  \t%s\tnumber of staged files with changes
  \t%S\tnumber of staged files with an M in front
  \t%u\tnumber of untracked files
  \t%U\tnumber of untracked files with a ? in front
  \t%x\tnumber of confilcts on a merge
  \t%X\tnumber of confilcts on a merge with a UU in front
  \t%+\tnumber of commits ahead of tracking branch with + as prefix
  \t%-\tnumber of commits behind tracking branch with - as prefix

  \texample: __git_stats \"(%b%+%-)[%u%m%d%R]\"
  \t__git_stats will clear out any empty braces, specifically <>,
  \t\t(), [], and {}" && return 2

  local format="$1"
  local st="$(git status --porcelain)"
  local branch="$(git symbolic-ref --short HEAD)"
  local upstream="$(git for-each-ref --format='%(upstream:short)' $(git symbolic-ref -q HEAD))"

  # Get the current branch and replace branch format specifier
  format="${format//\%b/$branch}"

  format="$(__formatter "m" "M" "M" "^.M" $format)"
  format="$(__formatter "s" "S" "M" "^M" $format)"
  format="$(__formatter "a" "A" "A" "^A" $format)"
  format="$(__formatter "r" "R" "R" "^R" $format)"
  format="$(__formatter "u" "U" "?" "^?" $format)"
  format="$(__formatter "d" "D" "D" "^D" $format)"
  format="$(__formatter "x" "X" "UU" "^UU" $format)"

  if git rev-parse --abbrev-ref --symbolic-full-name @{u} &>/dev/null; then
    local ahead="$(git rev-list --left-right $upstream..$branch | grep -c '>')"
    [ "$ahead" -gt "0" ] && format="${format//\%+/+$ahead}"

    local behind="$(git rev-list --left-right $branch..$upstream | grep -c '>')"
    [ "$behind" -gt "0" ] && format="${format//\%-/-$behind}"
  fi
  format="${format//\%+/}"
  format="${format//\%-/}"

  # Clear out empty braces if substitutions have resulted in empty braces
  format="${format//()/}"
  format="${format//<>/}"
  format="${format//[]/}"
  format="${format//\{\}/}"

  echo "$format"
}

```

----

Slow, and inflexible

----

# Attempt 2

----

Write and interpreted language: **Gist**

----

- Implemented in Rust
- Supports basic information like branch name and modified files

----

- Does not support terminal formatting
- Unecessarily complicated abstract syntax tree
- Not quite as ergonomic as possible

----

# Attempt 3: MangoHacks

----

Here at MangoHacks I finalized **Gist**

----

- Simplify abstract syntax tree
- De-dubpe the interpreter and parser
- Add format expressions to abstract syntax tree
- Implement format expressions in the interpreter

----

## Examples

```
gist"\<#m;*(\b)\B('..')\(#g(\+)#r(\-))>\[#g(\M\A\R\D)#r;i(\m\a\u\d)]\{#m;_(\h('@'))}"
```

```
gist "\(#m;*(\b)#g(\+)#r(\-))\[#g(\M\A\R\D)#r(\m\a\u\d)]\{#m;_(\h('@'))}':'"
```

----

## Future Work

----

1. Provide more helpful syntax errors
2. Don't fail in an ugly way on integer overflow
3. Add more subcommands to allow previewing format specifiers
4. Reduce unecessary copies in the interpreter
5. Add a strict mode

