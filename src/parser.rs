use std::path::Path;
use std::process::exit;
use is_executable::IsExecutable;

#[derive(Debug)]
pub enum Comparator {
    Max,
    Min,
}
#[derive(Debug)]
pub struct Config {
    variables: Vec<u32>,
    comparator: Comparator,
    timeout: u32,
    solver: String,
    cnf: String,
    output_dir: String,
    tmp_dir: String,
    tracked_metrics: Vec<String>,
    evaluation_metric: String,
}

pub fn parse_config(config_string: &str) -> Config {
    let mut variables = Vec::new();
    let mut comparator = Comparator::Min;
    let mut solver = String::from("");
    let mut timeout = 1000;
    let mut cnf = String::from("");
    let mut output_dir = String::from("output");
    let mut tmp_dir = String::from("tmp");
    let mut tracked_metrics = Vec::new();
    let mut evaluation_metric = String::from("");

    let mut is_set: [bool; 5] = [false, false, false, false, false];

    for line in config_string.lines() {
        let partial_parse_line = line.split(":").collect::<Vec<&str>>();
        let name = partial_parse_line[0];
        let argument = partial_parse_line[1].trim();

        match name {
            "variables" => {
                for var_str in argument.split(" ") {
                    match var_str.parse::<u32>() {
                        Ok(u) => {
                            if u == 0 {
                                println!("0 is not a valid cnf variable");
                                exit(1);
                            } else {
                                variables.push(u)
                            }
                        }
                        Err(_) => {
                            println!("Failed to parse variable: {}. Please ensure they are all positive integers.",var_str);
                            exit(1);
                        }
                    }
                }
                is_set[0] = true;
            }
            "comparator" => {
                match argument {
                    "min" => comparator = Comparator::Min,
                    "max" => comparator = Comparator::Max,
                    _ => {
                        println!("Failed to recognize Comparison Operator. Please use either 'min' or 'max'");
                        exit(1);
                    }
                }
            }
            "timeout" => match argument.parse::<u32>() {
                Ok(t) => timeout = t,
                Err(_) => {
                    println!("Failed to parse timeout. Please provide a positive integer number of seconds.");
                    exit(1);
                }
            },
            "solver" => {
                let solver_path = Path::new(argument);
                if !solver_path.exists() {
                    println!("Cannot find solver on your filesystem at location: {}. Please ensure it exists", argument);
                    exit(1);
                }
                if !solver_path.is_executable() {
                    println!("Provided solver is not executable.");
                    exit(1);
                }


                solver = String::from(argument);
                is_set[1] = true;
            }
            "cnf" => {
                is_set[2] = true;
            }
            "output dir" => {}
            "tmp dir" => {}
            "tracked metrics" => {
                is_set[3] = true;
            }
            "evaluation metric" => {
                is_set[4] = true;
            }
            unknown => {
                println!("Unrecognized Config Option: {}", unknown);
                exit(1);
            }
        }
    }

    Config {
        variables,
        comparator,
        timeout,
        solver,
        cnf,
        output_dir,
        tmp_dir,
        tracked_metrics,
        evaluation_metric,
    }
}
