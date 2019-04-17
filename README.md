# CRUST
[![Build Status](https://travis-ci.com/onehr/crust.svg?branch=master)](https://travis-ci.com/onehr/crust)
[![Gitter](https://badges.gitter.im/crust-dev/community.svg)](https://gitter.im/crust-dev/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2Fonehr%2Fcrust.svg?type=shield)](https://app.fossa.io/projects/git%2Bgithub.com%2Fonehr%2Fcrust?ref=badge_shield)


A simple C compiler written in the Rust-lang. (early development stage, started at Mar 30, 2019)

**(PS. this is the development branch, 
if you want to see how to write a simple c compiler in rust, you should check 
[branch master](https://github.com/onehr/crust/tree/master), 
which contains a simple c compiler written in rust without extra libs, 
it can read simple c source code and produce x86-64 assembly code).**

## Project Goal
Should follow the C11 Standard and generate binary code from C source code.

This compiler is in the very early development stage,
the plan is to continue developing it until it can compile real-world applications.

If you are interested in `crust` and want to contribute, feel free to join the Gitter chat room, 
we have already got some contributors now who are interested in building this project.

## Milestone 0.1 Goal
1. Finish the preprocessor.
2. Support all C11 grammar rules.
3. replace gcc with it's own assembler to generate binary code
4. Stabilize the interfaces among different layers.
5. With some possible optimizations.

## Track of current progress
- Preprocessor (working on)
    - [X] support `#include "local-header"`, nested-include is supported (need to add more features)
    - [X] Trigraph translation
    - [X] comment support `/**/ and //`
    - [X] line concatenation with ` \ `
    - [X] object-like macro expansion
    - [ ] function-like macro expansion
    - [ ] should support all directives later
- Lexer (working on)
    - [X] lex all c11 keywords
    - [ ] the floating point number and number with postfix should be supported later.
* Parser (almost done, need to be carefully tested)
    - [X] support c11 standard and generate ast tree
    - [ ] better ast printer
    - [ ] should be able handle typedef
    - [ ] add more tests for parser
* Semantics Analyzer (TODO)
    - [ ] need to add analyzer to track type
* IR generator (TODO)
* Optimizer (TODO)
* Assembly code generator (TODO)
* Assembler (TODO)
## Requirements

You need a valid rust environment, Cargo.

## Build
(PS. Now the crust can only preprocess, lex, and parse the source code, the generator was disabled now).
```bash
$ cargo build # use this command to build the project
```
run
```shell
$ cargo run -- -E source_file.c -o output_file.c # generate preprocessed file
$ cargo run -- --crust-print-source-token source_file.c -o output_file.c # print the token
$ cargo run -- --crust-print-source-ast source_file.c -o output_file.c # print the ast

```

## Running Tests

Run:
```bash
$ ./test_dev.sh
```

## License
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2Fonehr%2Fcrust.svg?type=large)](https://app.fossa.io/projects/git%2Bgithub.com%2Fonehr%2Fcrust?ref=badge_large)
