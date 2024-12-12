use std::{
    collections::HashMap,
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
    // keys are numbers, values are number of times that number appears
    numbers: HashMap<u64, u64>,
    next: HashMap<u64, u64>,
}

impl List {
    fn new(line: &str) -> Result<List> {
        let mut numbers = HashMap::new();
        for number in line
            .split(" ")
            .map(|x| Ok(x.parse()?))
            .collect::<Result<Vec<_>>>()?
        {
            List::increment(&mut numbers, number, 1);
        }

        Ok(List {
            numbers,
            next: HashMap::new(),
        })
    }

    fn advance(&mut self) -> Result<()> {
        self.next.clear();

        for (number, count) in self.numbers.iter() {
            if *number == 0 {
                List::increment(&mut self.next, 1, *count);
            } else {
                let s = number.to_string();
                let b = s.as_bytes();
                if b.len() % 2 == 0 {
                    let first_half = &b[..(b.len() / 2)];
                    let second_half = &b[(b.len() / 2)..];
                    let first_half = std::str::from_utf8(first_half)?;
                    let second_half = std::str::from_utf8(second_half)?;
                    List::increment(&mut self.next, first_half.parse()?, *count);
                    List::increment(&mut self.next, second_half.parse()?, *count);
                } else {
                    List::increment(&mut self.next, number * 2024, *count);
                }
            }
        }

        swap(&mut self.numbers, &mut self.next);

        Ok(())
    }

    fn len(&self) -> u64 {
        self.numbers.values().sum()
    }

    fn increment(counts: &mut HashMap<u64, u64>, number: u64, times: u64) {
        counts
            .entry(number)
            .and_modify(|existing| *existing += times)
            .or_insert(times);
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

    if file_contents.len() != 1 {
        Err("expected a single line of input")?;
    }

    let mut list = List::new(&file_contents[0])?;
    for _ in 0..75 {
        list.advance()?;
    }
    Ok(list.len())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day11.txt").unwrap(), 220566831337810);
    }
}
