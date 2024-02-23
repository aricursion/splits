use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};

use crate::cube::Cube;

pub fn parse_leaf_cubes(best_log_loc: &str) -> Result<Vec<(Cube, f32)>, io::Error> {
    let log_file = File::open(best_log_loc)?;
    let log_content = BufReader::new(log_file);
    let mut cubes = Vec::new();
    for line in log_content.lines() {
        let line = line?;
        if line.trim().is_empty() {
            break;
        }
        let mut line_split = line.split(':');
        let cube_str = line_split.next().unwrap();
        let cube: Cube = cube_str.parse().unwrap();
        let time_str = line_split.next().unwrap().trim();
        let time = time_str.parse::<f32>().unwrap();
        cubes.push((cube, time));
    }

    let mut leaf_cubes = Vec::new();
    for (cube, time) in &cubes {
        let mut leaf_cube = true;
        for (check_cube, _) in &cubes {
            if *cube == *check_cube {
                continue;
            }
            if cube.subsumes(check_cube) {
                leaf_cube = false;
                break;
            }
        }
        if leaf_cube {
            leaf_cubes.push((cube.clone(), *time));
        }
    }

    Ok(leaf_cubes)
}

pub fn parse_best_log(log_loc: &str, output_loc: &str) -> Result<(), io::Error> {
    let leaves = parse_leaf_cubes(log_loc)?;
    let mut outfile = OpenOptions::new().write(true).create(true).open(output_loc)?;
    for (leaf, _) in leaves {
        outfile.write_all(leaf.to_icnf_string().as_bytes())?;
    }
    Ok(())
}
