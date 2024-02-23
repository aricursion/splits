mod clause;
mod cmd_line;
mod cnf;
mod config;
mod cube;
mod reconstruct;
mod runners;
mod wcnf;

use clap::CommandFactory;
use counter::Counter;
use itertools::Itertools;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::exit;
use std::{fs, io};

use cmd_line::{get_args, Args};
use config::Config;
use cube::Cube;
use reconstruct::{parse_best_log, parse_leaf_cubes};
use runners::{hyper_vec, tree_gen};

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

fn parse_best(best_loc: &str) -> Result<(), io::Error> {
    let leaves = parse_leaf_cubes(best_loc)?;
    let mut var_counter: Counter<u32> = Counter::new();

    let leaf_cubes = leaves.iter().map(|(cube, _)| cube.0.iter().map(|x| x.abs_diff(0)));

    let times = leaves.iter().map(|(_, t)| t);
    let time_len = times.len();
    let sum_time = times.fold(0.0, |x, y| x + y);
    println!("Avg Runtime: {}", sum_time / (time_len as f32));
    println!("Sum Runtime: {}", sum_time);

    for cube in leaf_cubes {
        var_counter.extend(cube);
    }
    println!(
        "Variable Occurences: {:?}",
        var_counter.iter().sorted_by(|(_, x), (_, y)| usize::cmp(y, x))
    );

    Ok(())
}

fn run_tree(args: &Args, cfg_loc: &str) -> Result<(), io::Error> {
    let config_string = match fs::read_to_string(cfg_loc) {
        Ok(s) => s,
        Err(_) => {
            println!("Could not find config file");
            exit(1);
        }
    };

    let mut config = match Config::parse_config(&config_string) {
        Ok(c) => c,
        Err(c) => {
            println!("Config Error: {c}");
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
        Err(e) => {
            println!("Error establishing thread pool");
            println!("{e}");
            exit(1)
        }
    };

    setup_directories(&config)?;

    let start_cutoff_metric = match config.comparator {
        config::Comparator::MaxOfMin => f32::MIN,
        config::Comparator::MinOfMax => f32::MAX,
    };

    let start_cutoff_time = config.timeout as f32;

    match config.multitree_variables.to_owned() {
        Some(multitree_vars) => {
            let hvs = hyper_vec(&multitree_vars);
            let original_output_dir = config.output_dir;
            for v in hvs {
                let starter_cube = Cube(v);
                config.output_dir = format!("{}/{}", original_output_dir, &starter_cube);
                fs::create_dir(&config.output_dir)?;
                fs::create_dir(format!("{}/logs", &config.output_dir))?;
                tree_gen(
                    &config,
                    &pool,
                    &starter_cube,
                    &config.start_variables,
                    start_cutoff_metric,
                    start_cutoff_time,
                    0,
                )?;

                if !config.preserve_logs {
                    fs::remove_dir_all(format!("{}/logs", &config.output_dir))?
                }
            }
        }
        None => {
            tree_gen(
                &config,
                &pool,
                &Cube(Vec::new()),
                &config.start_variables,
                start_cutoff_metric,
                start_cutoff_time,
                0,
            )?;
            parse_best_log(
                &format!("{}/best.log", config.output_dir),
                &format!("{}/cubes.icnf", config.output_dir),
            )?;

            if !config.preserve_logs {
                fs::remove_dir_all(format!("{}/logs", &config.output_dir))?
            }
        }
    };

    if !config.preserve_cnf {
        fs::remove_dir_all(config.tmp_dir)?;
    }
    Ok(())
}
fn main() -> Result<(), io::Error> {
    let args = get_args();

    match (&args.parse_best, &args.config_file) {
        (None, Some(cfg_file)) => run_tree(&args, cfg_file),
        (Some(best_loc), None) => parse_best(best_loc),
        (None, None) => {
            Args::command().print_help()?;
            Ok(())
        }
        _ => {
            println!("Please don't use mutually exclusive options.");
            println!("If you are reading this I should update this help message");
            Ok(())
        }
    }
}
