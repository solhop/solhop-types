use crate::{Lit, Var};
use regex::Regex;
use std::io::BufRead;

/// Dimacs formula.
#[derive(Debug, PartialEq, Clone)]
pub enum Dimacs {
    /// Unweighted formula.
    Cnf {
        /// Number of variables.
        n_vars: usize,
        /// Clauses.
        clauses: Vec<Vec<Lit>>,
    },
    /// Weighted formula.
    Wcnf {
        /// Number of variables.
        n_vars: usize,
        /// Clauses with their weights.
        clauses: Vec<(Vec<Lit>, u64)>,
        /// Weight corresponding to hard clause.
        hard_weight: Option<u64>,
    },
}

/// Parse dimacs from buffer reader.
pub fn parse_dimacs_from_buf_reader<F>(reader: &mut F) -> Dimacs
where
    F: std::io::BufRead,
{
    let mut n_clauses = 0usize;
    let mut n_vars = 0usize;
    let mut clauses = vec![];
    let mut weights: Vec<u64> = vec![];
    let mut hard_weight = None;
    let mut is_wcnf = false;

    for line in reader.lines() {
        let line = line.unwrap();
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('c') {
            continue;
        } else if line.starts_with('p') {
            let re_cnf = Regex::new(r"^p\s+cnf\s+(\d+)\s+(\d+)").unwrap();
            let re_wcnf = Regex::new(r"^p\s+wcnf\s+(\d+)\s+(\d+)(?:\s+(\d+))?").unwrap();
            if let Some(cap) = re_cnf.captures(&line) {
                n_vars = cap[1].parse().unwrap();
                n_clauses = cap[2].parse().unwrap();
            } else if let Some(cap) = re_wcnf.captures(&line) {
                is_wcnf = true;
                n_vars = cap[1].parse().unwrap();
                n_clauses = cap[2].parse().unwrap();
                hard_weight = cap.get(3).map(|m| m.as_str().parse().unwrap()); // cap[3].parse().unwrap();
            }
        } else {
            let re = Regex::new(r"(-?\d+)").unwrap();
            let mut cl = vec![];
            let mut weight = 0u64;
            for (i, cap) in re.captures_iter(&line).enumerate() {
                if i == 0 && is_wcnf {
                    weight = cap[1].parse::<u64>().unwrap();
                    continue;
                }
                let l = match cap[1].parse::<i32>().unwrap() {
                    0 => continue,
                    n => n,
                };
                let var = Var::new((l.abs() - 1) as usize);
                let lit = if l > 0 { var.pos_lit() } else { var.neg_lit() };
                cl.push(lit);
            }
            clauses.push(cl);
            weights.push(weight);
            if clauses.len() == n_clauses {
                break;
            }
        }
    }

    if is_wcnf {
        Dimacs::Wcnf {
            n_vars,
            clauses: clauses.into_iter().zip(weights).collect(),
            hard_weight,
        }
    } else {
        Dimacs::Cnf { n_vars, clauses }
    }
}

/// Parse a cnf/wcnf dimacs file.
pub fn parse_dimacs_from_file(filename: &std::path::Path) -> Dimacs {
    let file = std::fs::File::open(filename).expect("File not found");
    let mut reader = std::io::BufReader::new(file);
    parse_dimacs_from_buf_reader(&mut reader)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let wcnf = "p wcnf 1 2\n\
        2 1 0\n\
        3 -1 0
        ";
        let var_1 = Var::new(0);
        assert_eq!(
            parse_dimacs_from_buf_reader(&mut std::io::BufReader::new(wcnf.as_bytes())),
            Dimacs::Wcnf {
                n_vars: 1,
                hard_weight: None,
                clauses: vec![(vec![var_1.pos_lit()], 2), (vec![var_1.neg_lit()], 3)]
            }
        );
    }
}
