use structopt::clap;

pub fn stop_stage_conflict_resolver_arg_group() -> clap::ArgGroup<'static> {
    clap::ArgGroup::with_name("stop_stage_conflict_resolver")
}
