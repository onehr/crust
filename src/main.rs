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

    println!("----------------------------");
    println!("SOURCE_FILE:{}", c_src_name);
    println!("----------------------------");
    println!("{}", contents);
    println!("----------------------------");

    let token_list_res = lexer::lex(&contents);
    let mut token_list = Vec::new();
    //let mut token_list: &std::vec::Vec<_>;

    match token_list_res {
        Ok(n) => {
            token_list = n;
        }
        Err(_) => panic!("Can not lex properly!"),
    }
    println!("{:?}", token_list);
    println!("number of tokens: {}", token_list.len());
    /*
    let mut it = token_list.iter().peekable();

    while let Some(&c) = it.peek() {
        println!("c = {:?}", c);
        it.next();
    }*/
    let parse_nodes = parser::parse_prog(&contents);
    println!("{:?}", parse_nodes);
    //let mut it = token_list.iter().peekable();
    //let parse_nodes = parser::parse_prog(token_list, &mut it);
    //println!("{:?}", parse_nodes);
    //fs::write(s_src_name, s_contents).expect("Can't write assembly code");
}

fn print_usage() {
    println!("-------------------------------");
    println!("Copyright (c) 2019, Haoran Wang");
    println!("-------------------------------");
    println!("Usage: crust src_file[.c]\n");
}
