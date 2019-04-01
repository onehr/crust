mod gen;
mod lexer;
mod parser;

use lexer::lex;

use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    // first check the length of args is 3.
    // format: crust source.c output.s
    // TODO: need to add more options later
    if args.len() != 3 {
        print_usage();
        return Ok(());
    }

    let c_src_name = &args[1];
    let s_src_name = &args[2];

    let contents = fs::read_to_string(c_src_name).expect("Can't read file");
    // println!("--------------------------------");
    // println!("SOURCE_FILE: [{}]", c_src_name);
    // println!("--------------------------------");
    // println!("{}", contents);

    let token_list = r#try!(lexer::lex(&contents));
    // println!("--------------------------------");
    // println!("Token List : ");
    // println!("{:?}", token_list);
    // println!("number of tokens: {}", token_list.len());

    let root_node = r#try!(parser::parse_prog(&contents, c_src_name));
    //println!("--------------------------------");
    //println!("AST nodes:\n{}", parser::print(&root_node, 0));

    let s_contents = gen::gen_as(&root_node);
    // println!("--------------------------------");
    // println!("AS FILE:\n{}", s_contents);

    fs::write(s_src_name, s_contents).expect("Can't write assembly code");
    Ok(())
}

fn print_usage() {
    println!("--------------------------------");
    println!("Copyright (c) 2019, Haoran Wang");
    println!("--------------------------------");
    println!("Usage: crust src_file[.c] out_file[.s]\n");
}
