use std::{
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    ops::{Add, AddAssign},
    path::Path,
    str::Utf8Error,
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

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self(format!("core::str::error::Utf8Error({value:?})"))
    }
}

#[derive(Debug, Clone, Copy)]
struct Point {
    x: i64,
    y: i64,
}

impl Add<Point> for Point {
    type Output = Self;

    fn add(self, rhs: Point) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign<Point> for Point {
    fn add_assign(&mut self, rhs: Point) {
        *self = *self + rhs;
    }
}

struct Robot {
    position: Point,
    velocity: Point,
}

struct State {
    width: i64,
    height: i64,
    robots: Vec<Robot>,
}

impl State {
    fn advance(&mut self) {
        for r in self.robots.iter_mut() {
            r.position += r.velocity;
            r.position.x %= self.width;
            r.position.y %= self.height;
            if r.position.x < 0 {
                r.position.x += self.width;
            }
            if r.position.y < 0 {
                r.position.y += self.height;
            }
        }
    }

    fn count(&self) -> u64 {
        let center_x = self.width / 2;
        let center_y = self.height / 2;
        let mut quad_1 = 0;
        let mut quad_2 = 0;
        let mut quad_3 = 0;
        let mut quad_4 = 0;
        for r in self.robots.iter() {
            if r.position.x < center_x && r.position.y < center_y {
                quad_1 += 1;
            } else if r.position.x > center_x && r.position.y < center_y {
                quad_2 += 1;
            } else if r.position.x < center_x && r.position.y > center_y {
                quad_3 += 1;
            } else if r.position.x > center_x && r.position.y > center_y {
                quad_4 += 1;
            }
        }
        quad_1 * quad_2 * quad_3 * quad_4
    }
}

#[allow(dead_code)]
fn do_it(path: &str, width: usize, height: usize) -> Result<u64> {
    let file_contents = BufReader::new(File::open(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("puzzle-inputs")
            .join(path),
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

    let r = Regex::new(r"^p=(-?[0-9]+),(-?[0-9]+) v=(-?[0-9]+),(-?[0-9]+)$")?;
    let mut state = State {
        width: width as i64,
        height: height as i64,
        robots: file_contents
            .iter()
            .map(|line| {
                Ok(r.captures(line)
                    .ok_or(format!("failed to match line: {}", line))?)
            })
            .collect::<Result<Vec<_>>>()?
            .iter()
            .map(|line| {
                let (_, [px, py, dx, dy]) = line.extract();
                Ok(Robot {
                    position: Point {
                        x: px.parse()?,
                        y: py.parse()?,
                    },
                    velocity: Point {
                        x: dx.parse()?,
                        y: dy.parse()?,
                    },
                })
            })
            .collect::<Result<Vec<_>>>()?,
    };

    for _ in 0..100 {
        state.advance();
    }

    Ok(state.count())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day14-sample.txt", 11, 7).unwrap(), 12);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day14.txt", 101, 103).unwrap(), 217328832);
    }
}
