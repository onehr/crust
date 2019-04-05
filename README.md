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

## Usage
Cause now it's just in bare-metal development stage, so now it only supports little features.

You can write only a main function with no input.
and this function will only contains local variables declaration, assignment, and expression.

Now You can write some thing like this.
```c
int main() {
        int ans = 0;
        for (int i = 0; i < 10; i = i + 1) {
                for (int j = 0; j < 10; j = j + 1) {
                        ans = ans + 1;
                }
        }
        while (1) {
                ans = ans + 1;
                if (ans > 120) break;
        }
        do {
                ans = ans + 2;
        } while (ans > 160);

        int b = ans + 1;
        int c = b + 1;
        int d = b + c - ans;
        int a = ans * 2;
        a = ans - 1;
        for (int i = 0; i < 10; i = i + 1)
                a = a + 1;
        a = -a;
        a = -a;
        a = a / 2;
        a = a * 3;

        if (a == 197) {
                return 10 + a;
        } else
                return 0 + a;
}
```

And then the Crust compiler it will generat assmbly file like this:
```assembly
        .file "test/valid/combine.c"
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
.LBB1:
        movq $0, %rax
        pushq %rax
        movq $0, %rax
        pushq %rax
.LBFOR3:
        movq -16(%rbp), %rax
        pushq %rax
        movq $10, %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        setl %al
        cmpq $0, %rax
        je .LEFOR4
.LBB5:
.LBB7:
        movq $0, %rax
        pushq %rax
.LBFOR9:
        movq -24(%rbp), %rax
        pushq %rax
        movq $10, %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        setl %al
        cmpq $0, %rax
        je .LEFOR10
.LBB11:
.LBB13:
        movq -8(%rbp), %rax
        pushq %rax
        movq $1, %rax
        popq %rcx
        addq %rcx, %rax
        movq %rax, -8(%rbp)
.LEB14:
        addq $0, %rsp
.LEB12:
        addq $0, %rsp
        movq -24(%rbp), %rax
        pushq %rax
        movq $1, %rax
        popq %rcx
        addq %rcx, %rax
        movq %rax, -24(%rbp)
        jmp .LBFOR9
.LEFOR10:
        addq $8, %rsp
.LEB8:
        addq $0, %rsp
.LEB6:
        addq $0, %rsp
        movq -16(%rbp), %rax
        pushq %rax
        movq $1, %rax
        popq %rcx
        addq %rcx, %rax
        movq %rax, -16(%rbp)
        jmp .LBFOR3
.LEFOR4:
        addq $8, %rsp
.LBWHILE15:
        movq $1, %rax
        cmpq $1, %rax
        jne .LEWHILE16
.LBB17:
        movq -8(%rbp), %rax
        pushq %rax
        movq $1, %rax
        popq %rcx
        addq %rcx, %rax
        movq %rax, -8(%rbp)
        movq -8(%rbp), %rax
        pushq %rax
        movq $120, %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        setg %al
        cmpq $0, %rax
        je .LS219
        jmp .LEWHILE16 # Break
        jmp .LENDIF20
.LS219:
.LENDIF20:
.LEB18:
        addq $0, %rsp
        jmp .LBWHILE15
.LEWHILE16:
.LBDO21:
.LBB23:
        movq -8(%rbp), %rax
        pushq %rax
        movq $2, %rax
        popq %rcx
        addq %rcx, %rax
        movq %rax, -8(%rbp)
.LEB24:
        addq $0, %rsp
        movq -8(%rbp), %rax
        pushq %rax
        movq $160, %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        setg %al
        cmpq $1, %rax
        je   .LBDO21
.LEDO22:
        movq -8(%rbp), %rax
        pushq %rax
        movq $1, %rax
        popq %rcx
        addq %rcx, %rax
        pushq %rax
        movq -16(%rbp), %rax
        pushq %rax
        movq $1, %rax
        popq %rcx
        addq %rcx, %rax
        pushq %rax
        movq -8(%rbp), %rax
        pushq %rax
        movq -16(%rbp), %rax
        pushq %rax
        movq -24(%rbp), %rax
        popq %rcx
        addq %rcx, %rax
        popq %rcx
        subq %rcx, %rax
        pushq %rax
        movq -8(%rbp), %rax
        pushq %rax
        movq $2, %rax
        popq %rcx
        imul %rcx, %rax
        pushq %rax
        movq $1, %rax
        pushq %rax
        movq -8(%rbp), %rax
        popq %rcx
        subq %rcx, %rax
        movq %rax, -40(%rbp)
        movq $0, %rax
        pushq %rax
.LBFOR25:
        movq -48(%rbp), %rax
        pushq %rax
        movq $10, %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        setl %al
        cmpq $0, %rax
        je .LEFOR26
.LBB27:
        movq -40(%rbp), %rax
        pushq %rax
        movq $1, %rax
        popq %rcx
        addq %rcx, %rax
        movq %rax, -40(%rbp)
.LEB28:
        addq $0, %rsp
        movq -48(%rbp), %rax
        pushq %rax
        movq $1, %rax
        popq %rcx
        addq %rcx, %rax
        movq %rax, -48(%rbp)
        jmp .LBFOR25
.LEFOR26:
        addq $8, %rsp
        movq -40(%rbp), %rax
        neg %rax
        movq %rax, -40(%rbp)
        movq -40(%rbp), %rax
        neg %rax
        movq %rax, -40(%rbp)
        movq $2, %rax
        pushq %rax
        movq -40(%rbp), %rax
        popq %rcx
        xorq %rdx, %rdx
        idivq %rcx
        movq %rax, -40(%rbp)
        movq -40(%rbp), %rax
        pushq %rax
        movq $3, %rax
        popq %rcx
        imul %rcx, %rax
        movq %rax, -40(%rbp)
        movq -40(%rbp), %rax
        pushq %rax
        movq $197, %rax
        popq %rcx
        cmpq %rax, %rcx # set ZF on if %rax == %rcx, set it off otherwise
        movq $0, %rax   # zero out EAX, does not change flag
        sete %al
        cmpq $0, %rax
        je .LS231
.LBB29:
        movq $10, %rax
        pushq %rax
        movq -40(%rbp), %rax
        popq %rcx
        addq %rcx, %rax
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
.LEB30:
        addq $0, %rsp
        jmp .LENDIF32
.LS231:
        movq $0, %rax
        pushq %rax
        movq -40(%rbp), %rax
        popq %rcx
        addq %rcx, %rax
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        ret
.LENDIF32:
.LEB2:
        addq $40, %rsp
        movq %rbp, %rsp
        popq	%rbp
        .cfi_def_cfa 7, 8
        .cfi_endproc
.LFE33:
        .size   main, .-main
        .ident	"crust: 0.1 (By Haoran Wang)"
        .section	.note.GNU-stack,"",@progbits
```

## Contact
If you got interested in this project, or got troubles with it, feel free to contact me with 
waharaxn@gmail.com, best with tag [Crust-dev].
