//mod gen;
mod ast;
mod cpp;
mod lexer;
mod opts;
mod parser;
mod sema;
mod symtable;

use std::{error, fs};

fn main() -> Result<(), Box<dyn error::Error>> {
    let opts: opts::Opts = {
        use structopt::StructOpt;

        opts::Opts::from_args()
    };

    // TODO: allow support for multiple input files.
    //       Currently it tries to get the first input file and thats all
    let input_file = opts.input()[0].clone();

    if opts.crust_debug_flags().print_filenames() {
        println!("Source file: {}\n", input_file.display())
    }

    let input_file_contents = fs::read_to_string(input_file.clone())?;

    if opts.crust_debug_flags().print_file_contents() {
        println!("File contents:\n{}\n", input_file_contents)
    }

    // 1. Preprocessing
    let contents_after_cpp = cpp::cpp_driver(input_file_contents, input_file.clone())?;

    if opts.stop_stage().preprocess() {
        fs::write(opts.output(), contents_after_cpp)?;
        return Ok(());
    }

    // 2. lexing
    let tokens = lexer::lex(&contents_after_cpp)?;

    if opts.crust_debug_flags().print_source_token() {
        println!("Tokens: \n");
        let mut inc = 0;
        for it in tokens.iter() {
            print!("{}: [{:?}] ", inc, it);
            inc += 1;
            if inc % 5 == 0 {
                println!("");
            }
        }
        println!("");
    }

    // 3. parsing
    let root_node = parser::parser_driver(&tokens, &input_file.display().to_string())?;

    if opts.crust_debug_flags().print_source_ast() {
        println!(
            "Source AST:\n{}\n",
            parser::parser_pretty_printer(&root_node, 0)
        )
    }

    // 4. semantics check
    sema::sema_driver(&root_node, &input_file.display().to_string())?;

    if opts.crust_debug_flags().print_filenames() {
        println!("Output file: {}\n", opts.output().display());
    }

    // let output_file_contents = gen::gen_prog(&root_node);

    if opts.crust_debug_flags().print_file_contents() {
        // println!("File contents:\n{}\n", output_file_contents)
    }

    // fs::write(opts.output(), output_file_contents)?;
    Ok(())
}
