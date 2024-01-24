mod cmd_line;
mod parser;

use std::fs;
use cmd_line::*;
use parser::parse_config;
fn main() -> Result<(), std::io::Error> {
    let args : Args = get_args();
    let config_string = fs::read_to_string(args.config_file)?;
    // the config parser handles its own errors
    // and exits on error.
    let config = parse_config(&config_string);

    return Ok(());
}
