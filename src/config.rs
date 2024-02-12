use crate::cnf::Cnf;
use crate::cube::Cube;
use crate::wcnf::Wcnf;
use is_executable::IsExecutable;
use std::path::Path;
use std::str::FromStr;
use std::{fmt, fs, io};

#[derive(Debug)]
pub enum Comparator {
    MaxOfMin,
    MinOfMax,
}

#[derive(Debug)]
pub enum SatType {
    Cnf(Cnf),
    Wcnf(Wcnf),
}

impl SatType {
    pub fn extend_cube_str(&self, cube: &Cube) -> String {
        match self {
            SatType::Cnf(c) => c.extend_cube_str(cube),
            SatType::Wcnf(w) => w.extend_cube_str(cube),
        }
    }
}

pub struct SatTypeError(pub String);

impl FromStr for SatType {
    type Err = SatTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<Cnf>() {
            Ok(c) => Ok(SatType::Cnf(c)),
            Err(_) => match s.parse::<Wcnf>() {
                Ok(wcnf) => Ok(SatType::Wcnf(wcnf)),
                Err(_) => Err(SatTypeError("Failed to parse string as CNF or WCNF".to_string())),
            },
        }
    }
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
    pub multitree_variables: Option<Vec<u32>>,
    pub comparator: Comparator,
    pub timeout: u32,
    pub solver: String,
    pub cnf: SatType,
    pub output_dir: String,
    pub tmp_dir: String,
    pub evaluation_metric: String,
    pub thread_count: usize,
    pub search_depth: u32,
    pub preserve_cnf: bool,
    pub preserve_logs: bool,
    pub cutoff_proportion: f32,
    pub time_proportion: f32,
    pub cutoff: f32,
    pub preproc_count: Option<usize>,
    pub debug: bool,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut vec_output = Vec::with_capacity(18);
        let variable_string = if self.variables.len() <= 30 {
            format!("{:?}", self.variables)
        } else {
            "<Omitted>".to_string()
        };
        vec_output.push(format!("          Variables: {variable_string}"));
        vec_output.push(format!("Multitree Variables: {:?}", self.multitree_variables));
        vec_output.push(format!("         Comparator: {}", self.comparator));
        vec_output.push(format!("            Timeout: {}", self.timeout));
        vec_output.push(format!("             Solver: {}", self.solver));
        match self.cnf {
            SatType::Cnf(_) => vec_output.push("           SAT Type: CNF".to_string()),
            SatType::Wcnf(_) => vec_output.push("          SAT Type: WCNF".to_string()),
        }
        vec_output.push(format!("   Output Directory: {}", self.output_dir));
        vec_output.push(format!("Temporary Directory: {}", self.tmp_dir));
        vec_output.push(format!("  Evaluation Metric: {}", self.evaluation_metric));
        vec_output.push(format!("       Thread Count: {}", self.thread_count));
        vec_output.push(format!("       Search Depth: {}", self.search_depth));
        vec_output.push(format!("       Preserve CNF: {}", self.preserve_cnf));
        vec_output.push(format!("      Preserve Logs: {}", self.preserve_logs));
        vec_output.push(format!("  Cutoff Proportion: {}", self.cutoff_proportion));
        vec_output.push(format!("    Time Proportion: {}", self.time_proportion));
        vec_output.push(format!("             Cutoff: {}", self.cutoff));
        vec_output.push(format!("   Preprocess Count: {:?}", self.preproc_count));
        vec_output.push(format!("         Debug Mode: {}", self.debug));

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
        let mut evaluation_metric_opt = None;
        let mut preserve_cnf = false;
        let mut preserve_logs = false;
        let mut multitree_variables = None;

        let mut search_depth = 1;
        let mut thread_count = rayon::current_num_threads();
        let mut cutoff_proportion = 1.0;
        let mut cutoff_opt = None;
        let mut time_proportion = 1.0;

        let mut preproc_count = None;
        let mut debug = false;

        for line in trimmed_cfg_string.lines() {
            let partial_parse_line = line.split(':').collect::<Vec<_>>();

            let lower_name = partial_parse_line[0].to_lowercase();
            let name = lower_name.trim();

            // if the line is a comment or is empty, skip it
            if name.contains('#') || line.trim().is_empty() {
                continue;
            }

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
                "multitree variables" => {
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
                    multitree_variables = Some(variable_vec);
                }

                "comparator" => match argument {
                    "minmax" => comparator = Comparator::MinOfMax,
                    "maxmin" => comparator = Comparator::MaxOfMin,
                    _ => {
                        return Err(ConfigError(
                            "Failed to recognize Comparison Operator. Please use either 'minmax' or 'maxin'."
                                .to_string(),
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
                "wcnf" | "cnf" => {
                    let cnf_path = Path::new(argument);
                    if !cnf_path.exists() {
                        return Err(ConfigError(format!("Cannot find (w)cnf at location {argument}.")));
                    }
                    let cnf_string = fs::read_to_string(cnf_path)?;
                    match cnf_string.parse::<SatType>() {
                        Ok(s) => cnf_opt = Some(s),
                        Err(SatTypeError(s)) => return Err(ConfigError(format!("Failed to parse: {s}"))),
                    }
                }
                "output dir" => {
                    output_dir = argument.to_string();
                }
                "tmp dir" => {
                    tmp_dir = argument.to_string();
                }
                "evaluation metric" => {
                    evaluation_metric_opt = Some(argument.to_string());
                }
                "search depth" => {
                    match argument.parse() {
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
                "thread count" => match argument.parse() {
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
                "preserve cnf" | "preserve wcnf" => match argument.parse() {
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
                            return Err(ConfigError(format!(
                                "Cutoff proportion {f} needs to be a positive float."
                            )));
                        }
                        cutoff_proportion = f
                    }
                    Err(_) => {
                        return Err(ConfigError(format!(
                            "Cannot parse {argument} as a cutoff proportion. Please make sure it is a positive float"
                        )))
                    }
                },
                "time proportion" => match argument.parse() {
                    Ok(f) => {
                        if f <= 0.0 {
                            return Err(ConfigError(format!(
                                "Time proportion {f} needs to be a positive float."
                            )));
                        }
                        time_proportion = f
                    }
                    Err(_) => {
                        return Err(ConfigError(format!(
                            "Cannot parse {argument} as a time proportion. Please make sure it is a positive float"
                        )))
                    }
                },
                "cutoff" => match argument.parse() {
                    Ok(f) => {
                        if f <= 0.0 {
                            return Err(ConfigError(format!("Cutoff {f} needs to be a positive float.")));
                        }
                        cutoff_opt = Some(f);
                    }
                    Err(_) => {
                        return Err(ConfigError(format!(
                            "Cannot parse {argument} as a cutoff. Please make sure it is a positive float"
                        )))
                    }
                },
                "preserve logs" => match argument.parse() {
                    Ok(b) => preserve_logs = b,
                    Err(_) => {
                        return Err(ConfigError(format!(
                            "Cannot parse {argument} as a boolean for preserving logs."
                        )))
                    }
                },
                "preprocess count" => match argument.parse() {
                    Ok(n) => {
                        if n == 0 {
                            return Err(ConfigError(format!(
                                "Preprocess count {n} needs to be a positive number."
                            )));
                        }
                        preproc_count = Some(n);
                    }
                    Err(_) => {
                        return Err(ConfigError(format!(
                            "Cannot parse {argument} as a cutoff. Please make sure it is a positive number"
                        )))
                    }
                },
                "debug" => match argument.parse() {
                    Ok(b) => debug = b,
                    Err(_) => {
                        return Err(ConfigError(format!(
                            "Cannot parse {argument} as a boolean for debugging."
                        )))
                    }
                },
                unknown => {
                    return Err(ConfigError(format!("Unknown config setting: {unknown}")));
                }
            }
        }

        let (variables, solver, cnf, evaluation_metric, cutoff) =
            match (variable_opt, solver_opt, cnf_opt, evaluation_metric_opt, cutoff_opt) {
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
                        "Please provide the evaluation metric in the config".to_string(),
                    ))
                }
                (_, _, _, _, None) => {
                    return Err(ConfigError(
                        "Please make sure there is a cutoff in the config".to_string(),
                    ))
                }
                (Some(v), Some(s), Some(c), Some(em), Some(ct)) => (v, s, c, em, ct),
            };

        Ok(Config {
            variables,
            multitree_variables,
            comparator,
            timeout,
            solver,
            cnf,
            output_dir,
            tmp_dir,
            evaluation_metric,
            thread_count,
            search_depth,
            preserve_cnf,
            cutoff_proportion,
            time_proportion,
            cutoff,
            preserve_logs,
            preproc_count,
            debug,
        })
    }
}
