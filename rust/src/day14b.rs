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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, PartialEq, Eq)]
struct Robot {
    position: Point,
    velocity: Point,
}

#[derive(Clone, PartialEq, Eq)]
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

    fn count_max_contiguous(&self) -> u64 {
        let grid = self.create_2d_grid();
        let mut visited = (0..self.height)
            .map(|_| (0..self.width).map(|_| false).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        fn visit(
            p: Point,
            width: i64,
            height: i64,
            grid: &Vec<Vec<bool>>,
            visited: &mut Vec<Vec<bool>>,
        ) -> u64 {
            if grid[p.y as usize][p.x as usize] && !visited[p.y as usize][p.x as usize] {
                visited[p.y as usize][p.x as usize] = true;
                let mut result = 1;
                if p.x >= 1 {
                    result += visit(Point { x: p.x - 1, y: p.y }, width, height, grid, visited);
                }
                if (p.x + 1) < width {
                    result += visit(Point { x: p.x + 1, y: p.y }, width, height, grid, visited);
                }
                if p.y >= 1 {
                    result += visit(Point { x: p.x, y: p.y - 1 }, width, height, grid, visited);
                }
                if (p.y + 1) < height {
                    result += visit(Point { x: p.x, y: p.y + 1 }, width, height, grid, visited);
                }
                if p.x >= 1 && p.y >= 1 {
                    result += visit(
                        Point {
                            x: p.x - 1,
                            y: p.y - 1,
                        },
                        width,
                        height,
                        grid,
                        visited,
                    );
                }
                if p.x >= 1 && (p.y + 1) < height {
                    result += visit(
                        Point {
                            x: p.x - 1,
                            y: p.y + 1,
                        },
                        width,
                        height,
                        grid,
                        visited,
                    );
                }
                if (p.x + 1) < width && p.y >= 1 {
                    result += visit(
                        Point {
                            x: p.x + 1,
                            y: p.y - 1,
                        },
                        width,
                        height,
                        grid,
                        visited,
                    );
                }
                if (p.x + 1) < width && (p.y + 1) < height {
                    result += visit(
                        Point {
                            x: p.x + 1,
                            y: p.y + 1,
                        },
                        width,
                        height,
                        grid,
                        visited,
                    );
                }
                result
            } else {
                0
            }
        }
        let mut count = 0;
        for r in self.robots.iter() {
            count = count.max(visit(
                r.position,
                self.width,
                self.height,
                &grid,
                &mut visited,
            ));
        }
        count
    }

    fn display(&self) {
        let grid = self.create_2d_grid();
        for y in 0..(self.height as usize) {
            for x in 0..(self.width as usize) {
                if grid[y][x] {
                    print!("X");
                } else {
                    print!(" ");
                }
            }
            println!("");
        }
    }

    fn create_2d_grid(&self) -> Vec<Vec<bool>> {
        let mut result = (0..self.height)
            .map(|_| (0..self.width).map(|_| false).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        for r in self.robots.iter() {
            result[r.position.y as usize][r.position.x as usize] = true;
        }
        result
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
                Ok(r.captures(&line)
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

    let original = state.clone();

    let mut i = 0;
    struct Solution {
        count: u64,
        state: State,
        i: u64,
    }
    let mut solution: Option<Solution> = None;
    loop {
        state.advance();
        i += 1;
        let count = state.count_max_contiguous();
        solution = Some(if let Some(solution) = solution {
            if count > solution.count {
                Solution {
                    count,
                    state: state.clone(),
                    i,
                }
            } else {
                solution
            }
        } else {
            Solution {
                count,
                state: state.clone(),
                i,
            }
        });

        if state == original {
            break;
        }
    }
    if let Some(solution) = &solution {
        solution.state.display();
        Ok(solution.i)
    } else {
        Err("never found a solution")?
    }
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day14.txt", 101, 103).unwrap(), 7412);
    }
}
