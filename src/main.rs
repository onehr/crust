
mod ast;
mod cpp;
mod lexer;
mod parser;
mod sema;
mod symtable;

use clap::{App, Arg};
use log::{trace, info};
use std::{fs, error, path::Path};

fn main()  -> Result<(), Box<dyn error::Error>> {
    let args = App::new("Crust")
        .version("0.1.0")
        .about("Crust is a C Compiler Powered by Rust")
        .arg(
            Arg::with_name("files")
                .required(true)
                .multiple(true)
                .help("Input files"),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    loggerv::Logger::new()
        .verbosity(args.occurrences_of("v"))
        .level(false)
        .colors(true)
        .level(true)
        .init()
        .unwrap();

    let files: Vec<_> = args.values_of("files").unwrap().collect();

    for file in files {
        info!("Compiling: {}", file);
        let path = Path::new(file);
        let file_contents = fs::read_to_string(path)?;

        // 1. Preprocessing
        let contents_after_cpp = cpp::cpp_driver(file_contents, path.to_path_buf())?;
        trace!("File content after replacing PreProcessors: {:?}", contents_after_cpp);

        // 2. lexing
        let tokens = lexer::lex(&contents_after_cpp)?;
        trace!("Tokens: {:?}", &tokens);


        // 3. parsing
        let root_node = parser::parser_driver(&tokens, &file)?;
        trace!(
            "Source AST:{}",
            parser::parser_pretty_printer(&root_node, 0)
        );
    }

    Ok(())
}
