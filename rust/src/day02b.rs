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

fn is_good(line: &[i32]) -> bool {
    /*
    rules:
    - all increasing or all decreasing
    - no delta outside the range [1, 3]
    */

    let mut increasing = 0;
    let mut decreasing = 0;
    let mut all_in_range = true;
    for i in 0..(line.len() - 1) {
        let a = line[i];
        let b = line[i + 1];
        let delta = b - a;

        // find out what direction we're going
        if delta > 0 {
            increasing += 1;
        } else if delta < 0 {
            decreasing += 1;
        }

        // find out if we're in range
        let delta = delta.abs();
        if delta < 1 || delta > 3 {
            all_in_range = false;
        }
    }

    if increasing > 0 && decreasing > 0 {
        false
    } else if !all_in_range {
        false
    } else {
        true
    }
}

fn duplicate_without_index<T>(source: &[T], to_remove: usize) -> Vec<T>
where
    T: Clone,
{
    source
        .iter()
        .enumerate()
        .filter_map(|(i, x)| {
            if i != to_remove {
                Some(x.clone())
            } else {
                None
            }
        })
        .collect()
}

#[allow(dead_code)]
fn do_it(path: &str) -> Result<u32> {
    let r = Regex::new(r"\s+")?;
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
        if line.is_empty() {
            Ok(None)
        } else {
            Ok(Some(
                r.split(line)
                    .map(|s| Ok(s.to_string().parse::<i32>()?))
                    .collect::<Result<Vec<_>>>()?,
            ))
        }
    })
    // break if we have an error
    .collect::<Result<Vec<_>>>()?
    .into_iter()
    // remove empty lines
    .flatten()
    .collect::<Vec<_>>();

    Ok(file_contents
        .into_iter()
        .filter(|line| {
            if is_good(&line) {
                true
            } else {
                for i in 0..line.len() {
                    let new_line = duplicate_without_index(line, i);
                    if is_good(&new_line) {
                        return true;
                    }
                }
                false
            }
        })
        .count() as u32)
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day02-sample.txt").unwrap(), 4);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day02.txt").unwrap(), 612);
    }
}