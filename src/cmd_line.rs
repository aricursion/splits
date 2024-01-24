use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub config_file: String,
}

pub fn get_args() -> Args {
    return Args::parse();
}
