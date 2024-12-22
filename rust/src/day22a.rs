use std::{
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    path::Path,
    str::Utf8Error,
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

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self(format!("core::str::error::Utf8Error({value:?})"))
    }
}

fn multiply_step(input: u64, arg: u64) -> u64 {
    let next = input * arg;
    (input ^ next) % 16777216
}

fn divide_step(input: u64, arg: u64) -> u64 {
    let next = input / arg;
    (input ^ next) % 16777216
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

    let input = file_contents
        .iter()
        .map(|line| Ok(line.parse::<u64>()?))
        .collect::<Result<Vec<_>>>()?;

    let mut result = 0;
    for number in input {
        let mut current = number;
        for _ in 0..2000 {
            let next = multiply_step(current, 64);
            let next = divide_step(next, 32);
            let next = multiply_step(next, 2048);
            current = next;
        }
        result += current;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day22-sample.txt").unwrap(), 37327623);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day22.txt").unwrap(), 17612566393);
    }
}
