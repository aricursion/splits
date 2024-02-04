use itertools::Itertools;
use std::fmt::Display;
use std::str::FromStr;

use crate::clause::Clause;
use crate::cube::Cube;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CnfErr(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cnf {
    num_vars: u32,
    num_clauses: usize,
    clauses: Vec<Clause>,
}

impl Display for Cnf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output_str = format!("p cnf {} {}\n", self.num_vars, self.num_clauses);
        for Clause(v) in &self.clauses {
            output_str.push_str(&format!("{} 0\n", &v.iter().map(|x| x.to_string()).join(" ")));
        }
        write!(f, "{}", output_str.trim_start())
    }
}

impl FromStr for Cnf {
    type Err = CnfErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_line(line: &str) -> Result<Clause, CnfErr> {
            let mut vec_line = line.trim().split(' ').collect::<Vec<&str>>();
            match vec_line.pop() {
                Some(x) => match x.parse::<i32>() {
                    Ok(n) => {
                        if n != 0 {
                            return Err(CnfErr("0 not at the end of a CNF line".to_string()));
                        }
                    }
                    Err(_) => return Err(CnfErr("Could not parse variable in CNF line".to_string())),
                },
                None => return Err(CnfErr("Empty cnf line".to_string())),
            };

            match vec_line.iter().map(|x| x.parse()).try_collect() {
                Ok(v) => Ok(Clause(v)),
                Err(_) => Err(CnfErr(format!("Could not parse clause part of CNF line {:#?}", line))),
            }
        }

        let mut cnf = Cnf { num_vars: 0, num_clauses: 0, clauses: Vec::new() };
        let mut largest_variable_seen = 0;

        for line in s.lines() {
            if line.contains("cnf") {
                //first line
                let elts = line.split(' ').collect::<Vec<&str>>();
                if elts[1] != "cnf" {
                    return Err(CnfErr("Header not formatted correctly".to_string()));
                }
                match elts[2].parse::<u32>() {
                    Ok(x) => cnf.num_vars = x,
                    Err(_) => return Err(CnfErr("Failed to parse number of variables".to_string())),
                }

                match elts[3].parse::<usize>() {
                    Ok(x) => cnf.num_clauses = x,
                    Err(_) => return Err(CnfErr("Failed to parse number of clauses".to_string())),
                }
            } else {
                let Clause(v) = parse_line(line)?;
                for var in &v {
                    let abs_var = var.abs();

                    if abs_var > largest_variable_seen {
                        largest_variable_seen = abs_var;
                    }
                }
                cnf.clauses.push(Clause(v));
            }
        }

        if cnf.num_clauses != cnf.clauses.len() {
            return Err(CnfErr(
                "The number of clauses don't agree with the actual number of clauses".to_string(),
            ));
        }

        Ok(cnf)
    }
}

impl Cnf {
    pub fn extend_cube(&mut self, Cube(v): &Cube) {
        for var in v {
            let abs_var = var.unsigned_abs();
            if abs_var > self.num_vars {
                self.num_vars = abs_var;
            }
            self.clauses.push(Clause(vec![*var]));
            self.num_clauses += 1;
        }
    }

    // this function does not update the underlying CNF;
    // It clones but it's fine it needs to allocate the string anyway.
    pub fn extend_cube_str(&self, cube: &Cube) -> String {
        let mut cnf_copy = self.clone();
        cnf_copy.extend_cube(cube);
        cnf_copy.to_string()
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    #[test]
    fn cnf_test_from_to_from() {
        let cnf = Cnf {
            num_vars: 3,
            num_clauses: 3,
            clauses: vec![Clause(vec![]), Clause(vec![2, 3, 1]), Clause(vec![1, 3, -2])],
        };

        assert_eq!(cnf.to_string().parse::<Cnf>().unwrap(), cnf);
    }
    #[test]
    fn random_cnf_to_from_string_tests() {
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let cnf_length = rng.gen_range(1..10);
            let cnf_max_var = rng.gen_range(1..10000);
            let mut cnf = Cnf {
                num_vars: cnf_max_var,
                num_clauses: cnf_length,
                clauses: Vec::with_capacity(cnf_length),
            };
            for _ in 0..cnf_length {
                let cnf_clause_length = rng.gen_range(1..30);
                let mut v = Vec::with_capacity(cnf_clause_length);
                for _ in 0..cnf_clause_length {
                    let x = rng.gen_range(-(cnf_max_var as i32)..(cnf_max_var as i32));
                    if x != 0 {
                        v.push(x);
                    }
                }
                cnf.clauses.push(Clause(v));
            }
            println!("{}", cnf);
            println!("\n");
            assert_eq!(cnf.to_string().parse::<Cnf>().unwrap(), cnf);
            assert_eq!(cnf.to_string().parse::<Cnf>().unwrap().to_string(), cnf.to_string());
        }
    }
}
