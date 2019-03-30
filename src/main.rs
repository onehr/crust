use std::env;
use std::fs;

mod lexer;

fn main() {
    let args: Vec<String> = env::args().collect();

    // first check the length of args is 2.
    // format: crust source.c output.s
    // TODO: need to add more options later
    if args.len() != 2 {
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

    let token_list = lexer::lexer::lex(&contents);
    println!("{:?}", token_list);
    //fs::write(s_src_name, s_contents).expect("Can't write assembly code");

}

fn print_usage() {
    println!("-------------------------------");
    println!("Copyright (c) 2019, Haoran Wang");
    println!("-------------------------------");
    println!("Usage: crust src_file[.c]\n");
}
