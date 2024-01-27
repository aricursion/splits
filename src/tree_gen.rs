use crate::cnf;
use crate::config::Config;
use crate::cube::Cube;
use itertools::Itertools;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use rayon::ThreadPool;
use std::fs::{self, remove_file, File};
use std::io::prelude::*;
use std::process::Command;

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

fn run_solver(config: &Config, solver: String, cnf_loc: String, cube: Cube) {
    let output = Command::new(solver).arg(&cnf_loc).output().unwrap();

    let mut log_file = File::create(format!("{}/logs/{}.log", config.output_dir, cube)).unwrap();
    log_file.write_all(&output.stdout).unwrap();

    if !config.preserve_cnf {
        fs::remove_file(cnf_loc).unwrap();
    }
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
            commands.push((config.solver.clone(), modified_cnf_loc.clone(), split_var_cube))
        }
    }
    pool.install(|| {
        commands
            .into_par_iter()
            .for_each(|(com, cnf_loc, cube)| run_solver(config, com, cnf_loc, cube))
    });
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
