use crate::config::Config;
use crate::cube::Cube;

use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::ThreadPool;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;
use wait_timeout::ChildExt;

fn done_check(config: &Config, cube_vars: &Vec<i32>) -> bool {
    return config.variables.iter().all(|x| cube_vars.contains(&((*x) as i32)));
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
            return output;
        }
        None => return vec![vec![]],
    }
}

fn run_solver(config: &Config, cnf_loc: String, cube: Cube, timeout: f32) -> Result<Option<String>, std::io::Error> {
    let log_file_loc = format!("{}/logs/{}.log", config.output_dir, cube);

    let mut child = Command::new(config.solver.clone())
        .arg(&cnf_loc)
        .arg(&log_file_loc)
        .spawn()?;

    let timeout_dur = Duration::from_secs_f32(timeout);
    // sleep(timeout_dur);
    let id = child.id();

    match child.wait_timeout(timeout_dur)? {
        Some(_) => return Ok(Some(log_file_loc)),
        None => {
            // I'm not sure this is correct. Will this kill offset?
            Command::new("pkill").arg(format!("-P {}", id)).output().unwrap();
            return Ok(None);
        }
    }
}

fn parse_logs(config: &Config, log_file_location: &str) -> f32 {
    return 1.0;
}

pub fn tree_gen(config: &Config, pool: &ThreadPool, ccube: &Cube, prev_metric: f32) {
    let ccube_vec = &ccube.0;

    if done_check(config, ccube_vec) {
        return;
    }

    let split_var_vecs = config
        .variables
        .clone()
        .into_iter()
        .combinations(config.search_depth as usize)
        .collect::<Vec<Vec<u32>>>();

    let mut commands = Vec::new();
    for split_var_vec in split_var_vecs {
        if split_var_vec
            .iter()
            .any(|x| ccube.contains_var(*x as i32) || ccube.contains_var(-(*x as i32)))
        {
            continue;
        }
        let split_vars_hc = hyper_vec(&mut split_var_vec.clone());
        for split_var_comb in split_vars_hc {
            let split_var_cube = Cube(split_var_comb);
            let modified_cnf_str = config.cnf.extend_cube_str(&split_var_cube);
            let modified_cnf_loc = format!("{}/{}.cnf", config.tmp_dir, &split_var_cube);
            let mut modified_cnf_file = File::create(&modified_cnf_loc).unwrap();

            modified_cnf_file.write_all(modified_cnf_str.as_bytes()).unwrap();
            commands.push((modified_cnf_loc.clone(), split_var_cube))
        }
    }

    let (sender, receiver) = channel();
    pool.install(|| {
        commands.into_par_iter().for_each_with(sender, |s, (cnf_loc, cube)| {
            s.send(run_solver(config, cnf_loc, cube, prev_metric)).unwrap()
        })
    });

    let log_locations = receiver.iter().collect::<Vec<_>>();
    println!("{:?}", log_locations);
}

#[cfg(test)]
mod tests {
    use crate::tree_gen::hyper_vec;

    #[test]
    fn hyper_test() {
        let mut starting_cube = vec![1, 2];
        println!("{:?}", hyper_vec(&mut starting_cube));
    }
}
