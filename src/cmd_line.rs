use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub config_file: String,

    #[arg(long, default_value_t = false)]
    pub no_confirm: bool,
}

pub fn get_args() -> Args {
    Args::parse()
}
