use std::{
    collections::HashSet,
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    ops::{Add, AddAssign, Sub, SubAssign},
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

impl Sub<Point> for Point {
    type Output = Self;

    fn sub(self, rhs: Point) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign<Point> for Point {
    fn sub_assign(&mut self, rhs: Point) {
        *self = *self - rhs;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Box,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn to_vector(&self) -> Point {
        match self {
            Direction::Left => Point { x: -1, y: 0 },
            Direction::Right => Point { x: 1, y: 0 },
            Direction::Up => Point { x: 0, y: -1 },
            Direction::Down => Point { x: 0, y: 1 },
        }
    }
}

struct State {
    width: usize,
    height: usize,
    state: Vec<Cell>,
    robot_position: Point,
}

impl State {
    fn new(map: Vec<String>) -> Result<State> {
        let height = map.len();
        let width: HashSet<usize> = HashSet::from_iter(map.iter().map(|line| line.chars().count()));
        if width.len() != 1 {
            Err(format!("uneven map lines: {:?}", width))?;
        }
        let width = *width.iter().next().unwrap();
        let mut state = Vec::with_capacity(width * height);
        let mut robot_position = None;
        for y in 0..height {
            let line = map[y].chars().collect::<Vec<_>>();
            for x in 0..width {
                let c = line[x];
                let cell = match c {
                    'O' => Cell::Box,
                    '.' => Cell::Empty,
                    '#' => Cell::Wall,
                    '@' => {
                        robot_position = Some(Point {
                            x: x as i64,
                            y: y as i64,
                        });
                        Cell::Empty
                    }
                    _ => Err(format!("unparsable map char: {}", c))?,
                };
                state.push(cell);
            }
        }
        if let Some(robot_position) = robot_position {
            Ok(Self {
                width,
                height,
                state,
                robot_position,
            })
        } else {
            Err("missing robot position")?
        }
    }

    fn get(&self, p: Point) -> Cell {
        if p.x >= 0 && p.y >= 0 && (p.x as usize) < self.width && (p.y as usize) < self.height {
            self.state[(p.y as usize) * self.width + (p.x as usize)]
        } else {
            Cell::Wall
        }
    }

    fn set(&mut self, p: Point, value: Cell) -> Result<()> {
        if p.x >= 0 && p.y >= 0 && (p.x as usize) < self.width && (p.y as usize) < self.height {
            self.state[(p.y as usize) * self.width + (p.x as usize)] = value;
            Ok(())
        } else {
            Err(format!("set out of bounds {:?}", p))?
        }
    }

    fn advance(&mut self, d: Direction) -> Result<()> {
        let mut pos = self.robot_position + d.to_vector();
        while self.get(pos) == Cell::Box {
            pos += d.to_vector();
        }
        match self.get(pos) {
            // move all the boxes bewteen the robot and this empty space into this empty space
            Cell::Empty => {
                while pos != self.robot_position {
                    self.set(pos, self.get(pos - d.to_vector()))?;
                    pos -= d.to_vector();
                }
                self.robot_position += d.to_vector();
            }
            Cell::Box => {
                Err("should be impossible, we walked until we found something other than a box")?
            }
            // ignore this move, we hit a wall
            Cell::Wall => (),
        }
        Ok(())
    }

    fn count_box_gps(&self) -> u64 {
        let mut result = 0u64;
        let mut i = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.state[i] == Cell::Box {
                    result += 100 * (y as u64) + (x as u64)
                }
                i += 1;
            }
        }
        result
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
        let line = line?;
        let line = line.trim();
        Ok(line.to_string())
    })
    // break if we have an error
    .collect::<Result<Vec<_>>>()?;

    let map_regex = Regex::new(r"^[O#\.@]+$")?;
    let instruction_regex = Regex::new(r"^[><^v]+$")?;
    let mut map = Vec::new();
    let mut instructions = Vec::new();
    for line in file_contents {
        if line.is_empty() {
            continue;
        }
        if map_regex.is_match(&line) {
            if !instructions.is_empty() {
                Err("found map line in the instructions section?")?
            }
            map.push(line);
        } else if instruction_regex.is_match(&line) {
            instructions.push(line);
        } else {
            Err(format!("unparsable line: {}", line))?
        }
    }

    let mut state = State::new(map)?;

    for c in instructions.join("").chars() {
        let d = match c {
            '<' => Direction::Left,
            '>' => Direction::Right,
            '^' => Direction::Up,
            'v' => Direction::Down,
            _ => Err(format!("unparsable direction: {}", c))?,
        };
        state.advance(d)?;
    }

    Ok(state.count_box_gps())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample1() {
        assert_eq!(do_it("day15-sample1.txt").unwrap(), 2028);
    }

    #[test]
    pub fn test_sample2() {
        assert_eq!(do_it("day15-sample2.txt").unwrap(), 10092);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day15.txt",).unwrap(), 1517819);
    }
}
