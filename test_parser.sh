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

crust_compile() {
    echo "TEST $inc: parse [$1]"
    echo "crust parse $1.c -> ast" && ./target/debug/crust $1.c -o $2.s
}

echo -e "[${BLUE}test begins${NC}]"
echo -e "${BLUE}crust now only try to parse those files${NC}"

srcdir=test/valid
for f in $srcdir/*.c
do
    inc=$(($inc+1))
    file=${f%.*}
    crust_compile $file ./gen/$file
    if [ "$?" -ne 0 ]; then
        echo -e "[${RED}Error${NC}]"
        exit 1
    else
        echo -e "[${BLUE}parse ok${NC}]"
    fi
done

srcdir=sample_code/
for f in $srcdir/*.c
do
    inc=$(($inc+1))
    file=${f%.*}
    crust_compile $file ./gen/$file
    if [ "$?" -ne 0 ]; then
        echo -e "[${RED}Error${NC}]"
        exit 1
    else
        echo -e "[${BLUE}parse ok${NC}]"
    fi
done
srcdir=test/invalid
for f in $srcdir/*.c
do
    inc=$(($inc+1))
    file=${f%.*}
    crust_compile $file ./gen/$file
    if [ "$?" -ne 1 ]; then
        echo -e "[${RED}Error${NC}]"
        exit 1
    else
        echo -e "[${BLUE}parse ok${NC}]"
    fi
done

echo -e "Now the parser can parse them all"
