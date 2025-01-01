# Prog Lang

![Build Status](https://github.com/im-fiv/prog-lang/actions/workflows/build.yml/badge.svg)
![Language](https://img.shields.io/badge/Language-Rust-orange)
![Contributions](https://img.shields.io/badge/Contributions-Open-brightgreen)
![Lines of Code](../image-data/badge.svg)
[![Hits-of-Code](https://hitsofcode.com/github/im-fiv/prog-lang?branch=main)](https://hitsofcode.com/github/im-fiv/prog-lang/view?branch=main)

Prog Lang is an interpreted programming language written in Rust, developed during the TulaHack 2024. It is designed to be as simple and as lightweight as possible. I have forked the original repository in order to preserve the exact version that we presented.

## Notable Features

- **Interpreted**: Albeit slower than compiled languages, does not require an architecture-dependent compiler and can be run on any machine.
- **Syntax**: Designed to be as simple as possible. Basic features, basic syntax.
- **Parsing Library**: Utilizes [pest.rs](https://pest.rs/) to parse the source code, enabling quick modifications to the grammar when needed.

## Whats Next

- [x] Objects
- [x] Spanned error reporting
- [x] Classes
- [ ] More standard functions
- [ ] Improved stability
  
## Getting Started

You can follow these simple steps to get started with Prog Lang:

1. **Clone the Repository**:

```bash
git clone https://github.com/im-fiv/prog-lang.git
```

2. **Build the project**:

```bash
cd prog-lang
cargo build --release
```

3. **Run**:

```bash
cargo run -- run file_name.prog
```

## Syntax

The specifications of Prog Lang are still being considered, but here's the currently accepted syntax:

```proglang
// this is a comment!
/* and so is this */

def variable_definition = "hello, world!"
def module_import = ("path goes here") -> import
def user_input = ("what is your favorite food?: ") -> input

variable_definition = "variable reassign!"

def function_definition = func(arg1, arg2) do
    return arg1 + arg2
end

def returning_nothing = func() do
    return none
end

def function_call = (2, 2) -> function_definition
() -> returning_nothing

if function_call == 4 then
    ("math works!") -> print
end

while function_call < 15 do
    ("while loop: ", function_call) -> print
    function_call = function_call + 1
end
```

Alternatively, the grammar file can be found at [crates/prog-parser/src/grammar.pest](../main/crates/prog-parser/src/grammar.pest)

## License

Prog Lang is [MIT licensed](https://en.wikipedia.org/wiki/MIT_License).
