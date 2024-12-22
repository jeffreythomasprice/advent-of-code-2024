use std::{
    cmp::Ordering,
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
enum Cell {
    Empty,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn index(&self) -> usize {
        match self {
            Direction::Left => 0,
            Direction::Right => 1,
            Direction::Up => 2,
            Direction::Down => 3,
        }
    }

    fn to_vector(&self) -> Point {
        match self {
            Direction::Left => Point { x: -1, y: 0 },
            Direction::Right => Point { x: 1, y: 0 },
            Direction::Up => Point { x: 0, y: -1 },
            Direction::Down => Point { x: 0, y: 1 },
        }
    }

    fn left(&self) -> Direction {
        match self {
            Direction::Left => Direction::Down,
            Direction::Right => Direction::Up,
            Direction::Up => Direction::Left,
            Direction::Down => Direction::Right,
        }
    }

    fn right(&self) -> Direction {
        match self {
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
        }
    }
}

struct State {
    width: usize,
    height: usize,
    state: Vec<Cell>,
    start: Point,
    goal: Point,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GraphNode {
    position: Point,
    direction: Direction,
}

impl GraphNode {
    fn index(&self, width: usize, height: usize) -> usize {
        self.direction.index() * width * height
            + (self.position.y as usize) * width
            + self.position.x as usize
    }
}

#[derive(Debug, Clone)]
enum PathElement {
    Start,
    Element {
        distance: u64,
        previous: Vec<GraphNode>,
    },
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
        let mut start = None;
        let mut goal = None;
        for y in 0..height {
            let line = map[y].chars().collect::<Vec<_>>();
            for (x, c) in line.iter().enumerate() {
                state.push(match c {
                    '#' => Cell::Wall,
                    '.' => Cell::Empty,
                    'S' => {
                        start = Some(Point {
                            x: x as i64,
                            y: y as i64,
                        });
                        Cell::Empty
                    }
                    'E' => {
                        goal = Some(Point {
                            x: x as i64,
                            y: y as i64,
                        });
                        Cell::Empty
                    }
                    _ => Err(format!("unparsable map char: {}", c))?,
                });
            }
        }
        match (start, goal) {
            (Some(start), Some(goal)) => Ok(Self {
                width,
                height,
                state,
                start,
                goal,
            }),
            _ => Err("missing start and/or goal position")?,
        }
    }

    fn get(&self, p: Point) -> Cell {
        if p.x >= 0 && p.y >= 0 && (p.x as usize) < self.width && (p.y as usize) < self.height {
            self.state[(p.y as usize) * self.width + (p.x as usize)]
        } else {
            Cell::Wall
        }
    }

    fn count_all_tiles_on_shortest_path(&self) -> Result<u64> {
        /*
        dijkstra
        vertices are position + direction
        edges are cost to make that change, 1 for moving forward and 1000 for turning left or right
        terminate when you are at the goal
        */

        let mut queue = Vec::new();
        let mut queue_contains = (0..(self.width * self.height * 4))
            .map(|_| false)
            .collect::<Vec<_>>();
        let mut graph = (0..(self.width * self.height * 4))
            .map(|_| None)
            .collect::<Vec<_>>();
        for x in 0..self.width {
            for y in 0..self.height {
                let p = Point {
                    x: x as i64,
                    y: y as i64,
                };
                if self.get(p) == Cell::Empty {
                    for d in [
                        Direction::Left,
                        Direction::Right,
                        Direction::Up,
                        Direction::Down,
                    ] {
                        let node = GraphNode {
                            position: p,
                            direction: d,
                        };
                        queue.push(node);
                        queue_contains[self.graph_node_index(&node)] = true;
                        if d == Direction::Right && p == self.start {
                            graph[self.graph_node_index(&node)] = Some(PathElement::Start);
                        }
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
                    let a_value = &graph[a.index(self.width, self.height)];
                    let b_value = &graph[b.index(self.width, self.height)];

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
            queue_contains[self.graph_node_index(&next)] = false;

            let current_distance_to_next =
                self.effective_distance(&graph[self.graph_node_index(&next)]).ok_or("can't possibly have got to a node in the queue without there being some distance to it")?;

            self.neighbors(&next, |neighbor, delta| {
                if queue_contains[self.graph_node_index(&neighbor)] {
                    let current_distance_to_neighbor =
                        self.effective_distance(&graph[self.graph_node_index(&neighbor)]);

                    let proposed_distance_to_neighbor = current_distance_to_next + delta;

                    if let Some(current_distance_to_neighbor) = current_distance_to_neighbor {
                        if proposed_distance_to_neighbor < current_distance_to_neighbor {
                            // new distance is shorter, so this must be a better path
                            graph[self.graph_node_index(&neighbor)] = Some(PathElement::Element {
                                distance: proposed_distance_to_neighbor,
                                previous: vec![next],
                            });
                        } else if proposed_distance_to_neighbor == current_distance_to_neighbor {
                            // this is another route we could take to get here
                            match &mut graph[self.graph_node_index(&neighbor)] {
                                Some(PathElement::Element { distance: _, previous }) => {
                                    previous.push(next);
                                },
                                Some(PathElement::Start) => Err("found start element when expected a list of at least one previous element")?,
                                None => Err("found no path element when expected a list of at least one previous element")?,
                            };
                        } else {
                            // existing distance is shorter, nothing to do
                        }
                    } else {
                        // no existing distance to neighbor, this must be the better path
                        graph[self.graph_node_index(&neighbor)] = Some(PathElement::Element {
                            distance: proposed_distance_to_neighbor,
                            previous: vec![next],
                        });
                    };
                }
                Ok(())
            })?;
        }

        // start with all possible ways we could have reached the goal node
        let possible_goal_nodes = [
            GraphNode {
                position: self.goal,
                direction: Direction::Left,
            },
            GraphNode {
                position: self.goal,
                direction: Direction::Right,
            },
            GraphNode {
                position: self.goal,
                direction: Direction::Up,
            },
            GraphNode {
                position: self.goal,
                direction: Direction::Down,
            },
        ]
        .iter()
        .map(|node| {
            let distance = self
                .effective_distance(&graph[self.graph_node_index(node)])
                .unwrap_or(0);
            (*node, distance)
        })
        .collect::<Vec<_>>();
        let min_distance = possible_goal_nodes
            .iter()
            .map(|(_, d)| d)
            .min()
            .ok_or("expected a way to reach to goal but found none")?;
        let mut queue = Vec::new();
        for possible_goal_nodes in
            possible_goal_nodes
                .iter()
                .filter_map(|(node, d)| if d == min_distance { Some(node) } else { None })
        {
            queue.push(*possible_goal_nodes);
        }
        // collect all unique points along the way back to the start
        let mut results = HashSet::new();
        // we can definitely reach the start
        results.insert(self.start);
        while let Some(node) = queue.pop() {
            // we can reach this node
            results.insert(node.position);
            match &graph[self.graph_node_index(&node)] {
                // this was some reachable node, continue on all paths that led us here
                Some(PathElement::Element {
                    distance: _,
                    previous,
                }) => {
                    queue.extend_from_slice(previous);
                }
                // we're at the start, nothing to add
                Some(PathElement::Start) => (),
                // this wasn't a reachable node, ignore it
                None => (),
            };
        }
        Ok(results.len() as u64)
    }

    fn graph_node_index(&self, x: &GraphNode) -> usize {
        x.index(self.width, self.height)
    }

    fn effective_distance(&self, x: &Option<PathElement>) -> Option<u64> {
        // effective distance is 0 for Some(Start), and infinity for None
        x.as_ref().map(|x| match x {
            &PathElement::Element {
                distance,
                previous: _,
            } => distance,
            PathElement::Start => 0,
        })
    }

    fn neighbors<F>(&self, x: &GraphNode, mut f: F) -> Result<()>
    where
        F: FnMut(GraphNode, u64) -> Result<()>,
    {
        let forward = x.position + x.direction.to_vector();
        if self.get(forward) == Cell::Empty {
            f(
                GraphNode {
                    position: forward,
                    direction: x.direction,
                },
                1,
            )?;
        }
        f(
            GraphNode {
                position: x.position,
                direction: x.direction.left(),
            },
            1000,
        )?;
        f(
            GraphNode {
                position: x.position,
                direction: x.direction.right(),
            },
            1000,
        )?;
        Ok(())
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

    let state = State::new(file_contents)?;
    state.count_all_tiles_on_shortest_path()
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample1() {
        assert_eq!(do_it("day16-sample1.txt").unwrap(), 45);
    }

    #[test]
    pub fn test_sample2() {
        assert_eq!(do_it("day16-sample2.txt").unwrap(), 64);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day16.txt",).unwrap(), 476);
    }
}
