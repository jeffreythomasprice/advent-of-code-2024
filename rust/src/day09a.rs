use std::{
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
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
    for i in (1..input.len()).step_by(2) {
        let gap = input[i].to_string().parse::<u64>()?;
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

    let mut blocks = (0..next_position).map(|_| None).collect::<Vec<_>>();
    for f in files.iter() {
        for i in f.position..(f.position + f.len) {
            blocks[i as usize] = Some(f.index);
        }
    }

    let mut from_start = 0;
    let mut from_end = blocks.len() - 1;
    while from_start < from_end {
        if blocks[from_start].is_some() {
            from_start += 1;
            continue;
        }
        if blocks[from_end].is_none() {
            from_end -= 1;
            continue;
        }
        blocks[from_start] = blocks[from_end];
        blocks[from_end] = None;
    }

    Ok(blocks
        .iter()
        .take_while(|block| block.is_some())
        .enumerate()
        .map(|(i, block)| (i as u64) * block.unwrap_or(0))
        .sum())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day09-sample.txt").unwrap(), 1928);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day09.txt").unwrap(), 6398252054886);
    }
}
