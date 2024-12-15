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
    BoxLeft,
    BoxRight,
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
        let width = width * 2;
        let mut state = Vec::with_capacity(width * height);
        let mut robot_position = None;
        for y in 0..height {
            let line = map[y].chars().collect::<Vec<_>>();
            for (x, c) in line.iter().enumerate() {
                let cells = match c {
                    'O' => [Cell::BoxLeft, Cell::BoxRight],
                    '.' => [Cell::Empty, Cell::Empty],
                    '#' => [Cell::Wall, Cell::Wall],
                    '@' => {
                        robot_position = Some(Point {
                            x: (x * 2) as i64,
                            y: y as i64,
                        });
                        [Cell::Empty, Cell::Empty]
                    }
                    _ => Err(format!("unparsable map char: {}", c))?,
                };
                state.extend_from_slice(&cells);
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
            let i = (p.y as usize) * self.width + (p.x as usize);
            if self.state[i] == Cell::Wall {
                Err(format!("can't update cell that is a wall at {:?}", p))?;
            }
            self.state[i] = value;
            Ok(())
        } else {
            Err(format!("set out of bounds {:?}", p))?
        }
    }

    /*
    attempts to move the box located at the given point in the given direction
    if there is another box there, it tries to recursively push that box too
    stops when they hit a wall, in which case it returns false
    if the given point is empty after attempting the move (i.e. it was empty or a box that had space to move before the move) then it
    returns true
    if the space is filled after trying the move (i.e. it was a wall or a box that did not have space to move) it returns false
    */
    fn push_box_at(&mut self, p: Point, d: Direction) -> Result<bool> {
        // figure out what this point holds
        // if it's a box, figure out where the left and right points of the box are
        let box_part_1 = self.get(p);
        let (left_pos, right_pos) = match box_part_1 {
            // early exit, already empty
            Cell::Empty => return Ok(true),
            // early exit, can't move walls
            Cell::Wall => return Ok(false),
            Cell::BoxLeft => {
                let left_pos = p;
                let right_pos = p + Point { x: 1, y: 0 };
                (left_pos, right_pos)
            }
            Cell::BoxRight => {
                let left_pos = p + Point { x: -1, y: 0 };
                let right_pos = p;
                (left_pos, right_pos)
            }
        };
        // now we try to recursively move the points adjacent to this box in the direction of travel
        // if all cases we can early exit if the result is false because that means we can't move this one either
        match d {
            // only need to check one location, to the left or right
            Direction::Left => {
                let result = self.push_box_at(left_pos + d.to_vector(), d)?;
                if !result {
                    return Ok(false);
                }
            }
            Direction::Right => {
                let result = self.push_box_at(right_pos + d.to_vector(), d)?;
                if !result {
                    return Ok(false);
                }
            }
            // need to check both locations because there are two cells involved
            Direction::Up | Direction::Down => {
                // make sure we undo if either fails
                // that way we don't move one box out of two and then fail to move the other one
                let backup = self.state.clone();
                let left_result = self.push_box_at(left_pos + d.to_vector(), d)?;
                let right_result = self.push_box_at(right_pos + d.to_vector(), d)?;
                if !left_result || !right_result {
                    self.state = backup;
                    return Ok(false);
                }
            }
        };
        // we have free space to move this box
        self.set(left_pos, Cell::Empty)?;
        self.set(right_pos, Cell::Empty)?;
        self.set(left_pos + d.to_vector(), Cell::BoxLeft)?;
        self.set(right_pos + d.to_vector(), Cell::BoxRight)?;
        Ok(true)
    }

    fn advance(&mut self, d: Direction) -> Result<()> {
        let pos = self.robot_position + d.to_vector();
        if match self.get(pos) {
            Cell::BoxLeft | Cell::BoxRight => self.push_box_at(pos, d)?,
            Cell::Empty => true,
            Cell::Wall => false,
        } {
            self.robot_position = pos;
        }
        Ok(())
    }

    fn count_box_gps(&self) -> u64 {
        let mut result = 0u64;
        let mut i = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.state[i] == Cell::BoxLeft {
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
        assert_eq!(do_it("day15b-sample1.txt").unwrap(), 618);
    }

    #[test]
    pub fn test_sample2() {
        assert_eq!(do_it("day15-sample2.txt").unwrap(), 9021);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day15.txt",).unwrap(), 1538862);
    }
}
