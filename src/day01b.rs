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

#[allow(dead_code)]
fn do_it(path: &str) -> Result<u32> {
    let r = Regex::new(r"^(\d+)\s+(\d+)$")?;
    let file_contents = BufReader::new(File::open(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("puzzle-inputs")
            .join(path),
    )?)
    .lines()
    // parse lines
    .map(|line| {
        // ignore empty lines
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            Ok(None)
        } else {
            let captures = r.captures(line).ok_or(format!("bad line: {line}"))?;
            let (_, [left, right]) = captures.extract();
            Ok(Some((left.to_string(), right.to_string())))
        }
    })
    // break if we have an error
    .collect::<Result<Vec<_>>>()?
    .into_iter()
    // remove empty lines
    .flatten()
    .collect::<Vec<_>>();

    // split
    let (left, right): (Vec<String>, Vec<String>) = file_contents.into_iter().unzip();

    // parse into ints
    let left = left
        .into_iter()
        .map(|x| Ok(x.parse::<u32>()?))
        .collect::<Result<Vec<_>>>()?;
    let right = right
        .into_iter()
        .map(|x| Ok(x.parse::<u32>()?))
        .collect::<Result<Vec<_>>>()?;

    // count how often each number in the right list appears
    let counts = right.into_iter().fold(HashMap::new(), |mut result, x| {
        let count = match result.get(&x) {
            Some(existing) => existing + 1,
            None => 1,
        };
        result.insert(x, count);
        result
    });

    // multiply each number by the count and sum
    Ok(left
        .into_iter()
        .map(|x| match counts.get(&x) {
            Some(count) => x * count,
            None => 0,
        })
        .sum())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day01b-sample.txt").unwrap(), 31);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day01b.txt").unwrap(), 27267728);
    }
}
