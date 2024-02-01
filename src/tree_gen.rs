use crate::config::{
    Comparator::{MaxOfMin, MinOfMax},
    Config,
};
use crate::cube::Cube;

use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::ThreadPool;
use std::collections::{hash_map::Entry, HashMap};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;
use wait_timeout::ChildExt;

fn done_check(config: &Config, cube_vars: &[i32]) -> bool {
    return config
        .variables
        .iter()
        .all(|x| cube_vars.contains(&(*x as i32)) || cube_vars.contains(&(-(*x as i32))));
}

// this destroys v
fn hyper_vec(v: &mut Vec<u32>) -> Vec<Vec<i32>> {
    let mut output: Vec<Vec<i32>> = Vec::new();
    match v.pop() {
        Some(x) => {
            let res = hyper_vec(v);
            for mut mini_hyper in res {
                let mut mini_hyper_copy = mini_hyper.clone();
                mini_hyper.push(x as i32);
                mini_hyper_copy.push(-(x as i32));
                output.push(mini_hyper);
                output.push(mini_hyper_copy)
            }
            output
        }
        None => vec![vec![]],
    }
}

type ClassVecScores = HashMap<Vec<u32>, Vec<(Vec<i32>, Option<f32>)>>;

// this is some garbage code lol
// I should fix this
fn compare(config: &Config, hm: &ClassVecScores, prev_metric: f32) -> Option<Vec<(Vec<i32>, f32)>> {
    let cmp_helper = match config.comparator {
        MaxOfMin => |winning: Vec<(Vec<i32>, f32)>, chal: Vec<(Vec<i32>, f32)>| -> Vec<(Vec<i32>, f32)> {
            let winning_min = winning.iter().map(|x| x.1).reduce(f32::min).unwrap();
            let chal_min = chal.iter().map(|x| x.1).reduce(f32::min).unwrap();
            if winning_min > chal_min {
                return winning;
            }
            chal
        },
        MinOfMax => |winning: Vec<(Vec<i32>, f32)>, chal: Vec<(Vec<i32>, f32)>| -> Vec<(Vec<i32>, f32)> {
            let winning_max = winning.iter().map(|x| x.1).reduce(f32::max).unwrap();
            let chal_max = chal.iter().map(|x| x.1).reduce(f32::max).unwrap();
            if winning_max < chal_max {
                return winning;
            }
            chal
        },
    };

    let candidates = match config.comparator {
        MaxOfMin => hm
            .values()
            .filter(|class_vec| {
                class_vec
                    .iter()
                    .all(|x| x.1.is_some() && x.1.unwrap() > config.cutoff_proportion * prev_metric)
            })
            .collect::<Vec<_>>(),
        MinOfMax => hm
            .values()
            .filter(|class_vec| {
                class_vec
                    .iter()
                    .all(|x| x.1.is_some() && x.1.unwrap() < config.cutoff_proportion * prev_metric)
            })
            .collect::<Vec<_>>(),
    };

    let nice_candidates = candidates.iter().map(|class_vec| {
        class_vec
            .iter()
            .map(|v| (v.0.clone(), v.1.unwrap()))
            .collect::<Vec<_>>()
    });
    nice_candidates.reduce(cmp_helper)
}

fn run_solver(config: &Config, cnf_loc: String, cube: &Cube, prev_time: f32) -> Result<Option<String>, io::Error> {
    let log_file_loc = format!("{}/logs/{}.log", config.output_dir, cube);

    let mut child = Command::new(config.solver.clone())
        .args([&cnf_loc, &log_file_loc])
        .spawn()?;

    let timeout_dur = if config.evaluation_metric == "time" {
        Duration::from_secs_f32(prev_time)
    } else {
        Duration::from_secs(config.timeout as u64)
    };
    // sleep(timeout_dur);
    let id = child.id();

    let res = match child.wait_timeout(timeout_dur)? {
        Some(_) => Ok(Some(log_file_loc)),
        None => {
            // I'm not sure this is correct. Will this kill offset?
            Command::new("pkill").arg(format!("-P {}", id)).output().unwrap();
            Ok(None)
        }
    };

    if !config.preserve_cnf {
        fs::remove_file(cnf_loc)?;
    }

    res
}

fn parse_logs(config: &Config, log_file_location: &str) -> Result<(f32, HashMap<String, f32>), io::Error> {
    let mut log_file = File::open(log_file_location)?;
    let mut lines = String::new();
    log_file.read_to_string(&mut lines)?;
    let json_str = *lines.split("SPLITS DATA").collect::<Vec<_>>().last().unwrap();

    let json: HashMap<String, f32> = serde_json::from_str(json_str.trim())?;

    Ok((*json.get(&config.evaluation_metric).unwrap(), json))
}

pub fn tree_gen(config: &Config, pool: &ThreadPool, ccube: &Cube, prev_metric: f32) -> Result<(), io::Error> {
    let ccube_vec = &ccube.0;

    if done_check(config, ccube_vec) {
        return Ok(());
    }

    let num_valid_split_vars = config.variables.len()
        - ccube
            .0
            .iter()
            .filter(|x| config.variables.contains(&x.unsigned_abs()))
            .count();

    let search_depth = usize::min(num_valid_split_vars, config.search_depth as usize);

    let split_var_vecs = config
        .variables
        .clone()
        .into_iter()
        .combinations(search_depth)
        .collect::<Vec<Vec<u32>>>();

    let mut commands = Vec::new();
    for split_var_vec in &split_var_vecs {
        if split_var_vec
            .iter()
            .any(|x| ccube.contains_var(*x as i32) || ccube.contains_var(-(*x as i32)))
        {
            continue;
        }
        let split_vars_hc = hyper_vec(&mut split_var_vec.clone());
        for split_var_comb in split_vars_hc {
            let split_var_cube = ccube.extend_vars(split_var_comb);
            // println!("split_var_cue: {}", split_var_cube);
            let modified_cnf_str = config.cnf.extend_cube_str(&split_var_cube);
            let modified_cnf_loc = format!("{}/{}.cnf", config.tmp_dir, &split_var_cube);
            let mut modified_cnf_file = File::create(&modified_cnf_loc)?;

            modified_cnf_file.write_all(modified_cnf_str.as_bytes())?;
            commands.push((modified_cnf_loc, split_var_cube))
        }
    }

    let (sender, receiver) = channel();

    pool.install(|| {
        commands.into_par_iter().for_each_with(sender, |s, (cnf_loc, cube)| {
            let res = run_solver(config, cnf_loc, &cube, prev_metric);
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
        let eval_met = match log_loc {
            Ok(Some(log_loc)) => {
                let (eval_met, all_met) = parse_logs(config, &log_loc)?;

                all_log_file.write_all(format!("{}: {:?}\n", cube, all_met).as_bytes())?;
                Some(eval_met)
            }
            Ok(None) => {
                all_log_file.write_all(format!("{}: Timeout\n", cube).as_bytes())?;
                None
            }
            Err(e) => {
                all_log_file.write_all(format!("{}: {}\n", cube, e).as_bytes())?;
                None
            }
        };

        let class = cube.0.iter().rev().take(search_depth).map(|x| x.abs_diff(0)).collect();

        match hm_results.entry(class) {
            Entry::Occupied(mut v) => {
                v.get_mut().push((cube.0, eval_met));
            }
            Entry::Vacant(e) => {
                e.insert(vec![(cube.0, eval_met)]);
            }
        }
    }

    let best_vec = compare(config, &hm_results, prev_metric);
    let mut best_log_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(format!("{}/best.log", config.output_dir))?;

    if !config.preserve_logs {
        fs::remove_dir_all(format!("{}/logs", config.output_dir))?;
        fs::create_dir(format!("{}/logs", config.output_dir))?;
    }

    match best_vec {
        Some(best_vecs) => {
            for v in best_vecs {
                let extension_vars = v.0.into_iter().rev().take(search_depth).collect::<Vec<_>>();
                let new_cube = ccube.extend_vars(extension_vars);

                best_log_file.write_all(format!("{}: {:?}\n", &new_cube, v.1).as_bytes())?;
                match config.comparator {
                    MaxOfMin => {
                        if v.1 < config.cutoff {
                            tree_gen(config, pool, &new_cube, v.1)?
                        }
                    }
                    MinOfMax => {
                        if v.1 > config.cutoff {
                            tree_gen(config, pool, &new_cube, v.1)?
                        }
                    }
                }
            }
        }
        None => {
            println!("Failed to find further split after cube {}", ccube);
        }
    }

    Ok(())
}
