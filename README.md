# Prog-Lang
![Build Status](https://github.com/PrintHiEgProg/prog-lang/actions/workflows/build.yml/badge.svg)
![Language](https://img.shields.io/badge/Language-Rust-orange)
![Contributions](https://img.shields.io/badge/Contributions-Closed-red)
![Lines of Code](https://raw.githubusercontent.com/PrintHiEgProg/prog-lang/image-data/badge.svg)
[![Hits-of-Code](https://hitsofcode.com/github/PrintHiEgProg/prog-lang?branch=main)](https://hitsofcode.com/github/PrintHiEgProg/prog-lang/view?branch=main)

Prog-Lang is an interpreted programming language written in Rust, developed during the Spring 2024 Hackathon. It is designed to be as simple and as lightweight as possible.

## Notable Features

- **Interpreted**: Albeit slower than compiled languages, does not have a need for a compiler and can be run on any machine.
- **Syntax**: Designed to be as simple as possible. Basic features, basic syntax.
- **Parsing Library**: Utilizes [pest.rs](https://pest.rs/) to parse the source code, enabling quick modifications when needed.
  
## Getting Started

You can follow these simple steps to get started with Prog-Lang:


1. **Clone the Repository**:

```bash
git clone https://github.com/yourusername/prog-lang.git
```

2. **Build the project**:

```bash
cd prog-lang
cargo build --release
```

3. **Run**:

```bash
cargo run -- file_name.prog
```

## Syntax

The specifications of Prog-Lang are still being considered, but here's the currently accepted syntax:

```proglang
def text_to_print = "hello, world!"

// all functions are first-class functions and can be treated as values
def some_calculation = func(a, b, c) do
	return a + b * c
end

def main = func() do
	def calculated_stuff = (2, 2, 2) -> some_calculation
	def counter = 0

	while calculated_stuff + counter < 15 do
		counter += 1
		counter -> print
	end

	(
		calculated_stuff,
		"+",
		counter,
		"=",
		calculated_stuff + counter
	) -> print
end
```