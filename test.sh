#!/bin/bash
# build the project
cargo fmt
cargo build

# now just test whether the number returned was right
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'
inc=1
src=test/valid/multi_digit.c
dst=gen/multi_digit.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/newlines.c
dst=gen/newlines.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/no_newlines.c
dst=gen/no_newlines.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/return_0.c
dst=gen/return_0.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/return_2.c
dst=gen/return_2.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/spaces.c
dst=gen/spaces.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi

src=test/valid/bitwise.c
dst=gen/bitwise.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/bitwise_zero.c
dst=gen/bitwise_zero.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi

src=test/valid/neg.c
dst=gen/neg.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi

src=test/valid/nested_ops.c
dst=gen/nested_ops.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi

src=test/valid/nested_ops_2.c
dst=gen/nested_ops_2.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi

src=test/valid/not_five.c
dst=gen/not_five.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi

src=test/valid/not_zero.c
dst=gen/not_zero.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi


src=test/valid/add.c
dst=gen/add.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/associativity.c
dst=gen/associativity.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/associativity_2.c
dst=gen/associativity_2.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/associativity_3.c
dst=gen/associativity_3.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/associativity_4.c
dst=gen/associativity_4.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/associativity_5.c
dst=gen/associativity_5.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/div.c
dst=gen/div.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/mult.c
dst=gen/mult.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/parens.c
dst=gen/parens.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/precedence.c
dst=gen/precedence.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/sub.c
dst=gen/sub.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/sub_neg.c
dst=gen/sub_neg.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/unop_add.c
dst=gen/unop_add.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
src=test/valid/unop_parens.c
dst=gen/unop_parens.s
./target/debug/crust $src $dst
gcc -o a.out $dst
./a.out
a=$?
gcc -o b.out $src
./b.out
b=$?
inc=$(($inc+1))
echo "TEST $inc: [$src] -> [$dst]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
