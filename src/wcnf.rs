use itertools::Itertools;
use std::fmt::Display;
use std::str::FromStr;

use crate::clause::Clause;
use crate::cube::Cube;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WcnfErr(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wcnf {
    num_vars: u32,
    num_clauses: u32,
    hard_weight: u32,
    clauses: Vec<(u32, Clause)>,
}

impl Display for Wcnf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output_str = format!("p wcnf {} {} {}\n", self.num_vars, self.num_clauses, self.hard_weight);
        for (w, Clause(v)) in &self.clauses {
            output_str.push_str(&format!("{w} {} 0\n", &v.iter().map(|x| x.to_string()).join(" ")));
        }
        write!(f, "{}", output_str.trim_start())
    }
}

impl FromStr for Wcnf {
    type Err = WcnfErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_header(header: &str, wcnf: &mut Wcnf) -> Result<(), WcnfErr> {
            let elts = header.trim().split(' ').collect::<Vec<_>>();
            if !(elts[0] == "p" && elts[1] == "wcnf") {
                return Err(WcnfErr("Header incorrectly formatted".to_string()));
            }

            match (elts[2].parse(), elts[3].parse(), elts[4].parse()) {
                (Ok(num_vars), Ok(num_clauses), Ok(hard_weight)) => {
                    wcnf.num_vars = num_vars;
                    wcnf.num_clauses = num_clauses;
                    wcnf.hard_weight = hard_weight;
                    Ok(())
                }
                _ => Err(WcnfErr(
                    "Failed to correctly parse numerical values in header".to_string(),
                )),
            }
        }

        fn parse_line(line: &str, wcnf: &mut Wcnf) -> Result<u32, WcnfErr> {
            let mut elts = line.trim().split(' ');
            let mut out_vec = Vec::new();
            let mut largest = 0;

            let weight = match elts.next().unwrap().parse::<u32>() {
                Ok(w) => w,
                Err(_) => return Err(WcnfErr(format!("Failed to parse line in wcnf: {}", line))),
            };

            for x in elts {
                match x.parse::<i32>() {
                    Ok(x) => {
                        if x != 0 {
                            out_vec.push(x);
                            if x.abs_diff(0) > largest {
                                largest = x.abs_diff(0);
                            }
                        } else {
                            break;
                        }
                    }
                    Err(_) => return Err(WcnfErr(format!("Failed to parse {x} as a variable in line {line}."))),
                }
            }

            wcnf.clauses.push((weight, Clause(out_vec)));
            Ok(largest)
        }

        let mut wcnf = Wcnf {
            num_vars: 0,
            num_clauses: 0,
            hard_weight: 0,
            clauses: Vec::new(),
        };
        let mut largest_variable_seen = 0;
        let mut lines = s.lines();
        match lines.next() {
            Some(h) => parse_header(h, &mut wcnf)?,
            None => return Err(WcnfErr("Wcnf doesn't have a header".to_string())),
        }
        for line in lines {
            let largest_var_in_line = parse_line(line, &mut wcnf)?;
            if largest_var_in_line > largest_variable_seen {
                largest_variable_seen = largest_var_in_line;
            }
        }

        wcnf.num_vars = largest_variable_seen;
        Ok(wcnf)
    }
}

impl Wcnf {
    pub fn extend_cube(&mut self, Cube(v): &Cube) {
        for var in v {
            let abs_var = var.unsigned_abs();
            if abs_var > self.num_vars {
                self.num_vars = abs_var;
            }
            self.clauses.push((self.hard_weight, Clause(vec![*var])));
            self.num_clauses += 1;
        }
    }

    // this function does not update the underlying CNF;
    // It clones but it's fine it needs to allocate the string anyway.
    pub fn extend_cube_str(&self, cube: &Cube) -> String {
        let mut wcnf_copy = self.clone();
        wcnf_copy.extend_cube(cube);
        wcnf_copy.to_string()
    }
}
