use std::{
    collections::HashSet,
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct Point {
    x: usize,
    y: usize,
}

struct Map {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

impl Map {
    fn new(lines: &[&str]) -> Result<Map> {
        let height = lines.len();
        let width: HashSet<usize> = HashSet::from_iter(lines.iter().map(|line| line.len()));
        if width.len() != 1 {
            Err(format!(
                "expected all lines to be the same length, got {}",
                width.len()
            ))?;
        }
        let width = *width.iter().next().unwrap();
        Ok(Map {
            width,
            height,
            data: lines
                .iter()
                .flat_map(|line| {
                    line.chars().map(|c| {
                        Ok(match c {
                            '0' => 0,
                            '1' => 1,
                            '2' => 2,
                            '3' => 3,
                            '4' => 4,
                            '5' => 5,
                            '6' => 6,
                            '7' => 7,
                            '8' => 8,
                            '9' => 9,
                            _ => Err(format!("unhandled map height: {}", c))?,
                        })
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        })
    }

    fn get(&self, p: Point) -> Option<u8> {
        if p.x < self.width && p.y < self.height {
            Some(self.data[p.y * self.width + p.x])
        } else {
            None
        }
    }

    fn find_all(&self, value: u8) -> Vec<Point> {
        let mut results = Vec::new();
        let mut i = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.data[i] == value {
                    results.push(Point { x, y });
                }
                i += 1;
            }
        }
        results
    }

    fn count_paths(&self, start: Point) -> u32 {
        if let Some(current_value) = self.get(start) {
            if current_value == 9 {
                // we're at the peak, so there's only one way to get to a peak from here
                1
            } else {
                // not at the peak, so iterate over possible neighbors
                [
                    if start.x >= 1 {
                        Some(Point {
                            x: start.x - 1,
                            y: start.y,
                        })
                    } else {
                        None
                    },
                    if start.x + 1 < self.width {
                        Some(Point {
                            x: start.x + 1,
                            y: start.y,
                        })
                    } else {
                        None
                    },
                    if start.y >= 1 {
                        Some(Point {
                            x: start.x,
                            y: start.y - 1,
                        })
                    } else {
                        None
                    },
                    if start.y + 1 < self.height {
                        Some(Point {
                            x: start.x,
                            y: start.y + 1,
                        })
                    } else {
                        None
                    },
                ]
                .iter()
                // keep only the ones that were in bounds
                .filter_map(|p| *p)
                .map(|possible_neighbor| {
                    if let Some(next_value) = self.get(possible_neighbor) {
                        // if we're going up exactly the right amount
                        if current_value + 1 == next_value {
                            // recurse, we have to count paths from there too
                            self.count_paths(possible_neighbor)
                        } else {
                            // not going up
                            0
                        }
                    } else {
                        // off grid
                        0
                    }
                })
                .sum()
            }
        } else {
            // off grid
            0
        }
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

    let map = Map::new(
        &file_contents
            .iter()
            .map(|line| line.as_str())
            .collect::<Vec<_>>(),
    )?;

    Ok(map.find_all(0).iter().map(|p| map.count_paths(*p)).sum())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample2() {
        assert_eq!(do_it("day10-sample2.txt").unwrap(), 81);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day10.txt").unwrap(), 1372);
    }
}
