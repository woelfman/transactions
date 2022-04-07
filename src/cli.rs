use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version)]
pub struct Args {
    #[clap(help = "CSV file to parse")]
    pub file: String,
}

pub fn parse_args() -> Args {
    Args::parse()
}
