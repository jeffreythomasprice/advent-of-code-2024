use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    ops::{Add, Sub},
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
struct PuzzleFile {
    index: u64,
    position: u64,
    len: u64,
}

#[derive(Debug)]
struct Gap {
    position: u64,
    len: u64,
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

    let input = file_contents
        .join("")
        .trim()
        .to_string()
        .chars()
        .collect::<Vec<_>>();
    let mut files = Vec::new();
    files.push(PuzzleFile {
        index: 0,
        position: 0,
        len: input[0].to_string().parse()?,
    });
    let mut next_index = 1;
    let mut next_position = files[0].len;
    let mut gaps = Vec::new();
    for i in (1..input.len()).step_by(2) {
        let gap = input[i].to_string().parse::<u64>()?;
        gaps.push(Gap {
            position: next_position,
            len: gap,
        });
        next_position += gap;

        let size = input[i + 1].to_string().parse::<u64>()?;
        files.push(PuzzleFile {
            index: next_index,
            position: next_position,
            len: size,
        });
        next_index += 1;
        next_position += size;
    }

    // iterate in reverse order
    for file in files.iter_mut().rev() {
        // find the first gap big enough to fit this file
        if let Some(gap) = gaps
            .iter_mut()
            .find(|gap| gap.position < file.position && gap.len >= file.len)
        {
            // move the file
            file.position = gap.position;
            // fix the gap
            if gap.len > file.len {
                // gap was bigger, so shrink it
                gap.position += file.len;
            }
            gap.len -= file.len;
        }
    }

    Ok(files
        .iter()
        .map(|f| (f.position..(f.position + f.len)).map(|i| i * f.index))
        .flatten()
        .sum())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day09-sample.txt").unwrap(), 2858);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day09.txt").unwrap(), 6415666220005);
    }
}
