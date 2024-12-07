use std::{
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    ops::Index,
    path::Path,
};

#[derive(Debug, Clone)]
struct Error(#[allow(dead_code)] String);

type Result<T> = std::result::Result<T, Error>;

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self(format!("std::io::Error({value:?})"))
    }
}

impl From<regex::Error> for Error {
    fn from(value: regex::Error) -> Self {
        Self(format!("regex::Error({value:?})"))
    }
}

impl From<ParseIntError> for Error {
    fn from(value: core::num::ParseIntError) -> Self {
        Self(format!("core::num::ParseIntError({value:?})"))
    }
}

#[derive(Debug)]
struct Line {
    answer: u64,
    values: Vec<u64>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Operator {
    Add,
    Multiply,
    Concat,
}

struct Operators {
    operators: Vec<Operator>,
}

impl Line {
    fn new(s: &str) -> Result<Line> {
        match s.split(":").collect::<Vec<_>>().as_slice() {
            &[answer, values] => {
                let answer = answer.parse()?;
                let values = values
                    .split(" ")
                    .filter_map(|value| {
                        let value = value.trim();
                        if value.is_empty() {
                            None
                        } else {
                            Some(value.parse())
                        }
                    })
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Ok(Self { answer, values })
            }
            _ => Err("wrong number of splits on \":\"".to_string())?,
        }
    }

    fn is_solvable(&self) -> Result<bool> {
        let mut operators = Operators::new(self)?;
        for _ in 0..3u32.pow((self.values.len() - 1) as u32) {
            if self.is_solution(&operators)? {
                return Ok(true);
            }
            operators.next();
        }
        Ok(false)
    }

    fn is_solution(&self, operators: &Operators) -> Result<bool> {
        let mut result = self.values[0];
        for i in 1..self.values.len() {
            let left = result;
            let right = self.values[i];
            result = match operators[i - 1] {
                Operator::Add => left + right,
                Operator::Multiply => left * right,
                Operator::Concat => {
                    let left = left.to_string();
                    let right = right.to_string();
                    (left + &right).parse()?
                }
            };
            if result > self.answer {
                return Ok(false);
            }
        }
        Ok(result == self.answer)
    }
}

impl Operators {
    fn new(line: &Line) -> Result<Self> {
        if line.values.is_empty() {
            Err("line is empty, no values")?;
        }

        let len = line.values.len() - 1;
        Ok(Self {
            operators: (0..len).map(|_| Operator::Add).collect(),
        })
    }

    fn next(&mut self) {
        for i in 0..self.operators.len() {
            let next = match self.operators[i] {
                Operator::Add => Operator::Multiply,
                Operator::Multiply => Operator::Concat,
                Operator::Concat => Operator::Add,
            };
            self.operators[i] = next;
            if next != Operator::Add {
                break;
            }
        }
    }
}

impl Index<usize> for Operators {
    type Output = Operator;

    fn index(&self, index: usize) -> &Self::Output {
        &self.operators[index]
    }
}

#[allow(dead_code)]
fn do_it(path: &str) -> Result<u64> {
    let file_contents = BufReader::new(File::open(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("puzzle-inputs")
            .join(path),
    )?)
    .lines()
    // parse lines
    .map(|line| {
        // ignore empty lines
        let line = line?;
        let line = line.trim();
        Ok(line.to_string())
    })
    // break if we have an error
    .collect::<Result<Vec<_>>>()?;

    let lines = file_contents
        .iter()
        .map(|line| Line::new(line))
        .collect::<Result<Vec<_>>>()?;

    Ok(lines
        .iter()
        .map(|line| Ok(if line.is_solvable()? { line.answer } else { 0 }))
        .collect::<Result<Vec<_>>>()?
        .iter()
        .sum())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day07-sample.txt").unwrap(), 11387);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day07.txt").unwrap(), 145397611075341);
    }
}
