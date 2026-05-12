mod ast;
mod cpp;
mod lexer;
mod parser;
mod sema;
mod symtable;

use clap::Parser;
use log::{LevelFilter, info, trace};
use std::{error, fs, path::PathBuf};

#[derive(Parser)]
#[command(name = "Crust", version, about = "Crust is a C Compiler Powered by Rust")]
struct Cli {
    #[arg(required = true, help = "Input files")]
    files: Vec<PathBuf>,

    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count, help = "Sets the level of verbosity")]
    verbose: u8,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let cli = Cli::parse();

    let level = match cli.verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    simple_logger::SimpleLogger::new()
        .with_level(level)
        .with_colors(true)
        .init()?;

    for path in &cli.files {
        info!("Compiling: {}", path.display());
        let file_contents = fs::read_to_string(path)?;

        // 1. Preprocessing
        let contents_after_cpp = cpp::cpp_driver(file_contents, path.clone())?;
        trace!("File content after replacing PreProcessors: {:?}", contents_after_cpp);

        // 2. lexing
        let tokens = lexer::lex(&contents_after_cpp)?;
        trace!("Tokens: {:?}", &tokens);

        // 3. parsing
        let file_str = path.to_string_lossy().into_owned();
        let root_node = parser::parser_driver(&tokens, &file_str)?;
        trace!(
            "Source AST:{}",
            parser::parser_pretty_printer(&root_node, 0)
        );
    }

    Ok(())
}
