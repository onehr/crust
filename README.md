# CRUST
[![Build Status](https://travis-ci.com/onehr/crust.svg?branch=master)](https://travis-ci.com/onehr/crust)

A simple C compiler (do not contain assembler) written in Rust-lang.

## Project Goal
Support C89 Standard, generate X86_64 Assembly Code from C source code.

## Now Support
**Cause now it's in development, so it only supports little features in C**.
1. Local Variables declaration and assignment
2. `return` statement
3. Unary Operator: `!`, `~`, '-'(Negative)
4. Binary Operator: `||`, `&&`, `<`, `>`, `>=`, `<=`, `==`, `+`, `-`, `/`, `*`
5. Now only support int data type.

## Usage
Cause now it's just in bare-metal development stage, so now it only supports little features.

You can write only a main function with no input.
and this function will only contains local variables declaration, assignment, and expression.

e.g.
```c
int main() {
        return (1 + 3 * 4 / 4 - 5 * 10 + 100) * 10 / 10 - 1;
}
```
it will generat assmbly file like this:
```assmbly
        .file "test/valid/complicated_exp.c"
        .global main
        .type main, @function
main:
.LFB0:
        .cfi_startproc
        pushq	%rbp
        .cfi_def_cfa_offset 16
        .cfi_offset 6, -16
        movq	%rsp, %rbp
        .cfi_def_cfa_register 6
        movq $1, %rax
        pushq %rax
        movq $10, %rax
        pushq %rax
        movq $5, %rax
        pushq %rax
        movq $10, %rax
        popq %rcx
        imul %rcx, %rax
        pushq %rax
        movq $1, %rax
        pushq %rax
        movq $4, %rax
        pushq %rax
        movq $3, %rax
        pushq %rax
        movq $4, %rax
        popq %rcx
        imul %rcx, %rax
        popq %rcx
        xorq %rdx, %rdx
        idivq %rcx
        popq %rcx
        addq %rcx, %rax
        popq %rcx
        subq %rcx, %rax
        pushq %rax
        movq $100, %rax
        popq %rcx
        addq %rcx, %rax
        pushq %rax
        movq $10, %rax
        popq %rcx
        imul %rcx, %rax
        popq %rcx
        xorq %rdx, %rdx
        idivq %rcx
        popq %rcx
        subq %rcx, %rax
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
        .cfi_endproc
.LFE1:
        .size	main, .-main
        .ident	"crust: 0.1 (By Haoran Wang)"
        .section	.note.GNU-stack,"",@progbits
```

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

## Contact
If you got interested in this project, or got troubles with it, feel free to contact me with 
waharaxn@gmail.com, best with tag [Crust-dev].
