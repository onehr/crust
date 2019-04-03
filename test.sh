#!/bin/bash
# build the project
cargo build
rm gen/*.s

# now just test whether the number returned was right
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'
inc=1
# now try function to test
test_fun() {
    ./target/debug/crust $1 $2
    gcc -o a.out $2
    ./a.out
    a=$?
    rm a.out
    gcc -o b.out $1
    ./b.out
    b=$?
    rm b.out
    inc=$(($inc+1))
    echo "TEST $inc: [$1] -> [$2]"
    echo "crustRet: $a gccRet: $b"
    if [ "$a" -eq "$b" ]; then
        echo -e "[${BLUE}Passed${NC}]"
    else
        echo -e "[${RED}Error${NC}]"
        exit 1
    fi
}
src=test/valid/multi_digit.c
dst=gen/multi_digit.s
test_fun $src $dst

src=test/valid/newlines.c
dst=gen/newlines.s
test_fun $src $dst

src=test/valid/no_newlines.c
dst=gen/no_newlines.s
test_fun $src $dst

src=test/valid/return_0.c
dst=gen/return_0.s
test_fun $src $dst

src=test/valid/return_2.c
dst=gen/return_2.s
test_fun $src $dst

src=test/valid/spaces.c
dst=gen/spaces.s
test_fun $src $dst

src=test/valid/bitwise.c
dst=gen/bitwise.s
test_fun $src $dst

src=test/valid/bitwise_zero.c
dst=gen/bitwise_zero.s
test_fun $src $dst

src=test/valid/neg.c
dst=gen/neg.s
test_fun $src $dst

src=test/valid/nested_ops.c
dst=gen/nested_ops.s
test_fun $src $dst

src=test/valid/nested_ops_2.c
dst=gen/nested_ops_2.s
test_fun $src $dst

src=test/valid/not_five.c
dst=gen/not_five.s
test_fun $src $dst

src=test/valid/not_zero.c
dst=gen/not_zero.s
test_fun $src $dst

src=test/valid/add.c
dst=gen/add.s
test_fun $src $dst

src=test/valid/associativity.c
dst=gen/associativity.s
test_fun $src $dst

src=test/valid/associativity_2.c
dst=gen/associativity_2.s
test_fun $src $dst

src=test/valid/associativity_3.c
dst=gen/associativity_3.s
test_fun $src $dst

src=test/valid/associativity_4.c
dst=gen/associativity_4.s
test_fun $src $dst

src=test/valid/associativity_5.c
dst=gen/associativity_5.s
test_fun $src $dst

src=test/valid/div.c
dst=gen/div.s
test_fun $src $dst

src=test/valid/mult.c
dst=gen/mult.s
test_fun $src $dst

src=test/valid/parens.c
dst=gen/parens.s
test_fun $src $dst

src=test/valid/precedence.c
dst=gen/precedence.s
test_fun $src $dst

src=test/valid/sub.c
dst=gen/sub.s
test_fun $src $dst

src=test/valid/sub_neg.c
dst=gen/sub_neg.s
test_fun $src $dst

src=test/valid/unop_add.c
dst=gen/unop_add.s
test_fun $src $dst

src=test/valid/unop_parens.c
dst=gen/unop_parens.s
test_fun $src $dst

srcdir=test/valid
dstdir=gen

src=$srcdir/and_false.c
dst=$dstdir/and_false.s
test_fun $src $dst

src=$srcdir/and_true.c
dst=$dstdir/and_true.s
test_fun $src $dst

src=$srcdir/eq_false.c
dst=$dstdir/eq_false.s
test_fun $src $dst

src=$srcdir/eq_true.c
dst=$dstdir/eq_true.s
test_fun $src $dst

src=$srcdir/ge_false.c
dst=$dstdir/ge_false.s
test_fun $src $dst

src=$srcdir/ge_true.c
dst=$dstdir/ge_true.s
test_fun $src $dst

src=$srcdir/gt_false.c
dst=$dstdir/gt_false.s
test_fun $src $dst

src=$srcdir/gt_true.c
dst=$dstdir/gt_true.s
test_fun $src $dst

src=$srcdir/le_false.c
dst=$dstdir/le_false.s
test_fun $src $dst

src=$srcdir/le_true.c
dst=$dstdir/le_true.s
test_fun $src $dst

src=$srcdir/lt_false.c
dst=$dstdir/lt_false.s
test_fun $src $dst

src=$srcdir/lt_true.c
dst=$dstdir/lt_true.s
test_fun $src $dst

src=$srcdir/ne_false.c
dst=$dstdir/ne_false.s
test_fun $src $dst

src=$srcdir/ne_true.c
dst=$dstdir/ne_true.s
test_fun $src $dst

src=$srcdir/or_false.c
dst=$dstdir/or_false.s
test_fun $src $dst

src=$srcdir/or_true.c
dst=$dstdir/or_true.s
test_fun $src $dst

src=$srcdir/precedence.c
dst=$dstdir/precedence.s
test_fun $src $dst

src=$srcdir/precedence_2.c
dst=$dstdir/precedence_2.s
test_fun $src $dst

src=$srcdir/precedence_3.c
dst=$dstdir/precedence_3.s
test_fun $src $dst

src=$srcdir/precedence_4.c
dst=$dstdir/precedence_4.s
test_fun $src $dst

src=$srcdir/assign.c
dst=$dstdir/assign.s
test_fun $src $dst

src=$srcdir/assign_val.c
dst=$dstdir/assign_val.s
test_fun $src $dst

src=$srcdir/exp_return_val.c
dst=$dstdir/exp_return_val.s
test_fun $src $dst

src=$srcdir/initialize.c
dst=$dstdir/initialize.s
test_fun $src $dst

# echo -e "[${RED}Skip${NC}] test/valid/missing_return.c"
# echo "(runs normal locally, but will get segmentation fault in travis, comment it first)"
# echo ""
src=$srcdir/missing_return.c
dst=$dstdir/missing_return.s
test_fun $src $dst

src=$srcdir/multiple_vars.c
dst=$dstdir/multiple_vars.s
test_fun $src $dst

src=$srcdir/no_initialize.c
dst=$dstdir/no_initialize.s
test_fun $src $dst

src=$srcdir/refer.c
dst=$dstdir/refer.s
test_fun $src $dst

src=$srcdir/unused_exp.c
dst=$dstdir/unused_exp.s
test_fun $src $dst
src=$srcdir/complicated_exp.c
dst=$dstdir/complicated_exp.s
test_fun $src $dst

src=$srcdir/assign_ternary.c
dst=$dstdir/assign_ternary.s
test_fun $src $dst

src=$srcdir/multiple_ternary.c
dst=$dstdir/multiple_ternary.s
test_fun $src $dst

src=$srcdir/nested_ternary.c
dst=$dstdir/nested_ternary.s
test_fun $src $dst

src=$srcdir/nested_ternary_2.c
dst=$dstdir/nested_ternary_2.s
test_fun $src $dst

src=$srcdir/rh_assignment.c
dst=$dstdir/rh_assignment.s
test_fun $src $dst

src=$srcdir/ternary.c
dst=$dstdir/ternary.s
test_fun $src $dst

src=$srcdir/ternary_short_circuit.c
dst=$dstdir/ternary_short_circuit.s
test_fun $src $dst

src=$srcdir/ternary_short_circuit_2.c
dst=$dstdir/ternary_short_circuit_2.s
test_fun $src $dst

src=$srcdir/else.c
dst=$dstdir/else.s
test_fun $src $dst

src=$srcdir/if_nested.c
dst=$dstdir/if_nested.s
test_fun $src $dst

src=$srcdir/if_nested_2.c
dst=$dstdir/if_nested_2.s
test_fun $src $dst

src=$srcdir/if_nested_3.c
dst=$dstdir/if_nested_3.s
test_fun $src $dst

src=$srcdir/if_nested_4.c
dst=$dstdir/if_nested_4.s
test_fun $src $dst

src=$srcdir/if_nested_5.c
dst=$dstdir/if_nested_5.s
test_fun $src $dst

src=$srcdir/if_not_taken.c
dst=$dstdir/if_not_taken.s
test_fun $src $dst

src=$srcdir/if_taken.c
dst=$dstdir/if_taken.s
test_fun $src $dst

src=$srcdir/multiple_if.c
dst=$dstdir/multiple_if.s
test_fun $src $dst


