use crate::config::{
    Comparator::{MaxOfMin, MinOfMax},
    Config,
};
use crate::cube::{neg_var, pos_var, Cube};

use std::cmp::Ordering;
use std::collections::{hash_map::Entry, HashMap};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::process::{exit, Command};
use std::sync::mpsc::channel;
use std::time::Duration;

use itertools::Itertools;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::ThreadPool;
use wait_timeout::ChildExt;

fn done_check(variables: &[u32], cube_vars: &[i32]) -> bool {
    return variables.iter().all(|x| Cube(cube_vars.to_vec()).contains_var(*x));
}

pub fn hyper_vec(v: &[u32]) -> Vec<Vec<i32>> {
    fn helper(mut v: Vec<u32>) -> Vec<Vec<i32>> {
        let mut output: Vec<Vec<i32>> = Vec::new();
        match v.pop() {
            Some(x) => {
                let res = helper(v);
                for mut mini_hyper in res {
                    let mut mini_hyper_copy = mini_hyper.clone();
                    mini_hyper.push(pos_var(x));
                    mini_hyper_copy.push(neg_var(x));
                    output.push(mini_hyper);
                    output.push(mini_hyper_copy)
                }
                output
            }
            None => vec![vec![]],
        }
    }
    helper(v.to_vec())
}

#[derive(Debug)]
struct VecScore {
    cube: Vec<i32>,
    eval_met: Option<f32>,
    runtime: Option<f32>,
}

/// The keys are "class vectors" which are just cubes.
/// The output is a vector which contains
/// (the element of the class, the evaluation metric, and the time)
type ClassVecScores = HashMap<Vec<u32>, Vec<VecScore>>;

/// Takes in the class vec scores and returns the best scores being a vector of
/// (the vector with the score, the metric, and the runtime)
fn best_class_vec(config: &Config, hm: &ClassVecScores, prev_metric: f32) -> Option<Vec<(Vec<i32>, f32, f32)>> {
    let cmp_helper = match config.comparator {
        MaxOfMin => {
            |winning: Vec<(Vec<i32>, f32, f32)>, chal: Vec<(Vec<i32>, f32, f32)>| -> Vec<(Vec<i32>, f32, f32)> {
                let winning_min = winning.iter().map(|x| x.1).reduce(f32::min).unwrap();
                let chal_min = chal.iter().map(|x| x.1).reduce(f32::min).unwrap();
                if winning_min > chal_min {
                    return winning;
                }
                chal
            }
        }
        MinOfMax => {
            |winning: Vec<(Vec<i32>, f32, f32)>, chal: Vec<(Vec<i32>, f32, f32)>| -> Vec<(Vec<i32>, f32, f32)> {
                let winning_max = winning.iter().map(|x| x.1).reduce(f32::max).unwrap();
                let chal_max = chal.iter().map(|x| x.1).reduce(f32::max).unwrap();
                if winning_max < chal_max {
                    return winning;
                }
                chal
            }
        }
    };

    let candidates = match config.comparator {
        MaxOfMin => hm
            .values()
            .filter(|class_vec| {
                class_vec.iter().all(|vec_score| {
                    vec_score.eval_met.is_some() && vec_score.eval_met.unwrap() > config.cutoff_proportion * prev_metric
                })
            })
            .collect::<Vec<_>>(),
        MinOfMax => hm
            .values()
            .filter(|class_vec| {
                class_vec.iter().all(|vec_score| {
                    vec_score.eval_met.is_some() && vec_score.eval_met.unwrap() < config.cutoff_proportion * prev_metric
                })
            })
            .collect::<Vec<_>>(),
    };

    let nice_candidates = candidates.iter().map(|class_vec| {
        class_vec
            .iter()
            .map(|v| (v.cube.clone(), v.eval_met.unwrap(), v.runtime.unwrap()))
            .collect::<Vec<_>>()
    });
    nice_candidates.reduce(cmp_helper)
}

/// Takes in the class vec scores and returns them sorted from
/// best to worst in a vec of
/// (class, the scores for the elements in its hc)
fn sort_class_vecs(config: &Config, hm: &ClassVecScores) -> Vec<Vec<u32>> {
    let cv_scores = hm.iter();
    #[allow(clippy::type_complexity)]
    let (in_cmp, out_cmp): (fn(f32, f32) -> f32, fn(f32, f32) -> Ordering) = match config.comparator {
        MaxOfMin => {
            let in_cmp = f32::min;
            let out_cmp = (|x, y| f32::total_cmp(&y, &x)) as fn(f32, f32) -> Ordering;
            (in_cmp, out_cmp)
        }
        MinOfMax => {
            let in_cmp = f32::max;
            let out_cmp = (|x, y| f32::total_cmp(&x, &y)) as fn(f32, f32) -> Ordering;
            (in_cmp, out_cmp)
        }
    };
    let cv_scores_unified = {
        // turn a vec of vecscores into a single score
        let map_helper = |(class_vec, vec_scores): (&Vec<u32>, &Vec<VecScore>)| {
            let fold_helper = |acc: Option<f32>, next: &VecScore| match (acc, next.eval_met) {
                (None, None) => None,
                (None, Some(y)) => Some(y),
                (Some(x), None) => Some(x),
                (Some(x), Some(y)) => Some(in_cmp(x, y)),
            };

            let unified_vec_scores = vec_scores.iter().fold(None, fold_helper);
            (class_vec.clone(), unified_vec_scores)
        };

        cv_scores.map(map_helper)
    };

    let sorted_scores = cv_scores_unified.sorted_by(|(_, cvs1), (_, cv2s)| match (cvs1, cv2s) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(x), Some(y)) => out_cmp(*x, *y),
    });
    sorted_scores.map(|(vec, _)| vec).collect_vec()
}

fn run_solver(config: &Config, cube: &Cube, timeout_time: f32) -> Result<Option<String>, io::Error> {
    let cnf_str = config.cnf.extend_cube_str(cube);
    let cnf_loc = format!("{}/{}.cnf", config.tmp_dir, cube);
    let mut cnf_file = File::create(&cnf_loc)?;
    cnf_file.write_all(cnf_str.as_bytes())?;

    let log_file_loc = format!("{}/logs/{}.log", config.output_dir, cube);

    let mut child = Command::new(&config.solver).args([&cnf_loc, &log_file_loc]).spawn()?;

    let timeout_dur = Duration::from_secs_f32(timeout_time);

    let tc = child.wait_timeout(timeout_dur)?;

    let res = match tc {
        Some(_) => Ok(Some(log_file_loc)),
        None => {
            signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM)?;
            child.wait()?;
            Ok(None)
        }
    };

    if !config.preserve_cnf {
        fs::remove_file(cnf_loc)?;
    }

    res
}
/// The input is a log file and it outputs a pair
/// consisting of the evaluation metric, and the
/// map of all the metrics. In particular, "time"
/// has to be in the hashmap for things to work.
fn parse_logs(config: &Config, log_file_location: &str) -> Result<(f32, HashMap<String, f32>), io::Error> {
    let mut log_file = File::open(log_file_location)?;
    let mut lines = String::new();
    log_file.read_to_string(&mut lines)?;
    let json_str = *lines.split("SPLITS DATA").collect::<Vec<_>>().last().unwrap();

    let json: HashMap<String, f32> = serde_json::from_str(json_str.trim())?;

    match json.get(&config.evaluation_metric) {
        Some(x) => Ok((*x, json)),
        None => {
            println!(concat!(
                "The evaluation metric did not appear in the output of the ",
                "solver (or something else went wrong, but it's probably that). ",
                "Please make sure it appears exactly as written in the config."
            ));
            exit(1)
        }
    }
}

pub fn tree_gen(
    config: &Config,
    pool: &ThreadPool,
    ccube: &Cube,
    variables: &[u32],
    prev_metric: f32,
    prev_time: f32,
    depth: u32,
) -> Result<(), io::Error> {
    println!("At cube {} with variables {:?}", ccube, variables);
    let ccube_vec = &ccube.0;

    if done_check(variables, ccube_vec) {
        println!("Failed to find better cube after {}. Ran out of variables.", ccube);
        return Ok(());
    }

    let num_valid_split_vars =
        variables.len() - ccube.0.iter().filter(|x| variables.contains(&x.unsigned_abs())).count();

    let search_depth = usize::min(num_valid_split_vars, config.search_depth as usize);
    let split_var_vecs = variables
        .iter()
        .copied()
        .combinations(search_depth)
        .collect::<Vec<Vec<u32>>>();

    let mut commands = Vec::new();
    for split_var_vec in &split_var_vecs {
        if split_var_vec.iter().any(|x| ccube.contains_var(*x)) {
            continue;
        }
        let split_vars_hc = hyper_vec(split_var_vec);
        for split_var_comb in split_vars_hc {
            let split_var_cube = ccube.extend_vars(split_var_comb);
            commands.push(split_var_cube)
        }
    }

    let (sender, receiver) = channel();
    let timeout_time = prev_time * config.time_proportion;
    pool.install(|| {
        commands.into_par_iter().for_each_with(sender, |s, cube| {
            let res = run_solver(config, &cube, timeout_time);
            s.send((cube, res)).unwrap()
        })
    });

    let solver_results = receiver.iter();
    let mut hm_results: ClassVecScores = HashMap::new();

    let mut all_log_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(format!("{}/all.log", config.output_dir))?;

    for (cube, log_loc) in solver_results {
        // println!("Writing {cube} to all.log");
        let (eval_met, time) = match log_loc {
            Ok(Some(log_loc)) => {
                let (eval_met, all_met) = parse_logs(config, &log_loc)?;

                all_log_file.write_all(format!("{}: {:?}\n", cube, all_met).as_bytes())?;
                (Some(eval_met), Some(*all_met.get("time").unwrap()))
            }
            Ok(None) => {
                all_log_file.write_all(format!("{}: Timeout\n", cube).as_bytes())?;
                (None, None)
            }
            Err(e) => {
                all_log_file.write_all(format!("{}: {}\n", cube, e).as_bytes())?;
                (None, None)
            }
        };

        let class = cube
            .0
            .iter()
            .rev()
            .take(search_depth)
            .rev()
            .map(|x| x.unsigned_abs())
            .collect();

        match hm_results.entry(class) {
            Entry::Occupied(mut v) => {
                v.get_mut().push(VecScore { cube: cube.0, eval_met, runtime: time });
            }
            Entry::Vacant(e) => {
                e.insert(vec![VecScore { cube: cube.0, eval_met, runtime: time }]);
            }
        }
    }

    let new_variables = match config.prune_pct {
        Some(pct) => {
            if config.prune_depth > depth {
                let sorted_class_vecs = sort_class_vecs(config, &hm_results);
                println!("sorted vec {:?}", sorted_class_vecs);
                // get all the variables (removing non-unqiue and skipping the ones in the cc)

                let sorted_vars = sorted_class_vecs
                    .into_iter()
                    .flatten()
                    .unique()
                    .skip(search_depth)
                    .collect_vec();
                println!("{:?}", sorted_vars);
                let num_vars = sorted_vars.len();
                sorted_vars
                    .into_iter()
                    .take(((num_vars as f32) * pct) as usize)
                    .collect()
            } else {
                variables.to_vec()
            }
        }
        None => variables.to_vec(),
    };

    let best_class_vecs = best_class_vec(config, &hm_results, prev_metric);
    let mut best_log_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(format!("{}/best.log", config.output_dir))?;

    if !config.preserve_logs {
        fs::remove_dir_all(format!("{}/logs", config.output_dir))?;
        fs::create_dir(format!("{}/logs", config.output_dir))?;
    }

    match best_class_vecs {
        Some(best_vecs) => {
            for v in best_vecs {
                let extension_vars = v.0.into_iter().rev().take(search_depth).rev().collect::<Vec<_>>();
                let new_cube = ccube.extend_vars(extension_vars);

                best_log_file.write_all(format!("{}: {:?}\n", &new_cube, v.1).as_bytes())?;
                if depth + 1 >= config.max_depth {
                    println!("Hit max depth with cube {}", new_cube);
                } else {
                    match config.comparator {
                        MaxOfMin => {
                            if v.1 < config.cutoff {
                                tree_gen(config, pool, &new_cube, &new_variables, v.1, v.2, depth + 1)?
                            }
                        }
                        MinOfMax => {
                            if v.1 > config.cutoff {
                                tree_gen(config, pool, &new_cube, &new_variables, v.1, v.2, depth + 1)?
                            }
                        }
                    }
                }
            }
        }
        None => {
            println!(
                "Failed to find further split after cube {}. Could not find better split",
                ccube
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hyper_vec_test() {
        let base = vec![1, 2, 3];
        println!("{:?}", hyper_vec(&base));
    }
}
