use crate::cnf::{Cnf, CnfErr};
use is_executable::IsExecutable;
use rayon;
use std::path::Path;
use std::{fmt, fs, io};

#[derive(Debug)]
pub enum Comparator {
    MaxOfMin,
    MinOfMax,
}

impl fmt::Display for Comparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Comparator::MaxOfMin => write!(f, "max of mins"),
            Comparator::MinOfMax => write!(f, "min of maxs"),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub variables: Vec<u32>,
    pub comparator: Comparator,
    pub timeout: u32,
    pub solver: String,
    pub cnf: Cnf,
    pub output_dir: String,
    pub tmp_dir: String,
    pub tracked_metrics: Vec<String>,
    pub evaluation_metric: String,
    pub thread_count: usize,
    pub search_depth: u32,
    pub preserve_cnf: bool,
    pub cutoff_proportion: f32,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut vec_output = Vec::with_capacity(11);
        vec_output.push(format!("Variables: {:?}", self.variables));
        vec_output.push(format!("Comparator: {}", self.comparator));
        vec_output.push(format!("Timeout: {}", self.timeout));
        vec_output.push(format!("Solver: {}", self.solver));
        vec_output.push(format!("CNF: <omitted>"));
        vec_output.push(format!("Output directory: {}", self.output_dir));
        vec_output.push(format!("Temporary directory: {}", self.tmp_dir));
        vec_output.push(format!("Tracked metrics: {:?}", self.tracked_metrics));
        vec_output.push(format!("Evaluation metric: {}", self.evaluation_metric));
        vec_output.push(format!("Thread count: {}", self.thread_count));
        vec_output.push(format!("Search depth: {}", self.search_depth));
        vec_output.push(format!("Preserve CNF: {}", self.preserve_cnf));
        vec_output.push(format!("Cutoff proportion: {}", self.cutoff_proportion));

        let output_str = vec_output.join("\n");
        write!(f, "{}", output_str)
    }
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
        Self(format!("IO Error while seting up: {e}"))
    }
}

impl Config {
    pub fn parse_config(config_string: &str) -> Result<Self, ConfigError> {
        let trimmed_cfg_string = config_string.trim();
        let mut variable_opt = None;
        let mut comparator = Comparator::MinOfMax;
        let mut solver_opt = None;
        let mut timeout = 600;
        let mut cnf_opt = None;
        let mut output_dir = String::from("splits_output_directory");
        let mut tmp_dir = String::from("splits_working_directory");
        let mut tracked_metrics_opt: Option<Vec<_>> = None;
        let mut evaluation_metric_opt = None;
        let mut preserve_cnf = false;

        let mut search_depth = 1;
        let mut thread_count = rayon::max_num_threads();
        let mut cutoff_proportion = 1.0;

        for line in trimmed_cfg_string.lines() {
            let partial_parse_line = line.split(':').collect::<Vec<_>>();
            let name = partial_parse_line[0].trim();
            let argument = partial_parse_line[1].trim();

            match name {
                "variables" => {
                    let mut variable_vec = Vec::new();
                    for var_str in argument.split(' ') {
                        match var_str.parse::<u32>() {
                            Ok(u) => {
                                if u == 0 {
                                    return Err(ConfigError("0 is not a valid cnf variable.".to_string()));
                                } else {
                                    variable_vec.push(u)
                                }
                            }
                            Err(_) => {
                                return Err(ConfigError(format!("Cannot parse {var_str} as a variable. Please make sure they are all positive integers.")));
                            }
                        }
                    }
                    variable_opt = Some(variable_vec);
                }
                "comparator" => match argument {
                    "minmax" => comparator = Comparator::MinOfMax,
                    "maxmin" => comparator = Comparator::MaxOfMin,
                    _ => {
                        return Err(ConfigError(
                            "Failed to recognize Comparison Operator. Please use either 'min' or 'max'.".to_string(),
                        ));
                    }
                },
                "timeout" => match argument.parse::<u32>() {
                    Ok(t) => timeout = t,
                    Err(_) => {
                        return Err(ConfigError(
                            "Failed to parse timeout. Please provide a positive integer number of seconds.".to_string(),
                        ));
                    }
                },
                "solver" => {
                    let solver_path = Path::new(argument);
                    if !solver_path.exists() {
                        return Err(ConfigError(format!(
                            "Cannot find solver on your filesystem at location: {argument}. Please ensure it exists."
                        )));
                    }
                    if !solver_path.is_executable() {
                        return Err(ConfigError("Provided solver is not executable.".to_string()));
                    }

                    solver_opt = Some(String::from(argument));
                }
                "cnf" => {
                    let cnf_path = Path::new(argument);
                    if !cnf_path.exists() {
                        return Err(ConfigError(format!("Cannot find cnf at location {argument}.")));
                    }
                    let cnf_string = fs::read_to_string(cnf_path)?;
                    match cnf_string.parse::<Cnf>() {
                        Ok(c) => cnf_opt = Some(c),
                        Err(CnfErr(s)) => return Err(ConfigError(format!("Failed to parse CNF: {s}"))),
                    }
                }
                "output dir" => {
                    output_dir = argument.to_string();
                }
                "tmp dir" => {
                    tmp_dir = argument.to_string();
                }
                "tracked metrics" => {
                    tracked_metrics_opt = Some(argument.split(' ').map(str::to_string).collect());
                }
                "evaluation metric" => {
                    evaluation_metric_opt = Some(argument.to_string());
                }
                "search depth" => {
                    match argument.parse::<u32>() {
                        Ok(u) => {
                            if u == 0 {
                                return Err(ConfigError("0 is not a valid search depth.".to_string()));
                            } else {
                                search_depth = u;
                            }
                        }
                        Err(_) => {
                            return Err(ConfigError(format!("Cannot parse {argument} as a search depth. Please make sure it is a positive integers.")));
                        }
                    }
                }
                "thread count" => match argument.parse::<usize>() {
                    Ok(u) => {
                        if u == 0 {
                            return Err(ConfigError("0 is not a valid number of threads.".to_string()));
                        } else {
                            thread_count = u;
                        }
                    }
                    Err(_) => {
                        return Err(ConfigError(format!("Cannot parse {argument} as a number of threads. Please make sure it is a positive integers.")));
                    }
                },
                "preserve cnf" => match argument.parse() {
                    Ok(b) => preserve_cnf = b,
                    Err(_) => {
                        return Err(ConfigError(format!(
                            "Cannot parse {argument} as a boolean. Please make sure it is either 'true' or 'false'"
                        )));
                    }
                },
                "cutoff proportion" => match argument.parse() {
                    Ok(f) => {
                        if f <= 0.0 {
                            return Err(ConfigError(format!("Cutoff proportion {f} needs to be a positive float.")));
                        }
                        cutoff_proportion = f
                    },
                    Err(_) => return Err(ConfigError(format!("Cannot parse {argument} as a cutoff proportion. Please make sure it is a positive float"))),

                }
                unknown => {
                    return Err(ConfigError(format!("Unknown config setting {unknown}")));
                }
            }
        }

        let (variables, solver, cnf, tracked_metrics, evaluation_metric) = match (
            variable_opt,
            solver_opt,
            cnf_opt,
            tracked_metrics_opt,
            evaluation_metric_opt,
        ) {
            (None, _, _, _, _) => return Err(ConfigError("Please provide variables in the config.".to_string())),
            (_, None, _, _, _) => {
                return Err(ConfigError(
                    "Please provide the path of the solver in the config.".to_string(),
                ))
            }
            (_, _, None, _, _) => {
                return Err(ConfigError(
                    "Please provide the path of the cnf file in the config.".to_string(),
                ))
            }
            (_, _, _, None, _) => {
                return Err(ConfigError(
                    "Please provide a list of tracked metrics in the config.".to_string(),
                ))
            }
            (_, _, _, _, None) => {
                return Err(ConfigError(
                    "Please provide the evaluation metric in the config".to_string(),
                ))
            }
            (Some(v), Some(s), Some(c), Some(tm), Some(em)) => (v, s, c, tm, em),
        };

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
            thread_count,
            search_depth,
            preserve_cnf,
            cutoff_proportion,
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
