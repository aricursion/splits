use is_executable::IsExecutable;
use std::path::Path;
use std::{fmt, fs, io};

#[derive(Debug)]
pub enum Comparator {
    Max,
    Min,
}
#[allow(dead_code)]
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

#[derive(Debug, Clone)]
pub struct ConfigError(pub String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        Self(format!("IO Error while parsing config: {e}"))
    }
}

impl Config {
    pub fn parse_config(config_string: &str) -> Result<Self, ConfigError> {
        let trimmed_cfg_string = config_string.trim();
        let mut variables = Vec::new();
        let mut comparator = Comparator::Min;
        let mut solver = String::from("");
        let mut timeout = 600;
        let mut cnf = String::from("");
        let mut output_dir = String::from("splits_output_directory");
        let mut tmp_dir = String::from("splits_working_directory");
        let mut tracked_metrics = Vec::new();
        let mut evaluation_metric = String::from("");

        let mut is_set: [bool; 5] = [false, false, false, false, false];

        fn is_set_error(i: u32) -> String {
            let out_str = match i {
                0 => "Please provide variables in the config.",
                1 => "Please provide the path of the solver in the config.",
                2 => "Please provide the path of the cnf file in the config.",
                3 => "Please provide a list of tracked metrics in the config.",
                4 => "Please provide the evaluation metric in the config",
                _ => panic!(),
            };
            return out_str.to_string();
        }
        for line in trimmed_cfg_string.lines() {
            let partial_parse_line = line.split(':').collect::<Vec<&str>>();
            let name = partial_parse_line[0].trim();
            let argument = partial_parse_line[1].trim();

            match name {
                "variables" => {
                    for var_str in argument.split(' ') {
                        match var_str.parse::<u32>() {
                            Ok(u) => {
                                if u == 0 {
                                    return Err(ConfigError(
                                        "0 is not a valid cnf variable.".to_string(),
                                    ));
                                } else {
                                    variables.push(u)
                                }
                            }
                            Err(_) => {
                                return Err(ConfigError(format!("Cannot parse {var_str} as a variable. Please make sure they are all positive integers.")));
                            }
                        }
                    }
                    is_set[0] = true;
                }
                "comparator" => match argument {
                    "min" => comparator = Comparator::Min,
                    "max" => comparator = Comparator::Max,
                    _ => {
                        return Err(ConfigError("Failed to recognize Comparison Operator. Please use either 'min' or 'max'.".to_string()));
                    }
                },
                "timeout" => match argument.parse::<u32>() {
                    Ok(t) => timeout = t,
                    Err(_) => {
                        return Err(ConfigError("Failed to parse timeout. Please provide a positive integer number of seconds.".to_string()));
                    }
                },
                "solver" => {
                    let solver_path = Path::new(argument);
                    if !solver_path.exists() {
                        return Err(ConfigError(format!("Cannot find solver on your filesystem at location: {argument}. Please ensure it exists.")));
                    }
                    if !solver_path.is_executable() {
                        return Err(ConfigError(
                            "Provided solver is not executable.".to_string(),
                        ));
                    }

                    solver = String::from(argument);
                    is_set[1] = true;
                }
                "cnf" => {
                    let cnf_path = Path::new(argument);
                    if !cnf_path.exists() {
                        return Err(ConfigError(format!(
                            "Cannot find cnf at location {argument}."
                        )));
                    }
                    cnf = fs::read_to_string(cnf_path)?;
                    is_set[2] = true;
                }
                "output dir" => {
                    output_dir = argument.to_string();
                }
                "tmp dir" => {
                    tmp_dir = argument.to_string();
                }
                "tracked metrics" => {
                    tracked_metrics = argument.split(' ').map(str::to_string).collect();
                    is_set[3] = true;
                }
                "evaluation metric" => {
                    evaluation_metric = argument.to_string();
                    is_set[4] = true;
                }
                unknown => {
                    return Err(ConfigError(format!("Unknown config setting {unknown}")));
                }
            }
        }
        for i in 0..is_set.len() {
            if !is_set[i] {
                let err_str = is_set_error(i as u32);
                return Err(ConfigError(err_str));
            }
        }

        if !tracked_metrics.contains(&evaluation_metric) {
            return Err(ConfigError(
                "The evaluation metric must appear in the set of tracked metrics.".to_string(),
            ));
        }

        Ok(Config {
            variables,
            comparator,
            timeout,
            solver,
            cnf,
            output_dir,
            tmp_dir,
            tracked_metrics,
            evaluation_metric,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_cfg() {
        let cfg_string = "
            variables: 1 2 3
            comparator: min
            timeout: 100
            solver: testing/test.sh
            cnf: testing/test.cnf
            output dir: output
            tmp dir: tmp
            tracked metrics: seconds ticks
            evaluation metric: seconds";
        assert!(
            Config::parse_config(cfg_string).is_ok(),
            "Config did not parse correctly"
        );
    }

    #[test]
    fn bad_cfgs() {
        let cfg_string1 = "
            variables: 1 2 3 0
            comparator: min
            timeout: 100
            solver: testing/test.sh
            cnf: testing/test.cnf
            output dir: output
            tmp dir: tmp
            tracked metrics: seconds ticks
            evaluation metric: seconds";
        assert!(
            Config::parse_config(cfg_string1).is_err(),
            "Config should not parse correctly due to 0."
        );

        let cfg_string2 = "
            variables: 1 2 3
            comparator: mom
            timeout: 100
            solver: testing/test.sh
            cnf: testing/test.cnf
            output dir: output
            tmp dir: tmp
            tracked metrics: seconds ticks
            evaluation metric: seconds";
        assert!(
            Config::parse_config(cfg_string2).is_err(),
            "Config should not parse correctly due to invalid comparator."
        );
    }
}
