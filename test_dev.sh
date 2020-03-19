#!/bin/bash
# build the project

## PS. Now this test do not generate any file.
## keep the -o file_name just for satisfy the command line option requirement

cargo build

RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'
inc=0

crust_compile() {
    echo "TEST $inc: parse [$1]"
    echo "crust parse $1.c -> ast" && ./target/debug/crust $1.c 
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

srcdir=sample_code
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

# should cause error in parser
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

# should cause no error
srcdir=test/valid/parser
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

# test for preprocessor
srcdir=test/valid/cpp
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

echo -e "Now the parser can parse them all"
