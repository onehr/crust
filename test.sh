#!/bin/bash
# build the project
cargo build
rm -r gen/
mkdir -p gen/test/valid

# now just test whether the number returned was right
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'
inc=0
test_fun() {
    $1.crust
    a=$?
    $1.gcc
    b=$?
    inc=$(($inc+1))
    echo "TEST $inc: [$1] -> crustRet: $a gccRet: $b"
    if [ "$a" -eq "$b" ]; then
        echo -e "[${BLUE}Passed${NC}]"
    else
        echo -e "[${RED}Error${NC}]"
        exit 1
    fi
}

crust_compile() {
    echo "crust compile $1.c -> $2.s" && ./target/debug/crust -o $2.s $1.c
}

gcc_compile() {
    echo "gcc compile $2.s -> $2.crust"
    gcc -o $2.crust $2.s
    echo "gcc compile $1.c -> $2.gcc"
    gcc -std=c99 -o $2.gcc $1.c
}
srcdir=test/valid
echo -e "[${BLUE}crust compile all the test file now, generated file should be in CRUST_HOME/gen/test/valid/${NC}]"
for f in $srcdir/*.c
do
    file=${f%.*}
    crust_compile $file ./gen/$file
done
echo -e "[${BLUE}Now gcc compile the assembly code and prepare for test${NC}]"
for f in $srcdir/*.c
do
    file=${f%.*}
    gcc_compile $file ./gen/$file
done
echo -e "[${BLUE}test begins${NC}]"
for f in gen/test/valid/*.crust
do
    exec=${f%.*}
    test_fun $exec
done

echo -e "Passed ${BLUE}All${NC} tests :)"
