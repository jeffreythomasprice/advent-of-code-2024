use std::{
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

    let file_contents = file_contents.join("");
    let r = Regex::new(r"mul\(([0-9]{1,3}),([0-9]{1,3})\)")?;
    Ok(r.captures_iter(&file_contents)
        .filter_map(|x| {
            let (_, [left, right]) = x.extract();
            let left = left.parse::<u32>().ok()?;
            let right = right.parse::<u32>().ok()?;
            Some(left * right)
        })
        .sum())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day03a-sample.txt").unwrap(), 161);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day03.txt").unwrap(), 192767529);
    }
}
