use itertools::Itertools;
use std::{fmt::Display, num::ParseIntError, str::FromStr};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Cube(pub Vec<i32>);

impl FromStr for Cube {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "" {
            return Ok(Cube(Vec::new()));
        }
        let string_reformat = s.replace('n', "-");
        let str_cube = string_reformat.split('_');
        return Ok(Cube(str_cube.map(|s| str::parse::<i32>(s)).try_collect()?));
    }
}

impl Display for Cube {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_cube = self.0.iter().map(|x| x.to_string().replace('-', "n")).join("_");

        write!(f, "{}", str_cube)
    }
}

impl IntoIterator for Cube {
    type Item = i32;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[allow(dead_code)]
impl Cube {
    // this is n^2, but faster for small vectors (like cubes)
    // https://stackoverflow.com/a/64227550/10833363
    pub fn subsumes(&self, Cube(cv1): &Cube) -> bool {
        return self.0.iter().all(|var| cv1.contains(var));
    }
    pub fn extend(&self, var: u32) -> (Cube, Cube) {
        let mut v1 = self.0.clone();
        let mut v2 = self.0.clone();
        v1.push(var as i32);
        v2.push(-(var as i32));
        return (Cube(v1), Cube(v2));
    }

    pub fn extend_vars(&self, vars: Vec<i32>) -> Cube {
        let mut new_vec = self.0.clone();
        new_vec.extend(vars);
        return Cube(new_vec);
    }

    pub fn contains_var(&self, var: i32) -> bool {
        return self.0.iter().any(|x| *x == var);
    }
}

#[cfg(test)]
mod tests {
    use crate::cube::Cube;
    use rand::Rng;

    #[test]
    fn cube_from_to_string() {
        let cube_str = "n120_n5_34_32_1";
        assert_eq!(cube_str, cube_str.parse::<Cube>().unwrap().to_string());
    }

    #[test]
    fn cube_from_to_string2() {
        let cube_str = "";
        assert_eq!(cube_str, cube_str.parse::<Cube>().unwrap().to_string());
    }

    #[test]
    fn cube_to_from_string() {
        let cube = Cube(vec![-34, 29, 32, 91, -4]);
        assert_eq!(cube, cube.to_string().parse::<Cube>().unwrap())
    }

    #[test]
    fn cube_to_from_string2() {
        let cube = Cube(vec![]);
        assert_eq!(cube, cube.to_string().parse::<Cube>().unwrap())
    }

    #[test]
    fn random_cube_to_from_string_tests() {
        let mut rng = rand::thread_rng();
        for _ in 0..10000 {
            let cube_size: u8 = rng.gen_range(0..20);
            let mut cube_vec = Vec::with_capacity(cube_size as usize);
            for _ in 0..cube_size {
                let cube_elt: i32 = rng.gen_range(-1000..1000);
                if cube_elt == 0 {
                    continue;
                }
                cube_vec.push(cube_elt);
            }

            let cube = Cube(cube_vec);
            assert_eq!(cube, cube.to_string().parse::<Cube>().unwrap());
            assert_eq!(cube.to_string(), cube.to_string().parse::<Cube>().unwrap().to_string());
        }
    }
}
