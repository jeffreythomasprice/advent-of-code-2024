use std::{
    collections::HashMap,
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    path::Path,
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

#[derive(Debug)]
struct Rule {
    left: u32,
    right: u32,
}

impl Rule {
    fn new(left: &str, right: &str) -> Result<Self> {
        Ok(Self {
            left: left.parse()?,
            right: right.parse()?,
        })
    }
}

fn is_sequence_valid(sequence: &[u32], rules: &HashMap<u32, Vec<&Rule>>) -> bool {
    for i in 0..sequence.len() {
        let value = sequence[i];
        if let Some(rules) = rules.get(&value) {
            for j in (i + 1)..sequence.len() {
                let other_value = sequence[j];
                let rule = rules.iter().find(|rule| rule.right == other_value);
                if rule.is_none() {
                    return false;
                }
            }
        } else if (i + 1) < sequence.len() {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
fn do_it(path: &str) -> Result<u32> {
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

    let divider_regex = Regex::new(r"^(\d+)\|(\d+)$")?;
    let sequence_regex = Regex::new(r"^(\d+)(?:,(\d+))*$")?;

    let mut iter = file_contents.into_iter();
    let rules = iter
        .by_ref()
        .take_while(|line| divider_regex.is_match(line))
        .collect::<Vec<_>>();
    let sequences = iter
        .by_ref()
        .take_while(|line| sequence_regex.is_match(line))
        .collect::<Vec<_>>();
    let remainder = iter.collect::<Vec<_>>();
    if !remainder.is_empty() {
        Err(format!("unmatched line at end of input: {:?}", remainder))?;
    }

    let rules = rules
        .into_iter()
        .map(|line| {
            let (_, [left, right]) = divider_regex
                .captures(&line)
                .ok_or("shold be impossible, already matched")?
                .extract();
            Rule::new(left, right)
        })
        .collect::<Result<Vec<_>>>()?;

    let rules_map = {
        let mut result = HashMap::new();
        for rule in rules.iter() {
            result.entry(rule.left).or_insert_with(Vec::new).push(rule);
        }
        result
    };

    let sequences = sequences
        .into_iter()
        .map(|line| {
            line.split(",")
                .map(|num| Ok(num.trim().parse::<u32>()?))
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(sequences
        .iter()
        .filter(|sequence| is_sequence_valid(sequence, &rules_map))
        .map(|sequence| sequence[sequence.len() / 2])
        .sum())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day05-sample.txt").unwrap(), 143);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day05.txt").unwrap(), 5391);
    }
}
