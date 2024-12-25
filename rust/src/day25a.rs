use std::{
    collections::HashSet,
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

#[allow(dead_code)]
fn do_it<F>(path: &str, z_func: F) -> Result<u64>
where
    F: Fn(u64, u64) -> u64,
{
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

    let chunks = file_contents.split(|line| line.is_empty()).collect::<Vec<_>>();

    let mut sizes = HashSet::new();
    let mut locks = Vec::new();
    let mut pins = Vec::new();
    for chunk in chunks {
        let height = chunk.len();
        let widths: HashSet<usize> = HashSet::from_iter(chunk.iter().map(|line| line.len()));
        if widths.len() != 1 {
            Err(format!("uneven chunk line lengths: {:?}", widths))?;
        }
        let width = *widths.iter().next().unwrap();
        sizes.insert((width, height));

        let chunk_2d_vec = chunk.iter().map(|line| line.chars().collect::<Vec<_>>()).collect::<Vec<_>>();

        let filled_line = (0..width).map(|_| '#').collect::<String>();
        let empty_line = (0..width).map(|_| '.').collect::<String>();
        let top_line = chunk.first().unwrap();
        let bottom_line = chunk.last().unwrap();
        if *top_line == filled_line && *bottom_line == empty_line {
            locks.push(
                (0..width)
                    .map(|x| (1..height).filter(|y| chunk_2d_vec[*y][x] == '#').count())
                    .collect::<Vec<_>>(),
            );
        } else if *top_line == empty_line && *bottom_line == filled_line {
            pins.push(
                (0..width)
                    .map(|x| (0..(height - 1)).filter(|y| chunk_2d_vec[*y][x] == '#').count())
                    .collect::<Vec<_>>(),
            );
        } else {
            Err("not a lock or a key")?;
        }
    }
    if sizes.len() != 1 {
        Err(format!("not all chunks are the same size: {:?}", sizes))?;
    }
    let (_, height) = *sizes.iter().next().unwrap();
    let max_allowed_height = height - 2;

    let mut result = 0;
    for lock in locks.iter() {
        for pin in pins.iter() {
            if lock.iter().zip(pin.iter()).all(|(a, b)| {
                if a + b > max_allowed_height {
                    // combined lock and pin sizes exceed the bounds, so this isn't a good match
                    false
                } else {
                    // in bounds, this is a possible match
                    true
                }
            }) {
                result += 1;
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day25-sample.txt", |x, y| x & y).unwrap(), 3);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day25.txt", |x, y| x + y).unwrap(), 3127);
    }
}
