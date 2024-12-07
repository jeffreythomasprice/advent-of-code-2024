use std::{
    collections::HashSet,
    env,
    fmt::{Debug, Display},
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    ops::Add,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn to_vector(&self) -> Point {
        match self {
            Direction::Up => Point { x: 0, y: -1 },
            Direction::Down => Point { x: 0, y: 1 },
            Direction::Left => Point { x: -1, y: 0 },
            Direction::Right => Point { x: 1, y: 0 },
        }
    }

    fn turn_right(&self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Guard {
    position: Point,
    direction: Direction,
}

#[derive(Clone)]
struct State {
    width: usize,
    height: usize,
    data: Vec<bool>,
    guard: Guard,
    visited: Vec<bool>,
}

impl State {
    fn new(lines: &[String]) -> Result<Self> {
        let lines = lines
            .iter()
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();
        let height = lines.len();
        let widths: HashSet<usize> = HashSet::from_iter(lines.iter().map(|line| line.len()));
        if widths.len() == 1 {
            let width = *widths.iter().next().unwrap();
            let mut data = Vec::with_capacity(width * height);
            let mut guard = None;
            for (y, line) in lines.iter().enumerate() {
                for (x, c) in line.chars().enumerate() {
                    let mut proposed_guard = None;
                    match c {
                        '.' => data.push(false),
                        '#' => data.push(true),
                        '^' => {
                            data.push(false);
                            proposed_guard = Some(Guard {
                                position: Point {
                                    x: x as i32,
                                    y: y as i32,
                                },
                                direction: Direction::Up,
                            })
                        }
                        '>' => {
                            data.push(false);
                            proposed_guard = Some(Guard {
                                position: Point {
                                    x: x as i32,
                                    y: y as i32,
                                },
                                direction: Direction::Right,
                            })
                        }
                        '<' => {
                            data.push(false);
                            proposed_guard = Some(Guard {
                                position: Point {
                                    x: x as i32,
                                    y: y as i32,
                                },
                                direction: Direction::Left,
                            })
                        }
                        'v' => {
                            data.push(false);
                            proposed_guard = Some(Guard {
                                position: Point {
                                    x: x as i32,
                                    y: y as i32,
                                },
                                direction: Direction::Down,
                            })
                        }
                        _ => Err(format!("unhandled char: {c}"))?,
                    };
                    if let Some(proposed_guard) = proposed_guard {
                        match guard {
                            Some(_) => Err("two guard locations found")?,
                            None => guard = Some(proposed_guard),
                        };
                    }
                }
            }
            if let Some(guard) = guard {
                let initial_position = guard.position;
                let mut result = Self {
                    width,
                    height,
                    data,
                    guard,
                    visited: (0..(width * height)).map(|_| false).collect(),
                };
                result.visit(initial_position);
                Ok(result)
            } else {
                Err("no guard")?
            }
        } else {
            Err(format!("expected unique width, got {widths:?}"))?
        }
    }

    fn contains_point(&self, p: Point) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.width as i32 && p.y < self.height as i32
    }

    fn point_is_obstacle(&self, p: Point) -> bool {
        if self.contains_point(p) {
            self.data[(p.y as usize) * self.width + (p.x as usize)]
        } else {
            false
        }
    }

    fn add_obstacle(&mut self, p: Point) {
        if self.contains_point(p) {
            self.data[(p.y as usize) * self.width + (p.x as usize)] = true;
        }
    }

    fn guard_is_still_in_bounds(&self) -> bool {
        self.contains_point(self.guard.position)
    }

    fn visit(&mut self, p: Point) {
        if self.contains_point(p) {
            self.visited[(p.y as usize) * self.width + (p.x as usize)] = true;
        }
    }

    fn advance(&mut self) {
        let next_point = self.guard.position + self.guard.direction.to_vector();
        if self.point_is_obstacle(next_point) {
            self.guard.direction = self.guard.direction.turn_right()
        } else {
            self.guard.position = next_point;
            self.visit(next_point);
        }
    }

    // bool is true if the path is a loop
    fn find_path(&self) -> (bool, Vec<Guard>) {
        let mut state = self.clone();
        let mut path = Vec::new();
        let mut path_set = HashSet::new();
        path.push(state.guard.clone());
        path_set.insert(state.guard.clone());
        while state.guard_is_still_in_bounds() {
            state.advance();
            if path_set.contains(&state.guard) {
                return (true, path);
            }
            path.push(state.guard.clone());
            path_set.insert(state.guard.clone());
        }
        (false, path)
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let p = Point {
                    x: x as i32,
                    y: y as i32,
                };
                if self.guard.position == p {
                    match self.guard.direction {
                        Direction::Up => write!(f, "^")?,
                        Direction::Down => write!(f, "v")?,
                        Direction::Left => write!(f, "<")?,
                        Direction::Right => write!(f, ">")?,
                    };
                } else if self.point_is_obstacle(p) {
                    write!(f, "#")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
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

    let state = State::new(&file_contents)?;

    let (_, path) = state.find_path();

    Ok(HashSet::<Point>::from_iter(
        path.iter()
            .map(|previous_guard| previous_guard.position + previous_guard.direction.to_vector()),
    )
    .iter()
    .filter_map(|obstacle| {
        let mut state = state.clone();
        state.add_obstacle(*obstacle);
        let (is_infinite, _) = state.find_path();
        if is_infinite {
            Some(obstacle)
        } else {
            None
        }
    })
    .count())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day06-sample.txt").unwrap(), 6);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day06.txt").unwrap(), 1972);
    }
}
