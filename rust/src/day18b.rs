use std::{
    cmp::Ordering,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

struct Grid {
    width: usize,
    height: usize,
    data: Vec<bool>,
}

#[derive(Debug, Clone)]
enum PathElement {
    Start,
    Element { distance: u64 },
}

impl Grid {
    fn new(width: usize, height: usize, lines: &[String]) -> Result<Grid> {
        let mut result = Self {
            width,
            height,
            data: (0..(width * height)).map(|_| false).collect::<Vec<_>>(),
        };
        let r = Regex::new(r"^([0-9]+),([0-9]+)$")?;
        for line in lines {
            let (_, [x, y]) = r
                .captures(line)
                .ok_or(format!("regex failed: {line}"))?
                .extract();
            let x: usize = x.parse()?;
            let y: usize = y.parse()?;
            result.data[y * width + x] = true;
        }
        Ok(result)
    }

    fn shorted_path(&self, start: Point, goal: Point) -> Result<u64> {
        /*
        dijkstra
        vertices are position + direction
        edges are cost to make that change, 1 for moving forward and 1000 for turning left or right
        terminate when you are at the goal
        */

        let mut queue = Vec::new();
        let mut queue_contains = (0..(self.width * self.height))
            .map(|_| false)
            .collect::<Vec<_>>();
        let mut graph = (0..(self.width * self.height))
            .map(|_| None)
            .collect::<Vec<_>>();
        for x in 0..self.width {
            for y in 0..self.height {
                let p = Point {
                    x: x as i64,
                    y: y as i64,
                };
                let p_i = self.index(p)?;
                if !self.data[p_i] {
                    queue.push(p);
                    queue_contains[p_i] = true;
                    if p == start {
                        graph[p_i] = Some(PathElement::Start);
                    }
                }
            }
        }

        let mut goal_node = None;
        while !queue.is_empty() && goal_node.is_none() {
            // find the next element
            // sort in decreasing distance
            let (next_i, next) = queue
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| {
                    let a_value = &graph[self.index(**a).unwrap()];
                    let b_value = &graph[self.index(**b).unwrap()];

                    let a_distance = self.effective_distance(a_value);
                    let b_distance = self.effective_distance(b_value);

                    match (a_distance, b_distance) {
                        // both cells have no previous path element
                        (None, None) => Ordering::Equal,
                        // any distance is less than no previous
                        // but we sort backwards so the end of the vector is the next element, so real values go last
                        (None, Some(_)) => Ordering::Less,
                        (Some(_), None) => Ordering::Greater,
                        // real values, again sort backwards so the small number is at the end of the list
                        (Some(a), Some(b)) => b.cmp(&a),
                    }
                })
                .ok_or("failed to pop from queue, but it should have at least one thing")?;
            let next = *next;
            queue.swap_remove(next_i);
            let next_i = self.index(next)?;
            queue_contains[next_i] = false;

            let current_distance_to_next =
                self.effective_distance(&graph[next_i]).ok_or("can't possibly have got to a node in the queue without there being some distance to it")?;

            for d in [
                Direction::Left,
                Direction::Right,
                Direction::Up,
                Direction::Down,
            ] {
                let neighbor = next + d.to_vector();
                if let Ok(neighbor_i) = self.index(neighbor) {
                    if queue_contains[neighbor_i] {
                        let current_distance_to_neighbor =
                            self.effective_distance(&graph[neighbor_i]);

                        let proposed_distance_to_neighbor = current_distance_to_next + 1;

                        let replace = if let Some(current_distance_to_neighbor) =
                            current_distance_to_neighbor
                        {
                            if proposed_distance_to_neighbor < current_distance_to_neighbor {
                                // new distance is shorter
                                true
                            } else {
                                // existing distance is shorter
                                false
                            }
                        } else {
                            // no existing distance to neighbor, this must be the better path
                            true
                        };
                        if replace {
                            graph[neighbor_i] = Some(PathElement::Element {
                                distance: proposed_distance_to_neighbor,
                            });

                            if neighbor == goal {
                                goal_node = Some(neighbor);
                            }
                        }
                    }
                }
            }
        }

        if let Some(goal_node) = goal_node {
            match &graph[self.index(goal_node)?] {
                Some(PathElement::Element { distance }) => Ok(*distance),
                _ => Err("thought we found the goal node, but no distance found for it")?,
            }
        } else {
            Err("exited, but didn't find a path to the goal")?
        }
    }

    fn index(&self, p: Point) -> Result<usize> {
        if p.x >= 0 && p.y >= 0 && (p.x as usize) < self.width && (p.y as usize) < self.height {
            Ok(p.y as usize * self.width + p.x as usize)
        } else {
            Err(format!("out of bounds: {:?}", p))?
        }
    }

    fn effective_distance(&self, x: &Option<PathElement>) -> Option<u64> {
        // effective distance is 0 for Some(Start), and infinity for None
        x.as_ref().map(|x| match x {
            &PathElement::Element { distance } => distance,
            PathElement::Start => 0,
        })
    }
}

#[allow(dead_code)]
fn do_it(path: &str, width: usize, height: usize) -> Result<String> {
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

    // ignore empty lines
    let file_contents = file_contents
        .into_iter()
        .filter_map(|line| if line.is_empty() { None } else { Some(line) })
        .collect::<Vec<_>>();

    /*
    binary search a split point in the list
    looking for the first point at which the maze becomes unsolvable
    */
    let mut count = file_contents.len() / 2;
    let mut floor = 0;
    let mut ceiling = file_contents.len() - 1;
    let mut checked = (0..file_contents.len()).map(|_| None).collect::<Vec<_>>();
    loop {
        let grid = Grid::new(width, height, &file_contents[0..(count + 1)])?;

        let result = grid.shorted_path(
            Point { x: 0, y: 0 },
            Point {
                x: (width as i64) - 1,
                y: (height as i64) - 1,
            },
        );
        checked[count] = Some(result.is_ok());

        // if this one fails and the previous one succeeds then we're done
        if checked[count] == Some(false) && count >= 1 && checked[count - 1] == Some(true) {
            return Ok(file_contents[count].clone());
        }

        // same thing but in reverse, if the next one would fail us we're done
        if checked[count] == Some(true)
            && count + 1 < file_contents.len()
            && checked[count + 1] == Some(false)
        {
            return Ok(file_contents[count].clone());
        }

        // if we're successful we need to forward until we fail
        if checked[count] == Some(true) {
            floor = count;
            count = (count + 1)
                .max((count + ceiling) / 2)
                .min(file_contents.len() - 1);
            continue;
        }

        // if we failed we need to go backwards
        if checked[count] == Some(false) {
            ceiling = count;
            count = (count - 1).min((count + floor) / 2);
            continue;
        }

        Err("unstable?")?;
    }
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day18-sample.txt", 7, 7).unwrap(), "6,1");
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day18.txt", 71, 71).unwrap(), "43,12");
    }
}
