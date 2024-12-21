use std::{
    cmp::Ordering,
    collections::{ HashSet},
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    ops::{Add, AddAssign, Sub, SubAssign},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Wall,
}

#[derive(Debug, Clone)]
enum PathElement {
    Goal,
    Element { distance: u64 },
}

#[derive(Clone)]
struct Grid {
    width: usize,
    height: usize,
    data: Vec<Cell>,
    goal: Point,
}

impl Grid {
    fn new(lines: &[String]) -> Result<Self> {
        let height = lines.len();
        let widths = HashSet::<usize>::from_iter(lines.iter().map(|line| line.len()));
        if widths.len() != 1 {
            Err(format!(
                "expected all lines to be the same length, got {:?}",
                widths
            ))?;
        }
        let width = *widths.iter().next().unwrap();

        let mut start = None;
        let mut end = None;
        let mut data = Vec::with_capacity(width * height);
        for (y, line) in lines.iter().enumerate() {
            for (x, c) in line.chars().enumerate() {
                data.push(match c {
                    '.' => Cell::Empty,
                    '#' => Cell::Wall,
                    'S' => {
                        start = Some(Point {
                            x: x as i64,
                            y: y as i64,
                        });
                        Cell::Empty
                    }
                    'E' => {
                        end = Some(Point {
                            x: x as i64,
                            y: y as i64,
                        });
                        Cell::Empty
                    }
                    _ => Err(format!("illegal character: {}", c))?,
                });
            }
        }

        match (start, end) {
            (Some(_), Some(end)) => Ok(Self {
                width,
                height,
                data,
                goal: end,
            }),
            _ => Err("failed to find start and/or end position")?,
        }
    }

    fn count_shortcuts(&self) -> Result<Vec<u64>> {
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
                if self.data[p_i] == Cell::Empty {
                    queue.push(p);
                    queue_contains[p_i] = true;
                    if p == self.goal {
                        graph[p_i] = Some(PathElement::Goal);
                    }
                }
            }
        }

        while !queue.is_empty() {
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
                        }
                    }
                }
            }
        }

        /*
        now we have a graph that should contain for every empty cell:
        - the distance to the goal if we take no shortcuts
        - the next point towards the goal

        now we can find all possible shortcuts we could take and compare the distance if we take them
        */

        let mut results = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let before_shortcut = Point {
                    x: x as i64,
                    y: y as i64,
                };
                let before_shortcut_i = self.index(before_shortcut)?;
                // find all the walls around this point
                for d in [
                    Direction::Left,
                    Direction::Right,
                    Direction::Up,
                    Direction::Down,
                ] {
                    let shortcut_1 = before_shortcut + d.to_vector();
                    // make sure to ignore out of bounds points
                    if let Ok(shortcut_1_i) = self.index(shortcut_1) {
                        if self.data[shortcut_1_i] == Cell::Wall {
                            // now find all the empty spots next to that wall that aren't the original point
                            for d in [
                                Direction::Left,
                                Direction::Right,
                                Direction::Up,
                                Direction::Down,
                            ] {
                                let shortcut_2 = shortcut_1 + d.to_vector();
                                if let Ok(shortcut_2_i) = self.index(shortcut_2) {
                                    if shortcut_2 != before_shortcut
                                        && self.data[shortcut_2_i] == Cell::Empty
                                    {
                                        // we're now sure that before_shortcut -> shortcut_1 -> shortcut_2 is a shortcut
                                        let distance_without_shortcut = self
                                            .effective_distance(&graph[before_shortcut_i])
                                            .unwrap_or(0);
                                        let distance_with_shortcut = self
                                            .effective_distance(&graph[shortcut_2_i])
                                            .unwrap_or(0) 
                                            // plus the distance it took to actually take the shortcut
                                            + 2;
                                        // if we have saved time doing this we remember how much time we saved
                                        if distance_with_shortcut < distance_without_shortcut {
                                            results.push(distance_without_shortcut  - distance_with_shortcut);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(results)
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
            &PathElement::Element { distance  } => distance,
            PathElement::Goal => 0,
        })
    }
}

#[allow(dead_code)]
fn do_it(path: &str, at_least_time_saved: u64) -> Result<usize> {
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

    let grid = Grid::new(&file_contents)?;

   let time_saved =  grid.count_shortcuts()?;
   Ok(time_saved.into_iter().filter(|x| *x >= at_least_time_saved).count())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day20-sample.txt", 20).unwrap(), 5);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day20.txt", 100).unwrap(), 1375);
    }
}
