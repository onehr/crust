use std::path;

mod arg_groups;

#[derive(structopt::StructOpt, Clone, Debug)]
pub struct Opts {
    #[structopt(flatten)]
    crust_debug_flags: CrustDebugFlags,
    /// The input files(s)
    #[structopt(parse(from_os_str), required = true)]
    input: Vec<path::PathBuf>,
    /// The output file
    #[structopt(short = "-o", parse(from_os_str))]
    output: path::PathBuf,
    #[structopt(flatten)]
    stop_stage: StopStage,
}

#[derive(structopt::StructOpt, Clone, Copy, Debug)]
pub struct CrustDebugFlags {
    /// Print file contents
    #[structopt(long = "--crust-print-file-contents")]
    print_file_contents: bool,
    /// Print filenames as they are processed
    #[structopt(long = "--crust-print-filenames")]
    print_filenames: bool,
    /// Print the source file ast.
    #[structopt(long = "--crust-print-source-ast")]
    print_source_ast: bool,
}

#[derive(structopt::StructOpt, Clone, Copy, Debug)]
#[structopt(raw(group = "self::arg_groups::stop_stage_conflict_resolver_arg_group()"))]
pub struct StopStage {
    /// Stop after the assembly stage
    #[structopt(group = "stop_stage_conflict_resolver", short = "-c")]
    assemble: bool,
    /// Stop after the compilation stage
    #[structopt(group = "stop_stage_conflict_resolver", short = "-S")]
    compile: bool,
    /// Stop after the preprocessing stage
    #[structopt(group = "stop_stage_conflict_resolver", short = "-E")]
    preprocess: bool,
}

impl Opts {
    pub fn crust_debug_flags(&self) -> CrustDebugFlags {
        self.crust_debug_flags
    }

    pub fn input(&self) -> &[path::PathBuf] {
        &self.input
    }

    pub fn output(&self) -> &path::PathBuf {
        &self.output
    }

    pub fn stop_stage(&self) -> StopStage {
        self.stop_stage
    }
}

impl CrustDebugFlags {
    pub fn print_file_contents(&self) -> bool {
        self.print_file_contents
    }

    pub fn print_filenames(&self) -> bool {
        self.print_filenames
    }

    pub fn print_source_ast(&self) -> bool {
        self.print_source_ast
    }
}

impl StopStage {
    pub fn assemble(&self) -> bool {
        self.assemble
    }

    pub fn compile(&self) -> bool {
        self.compile
    }

    pub fn preprocess(&self) -> bool {
        self.preprocess
    }
}
