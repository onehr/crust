mod lexer;
mod parser;
use lexer::lex;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    // first check the length of args is 2.
    // format: crust source.c output.s
    // TODO: need to add more options later
    if args.len() <= 1 {
        print_usage();
        return;
    }

    let c_src_name = &args[1];
    let s_src_name = &args[2];

    // TODO: add better error messages.
    let contents = fs::read_to_string(c_src_name).expect("Can't read file");

    println!("--------------------------------");
    println!("SOURCE_FILE: [{}]", c_src_name);
    println!("--------------------------------");
    println!("{}", contents);

    // TODO: use try! macro later, now will get error
    let mut token_list = Vec::new();
    match lexer::lex(&contents) {
        Ok(n) => {
            token_list = n;
        }
        Err(_) => panic!("Can not lex properly!"),
    }

    println!("--------------------------------");
    println!("Token List : ");
    println!("{:?}", token_list);
    println!("number of tokens: {}", token_list.len());

    let mut root_node = parser::ParseNode::new();
    match parser::parse_prog(&contents, c_src_name) {
        Ok(n) => {
            root_node = n;
        }
        _ => panic!("Can not parse properly"),
    }
    println!("--------------------------------");
    println!("AST nodes:\n{}", parser::print(&root_node, 0));

    //fs::write(s_src_name, s_contents).expect("Can't write assembly code");
}

fn print_usage() {
    println!("--------------------------------");
    println!("Copyright (c) 2019, Haoran Wang");
    println!("--------------------------------");
    println!("Usage: crust src_file[.c]\n");
}
