use structopt::clap;

pub fn overall_opts_arg_group() -> clap::ArgGroup<'static> {
    clap::ArgGroup::with_name("OverallOptions").required(true)
}

pub fn output_opts_arg_group() -> clap::ArgGroup<'static> {
    clap::ArgGroup::with_name("OutputOptions")
}

pub fn stop_stage_conflict_resolver_arg_group() -> clap::ArgGroup<'static> {
    clap::ArgGroup::with_name("stop_stage_conflict_resolver")
}
