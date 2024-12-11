use std::{
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    mem::swap,
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

struct List {
    numbers: Vec<u64>,
    next: Vec<u64>,
}

impl List {
    fn new(line: &str) -> Result<List> {
        Ok(List {
            numbers: line
                .split(" ")
                .map(|x| Ok(x.parse()?))
                .collect::<Result<Vec<_>>>()?,
            next: Vec::new(),
        })
    }

    fn advance(&mut self) -> Result<()> {
        self.next.clear();

        for x in self.numbers.iter() {
            if *x == 0 {
                self.next.push(1);
            } else {
                let s = x.to_string();
                let b = s.as_bytes();
                if b.len() % 2 == 0 {
                    let first_half = &b[..(b.len() / 2)];
                    let second_half = &b[(b.len() / 2)..];
                    let first_half = std::str::from_utf8(first_half)?;
                    let second_half = std::str::from_utf8(second_half)?;
                    self.next.push(first_half.parse()?);
                    self.next.push(second_half.parse()?);
                } else {
                    self.next.push(x * 2024);
                }
            }
        }

        swap(&mut self.numbers, &mut self.next);

        Ok(())
    }

    fn len(&self) -> usize {
        self.numbers.len()
    }
}

#[allow(dead_code)]
fn do_it(path: &str) -> Result<usize> {
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

    if file_contents.len() != 1 {
        Err("expected a single line of input")?;
    }

    let mut list = List::new(&file_contents[0])?;
    for _ in 0..25 {
        list.advance()?;
    }
    Ok(list.len())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample1() {
        assert_eq!(do_it("day11-sample.txt").unwrap(), 55312);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day11.txt").unwrap(), 186175);
    }
}
