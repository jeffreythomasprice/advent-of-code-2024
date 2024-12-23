use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
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
    x: i8,
    y: i8,
}

impl Add for Point {
    type Output = Self;

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
            Direction::Up => Point { x: 0, y: -1 },
            Direction::Down => Point { x: 0, y: 1 },
            Direction::Left => Point { x: -1, y: 0 },
            Direction::Right => Point { x: 1, y: 0 },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NumericSymbol {
    Accept,
    Digit(char),
}

/*
+---+---+---+
| 7 | 8 | 9 |
+---+---+---+
| 4 | 5 | 6 |
+---+---+---+
| 1 | 2 | 3 |
+---+---+---+
    | 0 | A |
    +---+---+
*/
struct NumericKeypad {
    current: Point,
}

impl NumericKeypad {
    fn new() -> Self {
        Self {
            current: Point { x: 2, y: 3 },
        }
    }

    fn get(&self) -> Result<NumericSymbol> {
        Ok(match self.current {
            Point { x: 0, y: 0 } => NumericSymbol::Digit('7'),
            Point { x: 1, y: 0 } => NumericSymbol::Digit('8'),
            Point { x: 2, y: 0 } => NumericSymbol::Digit('9'),
            Point { x: 0, y: 1 } => NumericSymbol::Digit('4'),
            Point { x: 1, y: 1 } => NumericSymbol::Digit('5'),
            Point { x: 2, y: 1 } => NumericSymbol::Digit('6'),
            Point { x: 0, y: 2 } => NumericSymbol::Digit('1'),
            Point { x: 1, y: 2 } => NumericSymbol::Digit('2'),
            Point { x: 2, y: 2 } => NumericSymbol::Digit('3'),
            Point { x: 1, y: 3 } => NumericSymbol::Digit('0'),
            Point { x: 2, y: 3 } => NumericSymbol::Accept,
            _ => Err(format!("illegal position: {:?}", self.current))?,
        })
    }

    fn get_coordinates_of_symbol(&self, symbol: NumericSymbol) -> Result<Point> {
        Ok(match symbol {
            NumericSymbol::Accept => Point { x: 2, y: 3 },
            NumericSymbol::Digit('0') => Point { x: 1, y: 3 },
            NumericSymbol::Digit('1') => Point { x: 0, y: 2 },
            NumericSymbol::Digit('2') => Point { x: 1, y: 2 },
            NumericSymbol::Digit('3') => Point { x: 2, y: 2 },
            NumericSymbol::Digit('4') => Point { x: 0, y: 1 },
            NumericSymbol::Digit('5') => Point { x: 1, y: 1 },
            NumericSymbol::Digit('6') => Point { x: 2, y: 1 },
            NumericSymbol::Digit('7') => Point { x: 0, y: 0 },
            NumericSymbol::Digit('8') => Point { x: 1, y: 0 },
            NumericSymbol::Digit('9') => Point { x: 2, y: 0 },
            _ => Err(format!("illegal symbol: {:?}", symbol))?,
        })
    }
    fn update(&mut self, d: Direction) -> Result<()> {
        let next = self.current + d.to_vector();
        if next.x < 0 || next.y < 0 || next.x > 2 || next.y > 3 || (next.x == 0 && next.y == 3) {
            Err(format!("illegal position: {:?}", next))?
        } else {
            self.current = next;
            Ok(())
        }
    }

    fn update_to<F>(&mut self, symbol: NumericSymbol, mut f: F) -> Result<()>
    where
        F: FnMut(Direction) -> Result<()>,
    {
        let target = self.get_coordinates_of_symbol(symbol)?;

        if self.current.y == 3 {
            for _ in target.y..self.current.y {
                self.update(Direction::Up)?;
                f(Direction::Up)?;
            }
        }

        if target.x < self.current.x {
            for _ in target.x..self.current.x {
                self.update(Direction::Left)?;
                f(Direction::Left)?;
            }
        } else if target.x > self.current.x {
            for _ in self.current.x..target.x {
                self.update(Direction::Right)?;
                f(Direction::Right)?;
            }
        }

        if target.y < self.current.y {
            for _ in target.y..self.current.y {
                self.update(Direction::Up)?;
                f(Direction::Up)?;
            }
        } else if target.y > self.current.y {
            for _ in self.current.y..target.y {
                self.update(Direction::Down)?;
                f(Direction::Down)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DirectionalSymbol {
    Accept,
    Direction(Direction),
}

/*
    +---+---+
    | ^ | A |
+---+---+---+
| < | v | > |
+---+---+---+
*/
struct DirectionalKeypad {
    current: Point,
}

impl DirectionalKeypad {
    fn new() -> Self {
        Self {
            current: Point { x: 2, y: 0 },
        }
    }

    fn get(&self) -> Result<DirectionalSymbol> {
        Ok(match self.current {
            Point { x: 1, y: 0 } => DirectionalSymbol::Direction(Direction::Up),
            Point { x: 2, y: 0 } => DirectionalSymbol::Accept,
            Point { x: 0, y: 1 } => DirectionalSymbol::Direction(Direction::Left),
            Point { x: 1, y: 1 } => DirectionalSymbol::Direction(Direction::Down),
            Point { x: 2, y: 1 } => DirectionalSymbol::Direction(Direction::Right),
            _ => Err(format!("illegal position: {:?}", self.current))?,
        })
    }

    fn get_coordinates_of_symbol(&self, symbol: DirectionalSymbol) -> Point {
        match symbol {
            DirectionalSymbol::Accept => Point { x: 2, y: 0 },
            DirectionalSymbol::Direction(Direction::Left) => Point { x: 0, y: 1 },
            DirectionalSymbol::Direction(Direction::Right) => Point { x: 2, y: 1 },
            DirectionalSymbol::Direction(Direction::Up) => Point { x: 1, y: 0 },
            DirectionalSymbol::Direction(Direction::Down) => Point { x: 1, y: 1 },
        }
    }

    fn update(&mut self, d: Direction) -> Result<()> {
        let next = self.current + d.to_vector();
        if next.x < 0 || next.y < 0 || next.x > 2 || next.y > 1 || (next.x == 0 && next.y == 0) {
            Err(format!("illegal position: {:?}", next))?
        } else {
            self.current = next;
            Ok(())
        }
    }

    fn update_to<F>(&mut self, symbol: DirectionalSymbol, wiggle_rule: bool, mut f: F) -> Result<()>
    where
        F: FnMut(Direction) -> Result<()>,
    {
        let target = self.get_coordinates_of_symbol(symbol);

        if self.current != target {
            let results = match (self.current, target) {
                (Point { x: 1, y: 0 }, Point { x: 2, y: 0 }) => [Direction::Right].as_slice(),
                (Point { x: 1, y: 0 }, Point { x: 0, y: 1 }) => [Direction::Down, Direction::Left].as_slice(),
                (Point { x: 1, y: 0 }, Point { x: 1, y: 1 }) => [Direction::Down].as_slice(),
                (Point { x: 1, y: 0 }, Point { x: 2, y: 1 }) => [Direction::Down, Direction::Right].as_slice(),

                (Point { x: 2, y: 0 }, Point { x: 1, y: 0 }) => [Direction::Left].as_slice(),
                (Point { x: 2, y: 0 }, Point { x: 0, y: 1 }) => {
                    if wiggle_rule {
                        [Direction::Left, Direction::Down, Direction::Left].as_slice()
                    } else {
                        [Direction::Down, Direction::Left, Direction::Left].as_slice()
                    }
                }
                (Point { x: 2, y: 0 }, Point { x: 1, y: 1 }) => [Direction::Left, Direction::Down].as_slice(),
                (Point { x: 2, y: 0 }, Point { x: 2, y: 1 }) => [Direction::Down].as_slice(),

                (Point { x: 0, y: 1 }, Point { x: 1, y: 0 }) => [Direction::Right, Direction::Up].as_slice(),
                (Point { x: 0, y: 1 }, Point { x: 2, y: 0 }) => [Direction::Right, Direction::Right, Direction::Up].as_slice(),
                (Point { x: 0, y: 1 }, Point { x: 1, y: 1 }) => [Direction::Right].as_slice(),
                (Point { x: 0, y: 1 }, Point { x: 2, y: 1 }) => [Direction::Right, Direction::Right].as_slice(),

                (Point { x: 1, y: 1 }, Point { x: 1, y: 0 }) => [Direction::Up].as_slice(),
                (Point { x: 1, y: 1 }, Point { x: 2, y: 0 }) => [Direction::Right, Direction::Up].as_slice(),
                (Point { x: 1, y: 1 }, Point { x: 0, y: 1 }) => [Direction::Left].as_slice(),
                (Point { x: 1, y: 1 }, Point { x: 2, y: 1 }) => [Direction::Right].as_slice(),

                (Point { x: 2, y: 1 }, Point { x: 1, y: 0 }) => [Direction::Left, Direction::Up].as_slice(),
                (Point { x: 2, y: 1 }, Point { x: 2, y: 0 }) => [Direction::Up].as_slice(),
                (Point { x: 2, y: 1 }, Point { x: 0, y: 1 }) => [Direction::Left, Direction::Left].as_slice(),
                (Point { x: 2, y: 1 }, Point { x: 1, y: 1 }) => [Direction::Left].as_slice(),

                _ => Err(format!("impossible move: {:?} -> {:?}", self.current, target))?,
            };

            for d in results {
                self.update(*d)?;
                f(*d)?;
            }
        }

        // if self.current.y == 0 {
        //     for _ in self.current.y..target.y {
        //         self.update(Direction::Down)?;
        //         f(Direction::Down)?;
        //     }
        // }

        // if target.x < self.current.x {
        //     for _ in target.x..self.current.x {
        //         self.update(Direction::Left)?;
        //         f(Direction::Left)?;
        //     }
        // } else if target.x > self.current.x {
        //     for _ in self.current.x..target.x {
        //         self.update(Direction::Right)?;
        //         f(Direction::Right)?;
        //     }
        // }

        // if target.y < self.current.y {
        //     for _ in target.y..self.current.y {
        //         self.update(Direction::Up)?;
        //         f(Direction::Up)?;
        //     }
        // } else if target.y > self.current.y {
        //     for _ in self.current.y..target.y {
        //         self.update(Direction::Down)?;
        //         f(Direction::Down)?;
        //     }
        // }

        Ok(())
    }
}

fn solve(sequence: &str) -> Result<u64> {
    // println!("TODO sequence: {}", sequence);

    let mut keypad_1 = DirectionalKeypad::new();
    let mut keypad_2 = DirectionalKeypad::new();
    let mut keypad_3 = DirectionalKeypad::new();
    let mut keypad_4 = NumericKeypad::new();

    // find the set of steps to execute on keypad 3 to get the sequence into keypad 4
    let mut keypad_3_directions = Vec::new();
    for c in sequence.chars() {
        let symbol = match c {
            '0'..='9' => NumericSymbol::Digit(c),
            'A' => NumericSymbol::Accept,
            _ => Err(format!("illegal character: {}", c))?,
        };
        // println!("TODO trying to type numberic symbol: {:?}", symbol);
        keypad_4.update_to(symbol, |d| {
            // println!("TODO     updating {:?}", d);
            keypad_3_directions.push(DirectionalSymbol::Direction(d));
            Ok(())
        })?;
        // println!("TODO     updating {:?}", DirectionalSymbol::Accept);
        keypad_3_directions.push(DirectionalSymbol::Accept);
    }
    // println!("");

    // now repeat that but for the sequence of steps you have to put into keypad 2 to get keypad 3 to type those directions
    let mut keypad_2_directions = Vec::new();
    for symbol in keypad_3_directions.iter() {
        // println!("TODO trying to type {:?}", symbol);
        keypad_3.update_to(*symbol, false, |d| {
            // println!("TODO     updating {:?}", d);
            keypad_2_directions.push(DirectionalSymbol::Direction(d));
            Ok(())
        })?;
        // println!("TODO     updating {:?}", DirectionalSymbol::Accept);
        keypad_2_directions.push(DirectionalSymbol::Accept);
    }
    // println!("");

    // and again for the sequence for keypad 1 to get keypad 2 to do that
    let mut keypad_1_directions = Vec::new();
    for symbol in keypad_2_directions.iter() {
        // println!("TODO trying to type {:?}", symbol);
        keypad_2.update_to(*symbol, true, |d| {
            // println!("TODO     updating {:?}", d);
            keypad_1_directions.push(DirectionalSymbol::Direction(d));
            Ok(())
        })?;
        // println!("TODO     updating {:?}", DirectionalSymbol::Accept);
        keypad_1_directions.push(DirectionalSymbol::Accept);
    }
    // println!("TODO keypad_1_directions.len(): {:?}", keypad_1_directions.len());
    // println!("");

    // TODO remove this
    print!("{}: ", sequence);
    for symbol in keypad_1_directions.iter() {
        let c = match symbol {
            DirectionalSymbol::Accept => 'A',
            DirectionalSymbol::Direction(Direction::Left) => '<',
            DirectionalSymbol::Direction(Direction::Right) => '>',
            DirectionalSymbol::Direction(Direction::Up) => '^',
            DirectionalSymbol::Direction(Direction::Down) => 'v',
        };
        print!("{}", c);
    }
    println!("");
    // for symbol in keypad_2_directions.iter() {
    //     let c = match symbol {
    //         DirectionalSymbol::Accept => 'A',
    //         DirectionalSymbol::Direction(Direction::Left) => '<',
    //         DirectionalSymbol::Direction(Direction::Right) => '>',
    //         DirectionalSymbol::Direction(Direction::Up) => '^',
    //         DirectionalSymbol::Direction(Direction::Down) => 'v',
    //     };
    //     print!("{}", c);
    // }
    // println!("");
    // for symbol in keypad_3_directions.iter() {
    //     let c = match symbol {
    //         DirectionalSymbol::Accept => 'A',
    //         DirectionalSymbol::Direction(Direction::Left) => '<',
    //         DirectionalSymbol::Direction(Direction::Right) => '>',
    //         DirectionalSymbol::Direction(Direction::Up) => '^',
    //         DirectionalSymbol::Direction(Direction::Down) => 'v',
    //     };
    //     print!("{}", c);
    // }
    // println!("");

    // TODO remove me
    // let mut keypad_1 = DirectionalKeypad::new();
    // let mut keypad_2 = DirectionalKeypad::new();
    // let mut keypad_3 = DirectionalKeypad::new();
    // let mut keypad_4 = NumericKeypad::new();
    // for d in keypad_1_directions.iter() {
    //     match d {
    //         DirectionalSymbol::Accept => {
    //             match keypad_2.get()? {
    //                 DirectionalSymbol::Accept => {
    //                     match keypad_3.get()? {
    //                         DirectionalSymbol::Accept => {
    //                             let value = keypad_4.get()?;
    //                             println!("TODO what did we type? {:?}", value);
    //                         }
    //                         DirectionalSymbol::Direction(direction) => keypad_4.update(direction)?,
    //                     };
    //                 }
    //                 DirectionalSymbol::Direction(direction) => keypad_3.update(direction)?,
    //             };
    //         }
    //         DirectionalSymbol::Direction(direction) => keypad_2.update(*direction)?,
    //     };
    // }
    // println!("");

    Ok(keypad_1_directions.len() as u64)
}

#[allow(dead_code)]
fn do_it(path: &str) -> Result<u64> {
    let file_contents = BufReader::new(File::open(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("puzzle-inputs").join(path),
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

    let r = Regex::new("^([0-9]+)A$")?;
    let mut result = 0;
    for line in file_contents.iter() {
        let (_, [number_part]) = r.captures(&line).ok_or(format!("regex failed: {}", line))?.extract();
        let number: u64 = number_part.parse()?;
        // println!("TODO number part = {}", number);
        let sequence = solve(line)?;
        result += sequence * number;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day21-sample.txt").unwrap(), 126384);
    }

    #[test]
    pub fn test_real() {
        // 217676, too high
        assert_eq!(do_it("day21.txt").unwrap(), 0);
    }
}
