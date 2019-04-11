mod gen;
mod lexer;
mod opts;
mod parser;

use std::{env, error, fs};

fn main() -> Result<(), Box<dyn error::Error>> {
    let opts: opts::Opts = {
        use structopt::StructOpt;

        opts::Opts::from_args()
    };

    // TODO: allow support for multiple input files.
    //       Currently it tries to get the first input file and thats all
    let input_file = opts.input()[0].clone();

    let input_file_contents = fs::read_to_string(input_file.clone())?;
    let tokens = lexer::lex(&input_file_contents)?;
    let root_node = parser::parse_prog(&input_file_contents, &input_file.display().to_string())?;
    let output_file_contents = gen::gen_prog(&root_node);

    fs::write(opts.output(), output_file_contents)?;
    Ok(())
    /*
    let args: Vec<String> = env::args().collect();
    let c_src_name = &args[1];
    let s_src_name = &args[2];
    let contents = fs::read_to_string(c_src_name).expect("Can't read file");
    if cfg!(feature = "source") {
        println!("--------------------------------");
        println!("SOURCE_FILE: [{}]", c_src_name);
        println!("--------------------------------");
        println!("{}", contents);
    }

    let token_list = r#try!(lexer::lex(&contents));
    if cfg!(feature = "token") {
        println!("tokens: {:?}\n", token_list);
        println!("number of tokens: {}", token_list.len());
    }

    let root_node = r#try!(parser::parse_prog(&contents, c_src_name));
    if cfg!(feature = "ast") {
        println!("\nAST nodes:\n{}\n", parser::print(&root_node, 0));
    }

    let s_contents = gen::gen_prog(&root_node);
    if cfg!(feature = "as") {
        println!("\nAS FILE:\n{}", s_contents);
    }

    fs::write(s_src_name, s_contents).expect("Can't write assembly code");
    */
}

fn print_usage() {
    println!("--------------------------------");
    println!("Copyright (c) 2019, Haoran Wang");
    println!("--------------------------------");
    println!("Usage: crust src_file[.c] out_file[.s]\n");
}
