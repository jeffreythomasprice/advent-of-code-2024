use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    ops::{Add, AddAssign, Sub, SubAssign},
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

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

struct Grid {
    width: usize,
    height: usize,
    towers: HashMap<char, Vec<Point>>,
}

impl Grid {
    fn new(lines: &[&str]) -> Result<Self> {
        let data = lines
            .iter()
            .filter_map(|line| {
                let line = line
                    .trim()
                    .chars()
                    .map(|c| match c {
                        '.' => None,
                        _ => Some(c),
                    })
                    .collect::<Vec<_>>();
                if line.is_empty() {
                    None
                } else {
                    Some(line)
                }
            })
            .collect::<Vec<_>>();

        let height = data.len();
        let widths = HashSet::<usize>::from_iter(data.iter().map(|line| line.len()));
        if widths.len() != 1 {
            Err(format!("uneven row lengths: {widths:?}"))?;
        }
        let width = *widths.iter().next().unwrap();

        let mut towers = HashMap::new();
        for (y, row) in data.iter().enumerate() {
            for (x, value) in row.iter().enumerate() {
                if let Some(value) = value {
                    let tower = Point {
                        x: x as i32,
                        y: y as i32,
                    };
                    towers.entry(*value).or_insert(Vec::new()).push(tower);
                }
            }
        }

        Ok(Self {
            width,
            height,
            towers,
        })
    }

    fn iterate_tower_pairs<F>(&self, mut f: F)
    where
        F: FnMut(char, Point, Point),
    {
        for (kind, towers) in self.towers.iter() {
            let mut a = towers.iter();
            while let Some(x) = a.next() {
                let b = a.clone();
                for y in b {
                    f(*kind, *x, *y);
                }
            }
        }
    }

    fn contains(&self, p: Point) -> bool {
        p.x >= 0 && p.y >= 0 && (p.x as usize) < self.width && (p.y as usize) < self.height
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

    let grid = Grid::new(
        &file_contents
            .iter()
            .map(|line| line.as_str())
            .collect::<Vec<_>>(),
    )?;

    let mut results = HashSet::new();
    grid.iterate_tower_pairs(|_, a, b| {
        results.insert(a);
        results.insert(b);
        let delta = b - a;
        let mut x = a - delta;
        let mut y = b + delta;
        while grid.contains(x) {
            results.insert(x);
            x -= delta;
        }
        while grid.contains(y) {
            results.insert(y);
            y += delta;
        }
    });
    Ok(results.len())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day08-sample.txt").unwrap(), 34);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day08.txt").unwrap(), 813);
    }
}
