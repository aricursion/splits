mod cmd_line;
mod config;
mod cube;

use crate::config::{Config, ConfigError};
use cmd_line::*;
use std::fs;
use std::io::{stdin, stdout, Write};
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

    if !args.no_confirm {
        println!("Configuration:");
        println!("{}", config);
        println!("Be aware that this program will overwrite data in the temporary directory and output directory.");
        print!("Please confirm that this config is correct (yes/y): ");
        let mut confirmation = String::new();
        let _ = stdout().flush();
        stdin().read_line(&mut confirmation)?;
        confirmation = confirmation.trim().to_lowercase();
        if !(confirmation == "yes" || confirmation == "y") {
            println!("Exiting due to no confirmation.");
            exit(1);
        }
    }

    return Ok(());
}
