mod clause;
mod cmd_line;
mod cnf;
mod config;
mod cube;
mod reconstruct;
mod tree_gen;
mod wcnf;

use crate::config::{Config, ConfigError};
use crate::cube::Cube;
use crate::reconstruct::parse_logs;
use crate::tree_gen::tree_gen;
use cmd_line::get_args;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::exit;
use std::{fs, io};
use tree_gen::hyper_vec;

fn setup_directories(config: &Config) -> Result<(), io::Error> {
    if !Path::exists(Path::new(&config.output_dir)) {
        fs::create_dir(&config.output_dir)?;
    }

    if config.multitree_variables.is_none() && !Path::exists(Path::new(&format!("{}/logs", &config.output_dir))) {
        fs::create_dir(format!("{}/logs", &config.output_dir))?;
    }

    if !Path::exists(Path::new(&config.tmp_dir)) {
        fs::create_dir(&config.tmp_dir)?;
    }

    Ok(())
}

fn main() -> Result<(), io::Error> {
    let args = get_args();
    let config_string = fs::read_to_string(args.config_file)?;

    let mut config = match Config::parse_config(&config_string) {
        Ok(c) => c,
        Err(ConfigError(s)) => {
            println!("Config Error: {s}");
            exit(1);
        }
    };

    if !args.no_confirm {
        println!("Configuration:");
        println!("{}\n", config);
        println!("Be aware that this program will overwrite data in the temporary directory and output directory.");
        print!("Please confirm that this config is correct (yes/y): ");
        let mut confirmation = String::new();
        stdout().flush()?;
        stdin().read_line(&mut confirmation)?;
        confirmation = confirmation.trim().to_lowercase();
        if !(confirmation == "yes" || confirmation == "y") {
            println!("Exiting due to no confirmation.");
            exit(1);
        }
    }

    let pool = match rayon::ThreadPoolBuilder::new().num_threads(config.thread_count).build() {
        Ok(p) => p,
        Err(_) => {
            println!("Error establishing thread pool");
            exit(1)
        }
    };

    setup_directories(&config)?;

    let start_cutoff = match config.comparator {
        config::Comparator::MaxOfMin => 0.0,
        config::Comparator::MinOfMax => f32::MAX,
    };

    match config.multitree_variables.clone() {
        Some(mut multitree_vars) => {
            let hvs = hyper_vec(&mut multitree_vars);
            let original_output_dir = config.output_dir;
            for v in hvs {
                let starter_cube = Cube(v);
                config.output_dir = format!("{}/{}", original_output_dir, &starter_cube);
                fs::create_dir(&config.output_dir)?;
                fs::create_dir(format!("{}/logs", &config.output_dir))?;
                tree_gen(&config, &pool, &starter_cube, start_cutoff, config.timeout as f32)?;
            }
        }
        None => {
            tree_gen(&config, &pool, &Cube(Vec::new()), start_cutoff, config.timeout as f32)?;
            parse_logs(
                &format!("{}/best.log", config.output_dir),
                &format!("{}/cubes.icnf", config.output_dir),
            )?
        }
    };

    Ok(())
}
