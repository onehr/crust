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
echo "TEST $inc: [$src]"
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
echo "TEST $inc: [$src]"
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
echo "TEST $inc: [$src]"
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
echo "TEST $inc: [$src]"
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
echo "TEST $inc: [$src]"
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
echo "TEST $inc: [$src]"
echo "crustRet: $a gccRet: $b"
if [ "$a" -eq "$b" ]; then
    echo -e "${BLUE}[Passed]${NC}"
else
    echo -e "[Error]"
fi
