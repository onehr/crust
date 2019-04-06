# CRUST
[![Build Status](https://travis-ci.com/onehr/crust.svg?branch=master)](https://travis-ci.com/onehr/crust)

A simple C compiler written in Rust-lang.

## Project Goal
Support C11 Standard, generate X86_64 Assembly Code from C source code.

PS. Now this compiler is in his bare-metal stage, using it to develop was like using a stick to face the dragon.
But I will keep developing it until it can compile real-world applications.
Now I will mainly focus on how to use Rust and build the basic structures.

## Now Support
**Cause now it's in development, so it only supports little features in C**.
1. Local Variables declaration and assignment
2. `return` statement
3. Unary Operator: `!`, `~`, `-`(Negative)
4. Binary Operator: `||`, `&&`, `<`, `>`, `>=`, `<=`, `==`, `+`, `-`, `/`, `*`
5. Now only support int data type.
6. Support `if` `else`, and `exp1 ? exp2 : exp3`
7. Support local scope binding now.
8. Support `for`, `while`, `do`, `break`, `continue` now.
9. Support function now.
10. Support global variables now.

## Build
First you need to setup your rust enviroment to build crust compiler, you need also gcc to trasnlate the assembly code to binary.
```bash
$ cargo build
```
You can run with
```bash
$ cargo run source_file output_file
```
You can get [output_file] as an assembly code file, which can be assembled into an ELF executable file.
If you want to run the program, you should type:
```bash
$ gcc -o a.out output_file
$ ./a.out
$ echo $?
```
Then you see the return value now. (PS. we just invoke the gcc as an assembler driver, cause the output_file was already an assembly file).

## test
```bash
$ mkdir gen/
$ ./test.sh
```

## Development Platform
* Platform : Windows Linux Subsystem (Ubuntu on Windows) + rustc(1.33.0) + cargo(1.33.0) + Emacs
* Toolchain: gcc 7.3.0

## Usage Example
Cause now it's just in bare-metal development stage, so now it only supports little features.

You can define your own function now, but main function can not take input arguments.
So you can write something like this.

```c
int fib(int a) {if (a == 0 || a == 1) {return a;} else {return fib(a - 1) + fib(a - 2);}}
int max(int a, int b) {return a > b ? a : b;}
int min(int a, int b) {return a < b ? a : b;}
int sum(int a, int b) {return a + b;}
int mul(int a, int b) {return a * b;}
int div(int a, int b) {return a / b;}

int EXIT_SUCCESS = 0;
int EXIT_FAILURE = 1;

int main() {
        int a = 2;
        int b = 3;
        int n = 10;

        if (fib(n) != 55) return EXIT_FAILURE;

        if (min(a, b) != 2) return EXIT_FAILURE;

        if (max(a, b) != 3) return EXIT_FAILURE;

        if (sum(a, b) != 5) return EXIT_FAILURE;

        if (mul(a, b) != 6) return EXIT_FAILURE;

        if (div(a, b) != 0) return EXIT_FAILURE;

        return EXIT_SUCCESS;
}
```
This is actually a unit test, it can test every function works correctly,
if everything goes fine, this program should retrun `EXIT_SUCCESS`, if there's something wrong, it will return 1,
which is `EXIT_FAILURE`.

If we use the `crust` compiler to compile this program and run the final executable file,
the program will return 0, which is correct.

The generated code would be some thing like this:
```assembly
        .file "test/valid/combine_2.c"
        .text
        .global fib
        .type fib, @function
fib:
.LFB0:
        .cfi_startproc
        pushq	%rbp
        .cfi_def_cfa_offset 16
        .cfi_offset 6, -16
        movq	%rsp, %rbp
        .cfi_def_cfa_register 6
.LBB1:
        movq 16(%rbp), %rax
        pushq %rax
        movq $0, %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        sete %al
        cmpq $0, %rax
        je .LCLAUSE3
        movq $1, %rax
        jmp .LEND4
.LCLAUSE3:
        movq 16(%rbp), %rax
        pushq %rax
        movq $1, %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        sete %al
        cmpq $0, %rax
        movq $0, %rax
        setne %al
.LEND4: # end of clause here
        cmpq $0, %rax
        je .LS29
.LBB5:
        movq 16(%rbp), %rax
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
.LEB6:
        addq $0, %rsp # block out
        jmp .LENDIF10
.LS29:
.LBB7:
        movq $1, %rax
        pushq %rax
        movq 16(%rbp), %rax
        popq %rcx
        subq %rcx, %rax
        pushq %rax
        call fib
        addq $8, %rsp # remove the arguments
        pushq %rax
        movq $2, %rax
        pushq %rax
        movq 16(%rbp), %rax
        popq %rcx
        subq %rcx, %rax
        pushq %rax
        call fib
        addq $8, %rsp # remove the arguments
        popq %rcx
        addq %rcx, %rax
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
.LEB8:
        addq $0, %rsp # block out
.LENDIF10:
.LEB2:
        addq $0, %rsp # block out
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        .cfi_endproc
.LFE11:
        .size   fib, .-fib
        .text
        .global max
        .type max, @function
max:
.LFB12:
        .cfi_startproc
        pushq	%rbp
        .cfi_def_cfa_offset 16
        .cfi_offset 6, -16
        movq	%rsp, %rbp
        .cfi_def_cfa_register 6
.LBB13:
        movq 16(%rbp), %rax
        pushq %rax
        movq 24(%rbp), %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        setg %al
        cmpq $0, %rax
        je .LE315
        movq 16(%rbp), %rax
        jmp .LENDCOND16
.LE315:
        movq 24(%rbp), %rax
.LENDCOND16:
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
.LEB14:
        addq $0, %rsp # block out
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        .cfi_endproc
.LFE17:
        .size   max, .-max
        .text
        .global min
        .type min, @function
min:
.LFB18:
        .cfi_startproc
        pushq	%rbp
        .cfi_def_cfa_offset 16
        .cfi_offset 6, -16
        movq	%rsp, %rbp
        .cfi_def_cfa_register 6
.LBB19:
        movq 16(%rbp), %rax
        pushq %rax
        movq 24(%rbp), %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        setl %al
        cmpq $0, %rax
        je .LE321
        movq 16(%rbp), %rax
        jmp .LENDCOND22
.LE321:
        movq 24(%rbp), %rax
.LENDCOND22:
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
.LEB20:
        addq $0, %rsp # block out
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        .cfi_endproc
.LFE23:
        .size   min, .-min
        .text
        .global sum
        .type sum, @function
sum:
.LFB24:
        .cfi_startproc
        pushq	%rbp
        .cfi_def_cfa_offset 16
        .cfi_offset 6, -16
        movq	%rsp, %rbp
        .cfi_def_cfa_register 6
.LBB25:
        movq 16(%rbp), %rax
        pushq %rax
        movq 24(%rbp), %rax
        popq %rcx
        addq %rcx, %rax
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
.LEB26:
        addq $0, %rsp # block out
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        .cfi_endproc
.LFE27:
        .size   sum, .-sum
        .text
        .global mul
        .type mul, @function
mul:
.LFB28:
        .cfi_startproc
        pushq	%rbp
        .cfi_def_cfa_offset 16
        .cfi_offset 6, -16
        movq	%rsp, %rbp
        .cfi_def_cfa_register 6
.LBB29:
        movq 16(%rbp), %rax
        pushq %rax
        movq 24(%rbp), %rax
        popq %rcx
        imul %rcx, %rax
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
.LEB30:
        addq $0, %rsp # block out
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        .cfi_endproc
.LFE31:
        .size   mul, .-mul
        .text
        .global div
        .type div, @function
div:
.LFB32:
        .cfi_startproc
        pushq	%rbp
        .cfi_def_cfa_offset 16
        .cfi_offset 6, -16
        movq	%rsp, %rbp
        .cfi_def_cfa_register 6
.LBB33:
        movq 24(%rbp), %rax
        pushq %rax
        movq 16(%rbp), %rax
        popq %rcx
        xorq %rdx, %rdx
        idivq %rcx
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
.LEB34:
        addq $0, %rsp # block out
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        .cfi_endproc
.LFE35:
        .size   div, .-div
        .globl	EXIT_SUCCESS
        .data
        .align 8
        .type	EXIT_SUCCESS, @object
        .size	EXIT_SUCCESS, 8
EXIT_SUCCESS:
        .long	0
        .globl	EXIT_FAILURE
        .data
        .align 8
        .type	EXIT_FAILURE, @object
        .size	EXIT_FAILURE, 8
EXIT_FAILURE:
        .long	1
        .text
        .global main
        .type main, @function
main:
.LFB36:
        .cfi_startproc
        pushq	%rbp
        .cfi_def_cfa_offset 16
        .cfi_offset 6, -16
        movq	%rsp, %rbp
        .cfi_def_cfa_register 6
.LBB37:
        movq $2, %rax
        pushq %rax
        movq $3, %rax
        pushq %rax
        movq $10, %rax
        pushq %rax
        movq -24(%rbp), %rax
        pushq %rax
        call fib
        addq $8, %rsp # remove the arguments
        pushq %rax
        movq $55, %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        setne %al
        cmpq $0, %rax
        je .LS239
        movq EXIT_FAILURE(%rip), %rax
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
        jmp .LENDIF40
.LS239:
.LENDIF40:
        movq -16(%rbp), %rax
        pushq %rax
        movq -8(%rbp), %rax
        pushq %rax
        call min
        addq $16, %rsp # remove the arguments
        pushq %rax
        movq $2, %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        setne %al
        cmpq $0, %rax
        je .LS241
        movq EXIT_FAILURE(%rip), %rax
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
        jmp .LENDIF42
.LS241:
.LENDIF42:
        movq -16(%rbp), %rax
        pushq %rax
        movq -8(%rbp), %rax
        pushq %rax
        call max
        addq $16, %rsp # remove the arguments
        pushq %rax
        movq $3, %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        setne %al
        cmpq $0, %rax
        je .LS243
        movq EXIT_FAILURE(%rip), %rax
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
        jmp .LENDIF44
.LS243:
.LENDIF44:
        movq -16(%rbp), %rax
        pushq %rax
        movq -8(%rbp), %rax
        pushq %rax
        call sum
        addq $16, %rsp # remove the arguments
        pushq %rax
        movq $5, %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        setne %al
        cmpq $0, %rax
        je .LS245
        movq EXIT_FAILURE(%rip), %rax
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
        jmp .LENDIF46
.LS245:
.LENDIF46:
        movq -16(%rbp), %rax
        pushq %rax
        movq -8(%rbp), %rax
        pushq %rax
        call mul
        addq $16, %rsp # remove the arguments
        pushq %rax
        movq $6, %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        setne %al
        cmpq $0, %rax
        je .LS247
        movq EXIT_FAILURE(%rip), %rax
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
        jmp .LENDIF48
.LS247:
.LENDIF48:
        movq -16(%rbp), %rax
        pushq %rax
        movq -8(%rbp), %rax
        pushq %rax
        call div
        addq $16, %rsp # remove the arguments
        pushq %rax
        movq $0, %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        setne %al
        cmpq $0, %rax
        je .LS249
        movq EXIT_FAILURE(%rip), %rax
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
        jmp .LENDIF50
.LS249:
.LENDIF50:
        movq EXIT_SUCCESS(%rip), %rax
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
.LEB38:
        addq $24, %rsp # block out
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        .cfi_endproc
.LFE51:
        .size   main, .-main
        .ident	"crust: 0.1 (By Haoran Wang)"
        .section	.note.GNU-stack,"",@progbits
```

## Contact
If you got interested in this project, or got troubles with it, feel free to contact me with 
waharaxn@gmail.com, best with tag [Crust-dev].
