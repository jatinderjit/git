use clap::{Args, Parser};

const DEFAULT_BRANCH: &str = "main";

#[derive(Parser, Debug)]
#[command(version, about)]
pub enum Cli {
    Init(Init),
}

#[derive(Args, Debug)]
pub struct Init {
    pub path: Option<String>,

    #[arg(short = 'b', long, name = "branch-name", default_value = DEFAULT_BRANCH)]
    pub initial_branch: String,
}

pub fn parse() -> Cli {
    Cli::parse()
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
