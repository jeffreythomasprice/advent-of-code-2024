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

#[derive(Debug, Clone, Copy)]
struct Point {
    x: usize,
    y: usize,
}

struct Map {
    width: usize,
    height: usize,
    data: Vec<char>,
}

impl Map {
    fn new(lines: &[&str]) -> Result<Map> {
        let height = lines.len();
        let width: HashSet<usize> = HashSet::from_iter(lines.iter().map(|line| line.len()));
        if width.len() != 1 {
            Err(format!(
                "expected all lines to the same length, got {:?}",
                width
            ))?;
        }
        let width = *width.iter().next().unwrap();
        Ok(Map {
            width,
            height,
            data: lines.iter().flat_map(|line| line.chars()).collect(),
        })
    }

    fn solve(&self) -> u64 {
        let mut visited = (0..(self.width * self.height))
            .map(|_| false)
            .collect::<Vec<_>>();

        let mut result = 0;
        let mut i = 0;
        for y in 0..(self.height) {
            for x in 0..(self.width) {
                if !visited[i] {
                    let (child_area, child_perimeter) = self.visit(Point { x, y }, &mut visited);
                    result += child_area * child_perimeter;
                }
                i += 1;
            }
        }
        result
    }

    fn visit(&self, point: Point, visited: &mut Vec<bool>) -> (u64, u64) {
        let i = point.y * self.width + point.x;
        visited[i] = true;

        let this_symbol = self.data[i];

        let mut area = 1;
        let mut perimeter = 0;

        let possible_neighbors = &[
            if point.x >= 1 {
                Some(Point {
                    x: point.x - 1,
                    y: point.y,
                })
            } else {
                perimeter += 1;
                None
            },
            if point.x + 1 < self.width {
                Some(Point {
                    x: point.x + 1,
                    y: point.y,
                })
            } else {
                perimeter += 1;
                None
            },
            if point.y >= 1 {
                Some(Point {
                    x: point.x,
                    y: point.y - 1,
                })
            } else {
                perimeter += 1;
                None
            },
            if point.y + 1 < self.height {
                Some(Point {
                    x: point.x,
                    y: point.y + 1,
                })
            } else {
                perimeter += 1;
                None
            },
        ];
        for neighbor in possible_neighbors.iter().filter_map(|x| *x) {
            let other_i = neighbor.y * self.width + neighbor.x;
            let other_symbol = self.data[other_i];
            if this_symbol == other_symbol {
                if !visited[other_i] {
                    let (child_area, child_perimeter) = self.visit(neighbor, visited);
                    area += child_area;
                    perimeter += child_perimeter;
                }
            } else {
                perimeter += 1;
            }
        }

        (area, perimeter)
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

    let map = Map::new(
        &file_contents
            .iter()
            .map(|line| line.as_str())
            .collect::<Vec<_>>(),
    )?;
    Ok(map.solve())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample1() {
        assert_eq!(do_it("day12-sample1.txt").unwrap(), 140);
    }

    #[test]
    pub fn test_sample2() {
        assert_eq!(do_it("day12-sample2.txt").unwrap(), 772);
    }

    #[test]
    pub fn test_sample3() {
        assert_eq!(do_it("day12-sample3.txt").unwrap(), 1930);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day12.txt").unwrap(), 1433460);
    }
}
