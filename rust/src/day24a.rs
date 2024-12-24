use std::{
    collections::HashMap,
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    path::Path,
    str::Utf8Error,
};

use regex::Regex;

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

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self(format!("core::str::error::Utf8Error({value:?})"))
    }
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    And,
    Or,
    Xor,
}

#[allow(dead_code)]
fn do_it(path: &str) -> Result<u64> {
    let file_contents = BufReader::new(File::open(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("puzzle-inputs").join(path),
    )?)
    .lines()
    // parse lines
    .map(|line| {
        let line = line?;
        let line = line.trim();
        Ok(line.to_string())
    })
    // break if we have an error
    .collect::<Result<Vec<_>>>()?;

    // ignore empty lines
    let file_contents = file_contents
        .into_iter()
        .filter_map(|line| if line.is_empty() { None } else { Some(line) })
        .collect::<Vec<_>>();

    // key = name, value = initial value
    let mut values = HashMap::new();
    // key = output, value = (input1, input2)
    let mut gates = HashMap::new();

    let input_regex = Regex::new(r"^([a-zA-Z0-9]+): (0|1)$")?;
    let gate_regex = Regex::new(r"^([a-zA-Z0-9]+) (AND|OR|XOR) ([a-zA-Z0-9]+) -> ([a-zA-Z0-9]+)$")?;
    for line in file_contents {
        if let Some(captures) = input_regex.captures(&line) {
            let (_, [name, value]) = captures.extract();
            values.insert(name.to_string(), value == "1");
        } else if let Some(captures) = gate_regex.captures(&line) {
            let (_, [input1, op, input2, output]) = captures.extract();
            let op = match op {
                "AND" => Operation::And,
                "OR" => Operation::Or,
                "XOR" => Operation::Xor,
                _ => Err(format!("invalid operation: {:?}", op))?,
            };
            gates.insert(output.to_string(), (input1.to_string(), op, input2.to_string()));
        } else {
            Err(format!("error parsing line: {:?}", line))?;
        }
    }

    let mut to_remove = Vec::with_capacity(gates.len());
    while !gates.is_empty() {
        to_remove.clear();

        for (name, (input1, op, input2)) in gates.iter() {
            if let Some(result) = match (op, values.get(input1), values.get(input2)) {
                (Operation::And, Some(input1), Some(input2)) => Some(input1 & input2),
                (Operation::Or, Some(input1), Some(input2)) => Some(input1 | input2),
                (Operation::Xor, Some(input1), Some(input2)) => Some(input1 ^ input2),
                _ => None,
            } {
                values.insert(name.clone(), result);
                to_remove.push(name.clone());
            }
        }

        for name in to_remove.iter() {
            gates.remove(name);
        }
    }

    let mut result_values = values.iter().filter(|(name, _)| name.starts_with("z")).collect::<Vec<_>>();
    result_values.sort_by(|(a, _), (b, _)| a.cmp(b));
    let mut shift = 0;
    let mut result = 0;
    for (_, value) in result_values {
        result += if *value { 1 << shift } else { 0 };
        shift += 1;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample1() {
        assert_eq!(do_it("day24-sample1.txt").unwrap(), 4);
    }

    #[test]
    pub fn test_sample2() {
        assert_eq!(do_it("day24-sample2.txt").unwrap(), 2024);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day24.txt").unwrap(), 51410244478064);
    }
}
