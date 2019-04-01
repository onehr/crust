# CRUST
A simple C compiler (do not contain assembler) written in Rust-lang.
## Project Goal
Support C89 Standard, generate X86_64 Assembly Code from C source code.

First time writing Rust code, just try to implement a small compiler in a new way.

## Now Support
**Cause now it's in development, so it only supports little features in C**.
1. Local Variables declaration and assignment
2. `return` statement
3. Unary Operator: `!`, `~`, '-'(Negative)
4. Binary Operator: `||`, `&&`, `<`, `>`, `>=`, `<=`, `==`, `+`, `-`, `/`, `*`
5. Now only support int data type.

## Build
First you need to setup your rust enviroment to build crust compiler, you need also gcc to trasnlate the assembly code to binary.
```bash
$ cargo build
```
You can run with
```bash
$ cargo run source_file output_file
```

## test
```bash
$ mkdir gen/
$ ./test.sh
```

## Development Platform
* Platform : Windows Linux Subsystem (Ubuntu on Windows) + rustc(1.33.0) + cargo(1.33.0) + Emacs
* Toolchain: gcc 7.3.0

