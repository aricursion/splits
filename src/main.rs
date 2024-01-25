mod cmd_line;
mod config;

use crate::config::{Config, ConfigError};
use cmd_line::*;
use std::fs;
use std::process::exit;
fn main() -> Result<(), std::io::Error> {
    let args: Args = get_args();
    let config_string = fs::read_to_string(args.config_file)?;

    let config = match Config::parse_config(&config_string) {
        Ok(c) => c,
        Err(ConfigError(s)) => {
            println!("Config Error: {}", s);
            exit(1);
        }
    };
    println!("{:?}", config);

    return Ok(());
}
