use std::{
    collections::HashSet,
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    ops::{Add, Mul},
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

#[derive(Debug, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul<i32> for Point {
    type Output = Point;

    fn mul(self, rhs: i32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

struct Grid {
    width: u32,
    height: u32,
    grid: Vec<Vec<char>>,
}

impl Grid {
    fn new(file_contents: Vec<String>) -> Result<Self> {
        let height = file_contents.len();
        let width = file_contents
            .iter()
            .map(|s| s.len())
            .collect::<HashSet<_>>();
        if width.len() != 1 {
            Err(format!("lines with differnt widths detected: {:?}", width))?;
        }
        let width = *width.iter().next().unwrap();

        let grid = file_contents
            .iter()
            .map(|line| line.chars().collect::<Vec<_>>())
            .collect::<Vec<_>>();

        Ok(Self {
            width: width as u32,
            height: height as u32,
            grid,
        })
    }

    fn get_at(&self, p: &Point) -> Option<char> {
        if p.x >= 0 && p.y >= 0 && p.x < self.width as i32 && p.y < self.height as i32 {
            Some(self.grid[p.y as usize][p.x as usize])
        } else {
            None
        }
    }

    fn is_word(&self, starting_point: &Point, direction: &Point, word: &str) -> bool {
        let actual_data = word.chars().enumerate().map(|(i, c)| {
            let point = *starting_point + *direction * (i as i32);
            self.get_at(&point)
        });
        let actual_data = actual_data.collect::<Option<Vec<_>>>();
        actual_data == Some(word.chars().collect::<Vec<_>>())
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

    let grid = Grid::new(file_contents)?;

    let directions = [
        Point { x: 1, y: 0 },
        Point { x: -1, y: 0 },
        Point { x: 0, y: 1 },
        Point { x: 0, y: -1 },
        Point { x: 1, y: 1 },
        Point { x: 1, y: -1 },
        Point { x: -1, y: 1 },
        Point { x: -1, y: -1 },
    ];

    let mut count = 0;
    for y in 0..grid.height {
        for x in 0..grid.width {
            for dir in &directions {
                if grid.is_word(
                    &Point {
                        x: x as i32,
                        y: y as i32,
                    },
                    dir,
                    "XMAS",
                ) {
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample1() {
        assert_eq!(do_it("day04-sample1.txt").unwrap(), 4);
    }

    #[test]
    pub fn test_sample2() {
        assert_eq!(do_it("day04-sample2.txt").unwrap(), 18);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day04.txt").unwrap(), 2543);
    }
}
