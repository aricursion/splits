use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};

use crate::cube::Cube;

fn parse_leaf_cubes(log_loc: &str) -> Result<Vec<Cube>, io::Error> {
    let log_file = File::open(log_loc)?;
    let log_content = BufReader::new(log_file);
    let mut cubes = Vec::new();
    for line in log_content.lines() {
        let line = line?;
        if line.trim().is_empty() {
            break;
        }
        let cube_str = line.split(':').next().unwrap();
        let cube: Cube = cube_str.parse().unwrap();
        cubes.push(cube);
    }

    let mut leaf_cubes = Vec::new();
    println!("{:?}", cubes);
    for cube in &cubes {
        let mut leaf_cube = true;
        for check_cube in &cubes {
            if *cube == *check_cube {
                continue;
            }
            if cube.subsumes(check_cube) {
                leaf_cube = false;
                break;
            }
        }
        if leaf_cube {
            leaf_cubes.push(cube.clone());
        }
    }

    Ok(leaf_cubes)
}

pub fn parse_logs(log_loc: &str, output_loc: &str) -> Result<(), io::Error> {
    let leaves = parse_leaf_cubes(log_loc)?;
    let mut outfile = OpenOptions::new().write(true).create(true).open(output_loc)?;
    for leaf in leaves {
        let out_line = format!("a {} 0\n", leaf.to_string().replace('_', " ").replace('n', "-"));
        outfile.write_all(out_line.as_bytes())?;
    }
    Ok(())
}
