use std::{
    collections::{HashMap, HashSet},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
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
                    let mut sides = HashMap::new();
                    let child_area = self.visit(Point { x, y }, &mut visited, &mut sides);
                    let mut perimeter = 0;
                    for (_, sides) in sides.iter_mut() {
                        for (_, list) in sides.iter_mut() {
                            list.sort();
                            let mut previous = None;
                            if !list.is_empty() {
                                perimeter += 1;
                                for current in list {
                                    if let Some(previous) = previous {
                                        if *current - previous > 1 {
                                            perimeter += 1;
                                        }
                                    }
                                    previous = Some(*current);
                                }
                            }
                        }
                    }
                    result += child_area * perimeter;
                }
                i += 1;
            }
        }
        result
    }

    fn visit(
        &self,
        point: Point,
        visited: &mut Vec<bool>,
        sides: &mut HashMap<Direction, HashMap<u64, Vec<u64>>>,
    ) -> u64 {
        let i = point.y * self.width + point.x;
        visited[i] = true;

        let this_symbol = self.data[i];

        let mut area = 1;

        let possible_neighbors = &[
            if point.x >= 1 {
                Some((
                    Point {
                        x: point.x - 1,
                        y: point.y,
                    },
                    Direction::Left,
                    point.x as u64,
                    point.y as u64,
                ))
            } else {
                sides
                    .entry(Direction::Left)
                    .or_insert(HashMap::new())
                    .entry(point.x as u64)
                    .or_insert(Vec::new())
                    .push(point.y as u64);
                None
            },
            if point.x + 1 < self.width {
                Some((
                    Point {
                        x: point.x + 1,
                        y: point.y,
                    },
                    Direction::Right,
                    (point.x + 1) as u64,
                    point.y as u64,
                ))
            } else {
                sides
                    .entry(Direction::Right)
                    .or_insert(HashMap::new())
                    .entry((point.x + 1) as u64)
                    .or_insert(Vec::new())
                    .push(point.y as u64);
                None
            },
            if point.y >= 1 {
                Some((
                    Point {
                        x: point.x,
                        y: point.y - 1,
                    },
                    Direction::Up,
                    point.y as u64,
                    point.x as u64,
                ))
            } else {
                sides
                    .entry(Direction::Up)
                    .or_insert(HashMap::new())
                    .entry(point.y as u64)
                    .or_insert(Vec::new())
                    .push(point.x as u64);
                None
            },
            if point.y + 1 < self.height {
                Some((
                    Point {
                        x: point.x,
                        y: point.y + 1,
                    },
                    Direction::Down,
                    (point.y + 1) as u64,
                    point.x as u64,
                ))
            } else {
                sides
                    .entry(Direction::Down)
                    .or_insert(HashMap::new())
                    .entry((point.y + 1) as u64)
                    .or_insert(Vec::new())
                    .push(point.x as u64);
                None
            },
        ];
        for i in possible_neighbors.iter() {
            if let Some((neighbor, direction, fence_index, fence_location)) = i {
                let other_i = neighbor.y * self.width + neighbor.x;
                let other_symbol = self.data[other_i];
                if this_symbol == other_symbol {
                    if !visited[other_i] {
                        let child_area = self.visit(*neighbor, visited, sides);
                        area += child_area;
                    }
                } else {
                    sides
                        .entry(*direction)
                        .or_insert(HashMap::new())
                        .entry(*fence_index)
                        .or_insert(Vec::new())
                        .push(*fence_location);
                }
            }
        }

        area
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
        assert_eq!(do_it("day12-sample1.txt").unwrap(), 80);
    }

    #[test]
    pub fn test_sample2() {
        assert_eq!(do_it("day12b-sample2.txt").unwrap(), 236);
    }

    #[test]
    pub fn test_sample3() {
        assert_eq!(do_it("day12b-sample3.txt").unwrap(), 368);
    }

    #[test]
    pub fn test_sample4() {
        assert_eq!(do_it("day12-sample3.txt").unwrap(), 1206);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day12.txt").unwrap(), 855082);
    }
}
