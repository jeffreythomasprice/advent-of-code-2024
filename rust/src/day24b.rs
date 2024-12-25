use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    mem::swap,
    num::ParseIntError,
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
enum Operation {
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Input {
    Input(String),
    Gate(Gate),
}

impl Input {
    fn new(wires: &HashSet<String>, gates: &HashMap<String, (String, Operation, String)>, name: &str) -> Result<Self> {
        Ok(match (wires.get(name), gates.get(name)) {
            (Some(wire), None) => Self::Input(wire.clone()),
            (None, Some(_)) => Self::Gate(Gate::new(wires, gates, name)?),
            (None, None) => Err(format!("no wire or gate named {:?}", name))?,
            (Some(_), Some(_)) => Err(format!("both a wire and a gate are named {:?}", name))?,
        })
    }

    fn diff(a: &Self, b: &Self) -> Option<(Input, Input)> {
        match (a, b) {
            (Input::Input(a), Input::Input(b)) => {
                if a == b {
                    None
                } else {
                    Some((Self::Input(a.clone()), Self::Input(b.clone())))
                }
            }
            (Input::Input(a), Input::Gate(b)) => Some((Self::Input(a.clone()), Self::Gate(b.clone()))),
            (Input::Gate(a), Input::Input(b)) => Some((Self::Gate(a.clone()), Self::Input(b.clone()))),
            (Input::Gate(a), Input::Gate(b)) => Gate::diff(a, b),
        }
    }
}

impl PartialOrd for Input {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Input {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Input::Input(a), Input::Input(b)) => a.cmp(b),
            (Input::Input(_), Input::Gate(_)) => Ordering::Less,
            (Input::Gate(_), Input::Input(_)) => Ordering::Greater,
            (Input::Gate(a), Input::Gate(b)) => {
                let result = a.input1.cmp(&b.input1);
                if result != Ordering::Equal {
                    result
                } else {
                    a.input2.cmp(&b.input2)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Gate {
    name: String,
    input1: Box<Input>,
    input2: Box<Input>,
    operation: Operation,
}

impl Gate {
    fn new(wires: &HashSet<String>, gates: &HashMap<String, (String, Operation, String)>, name: &str) -> Result<Self> {
        let (input1, operation, input2) = gates.get(name).ok_or(format!("no such gate: {:?}", name))?;
        let input1 = Input::new(wires, gates, input1)?;
        let input2 = Input::new(wires, gates, input2)?;
        Ok(Self {
            name: name.to_string(),
            input1: Box::new(input1),
            input2: Box::new(input2),
            operation: *operation,
        }
        .normalized())
    }

    fn new_adder(bit: u32) -> Result<(Self, Self)> {
        if bit == 0 {
            // A ^ B
            let result = Self {
                name: format!("z{:02}", bit),
                input1: Box::new(Input::Input(format!("x{:02}", bit))),
                input2: Box::new(Input::Input(format!("y{:02}", bit))),
                operation: Operation::Xor,
            }
            .normalized();
            // A & B
            let carry = Self {
                name: format!("c{:02}", bit),
                input1: Box::new(Input::Input(format!("x{:02}", bit))),
                input2: Box::new(Input::Input(format!("y{:02}", bit))),
                operation: Operation::And,
            }
            .normalized();
            Ok((result, carry))
        } else {
            let (_, previous_carry) = Self::new_adder(bit - 1)?;
            // A ^ B ^ C
            let result = Self {
                name: format!("z{:02}", bit),
                input1: Box::new(Input::Gate(
                    Self {
                        name: format!("partial{:02}", bit),
                        input1: Box::new(Input::Input(format!("x{:02}", bit))),
                        input2: Box::new(Input::Input(format!("y{:02}", bit))),
                        operation: Operation::Xor,
                    }
                    .normalized(),
                )),
                input2: Box::new(Input::Gate(previous_carry.clone())),
                operation: Operation::Xor,
            }
            .normalized();
            /*
            any variant of:
            (A & B) | (B & C) | (A & C)
            (A & B) | ((A ^ B) & C)
            */
            let carry = Self {
                name: format!("c{:02}", bit),
                input1: Box::new(Input::Gate(
                    Self {
                        name: format!("partial{:02}", bit),
                        input1: Box::new(Input::Gate(
                            Self {
                                name: format!("partial{:02}", bit),
                                input1: Box::new(Input::Input(format!("x{:02}", bit))),
                                input2: Box::new(Input::Input(format!("y{:02}", bit))),
                                operation: Operation::Xor,
                            }
                            .normalized(),
                        )),
                        input2: Box::new(Input::Gate(previous_carry)),
                        operation: Operation::And,
                    }
                    .normalized(),
                )),
                input2: Box::new(Input::Gate(
                    Self {
                        name: format!("partial{:02}", bit),
                        input1: Box::new(Input::Input(format!("x{:02}", bit))),
                        input2: Box::new(Input::Input(format!("y{:02}", bit))),
                        operation: Operation::And,
                    }
                    .normalized(),
                )),
                operation: Operation::Or,
            }
            .normalized();
            Ok((result, carry))
        }
    }

    fn normalized(self) -> Self {
        let mut input1 = self.input1;
        let mut input2 = self.input2;
        // println!("TODO normalizing input1={:?}, input2={:?}", input1, input2);
        if input1.as_ref().cmp(input2.as_ref()) == Ordering::Greater {
            // println!("TODO they are backwards");
            swap(&mut input1, &mut input2);
        }
        // println!("TODO after normalizing input1={:?}, input2={:?}", input1, input2);
        Self {
            name: self.name,
            input1,
            input2,
            operation: self.operation,
        }
    }

    fn human_readable_string(&self, with_names: bool) -> String {
        let left = match self.input1.as_ref() {
            Input::Input(name) => name,
            Input::Gate(gate) => &format!("({})", gate.human_readable_string(with_names)),
        };
        let op = match self.operation {
            Operation::And => "AND",
            Operation::Or => "OR",
            Operation::Xor => "XOR",
        };
        let right = match self.input2.as_ref() {
            Input::Input(name) => name,
            Input::Gate(gate) => &format!("({})", gate.human_readable_string(with_names)),
        };
        if with_names {
            format!("{}({} {} {})", self.name, left, op, right)
        } else {
            format!("{} {} {}", left, op, right)
        }
    }

    fn diff(a: &Self, b: &Self) -> Option<(Input, Input)> {
        // TODO account for differences in operator?
        let diff1 = Input::diff(a.input1.as_ref(), b.input1.as_ref());
        let diff2 = Input::diff(a.input2.as_ref(), b.input2.as_ref());
        diff1.or(diff2)
    }

    fn get_all_names(&self, results: &mut Vec<String>) {
        results.push(self.name.clone());
        if let Input::Gate(gate) = self.input1.as_ref() {
            gate.get_all_names(results);
        }
        if let Input::Gate(gate) = self.input2.as_ref() {
            gate.get_all_names(results);
        }
    }

    fn fix_names(&mut self, gates: &HashMap<String, Gate>) {
        if let Input::Gate(gate) = self.input1.as_mut() {
            gate.fix_names(gates);
        }
        if let Input::Gate(gate) = self.input2.as_mut() {
            gate.fix_names(gates);
        }
        if let Some(real) = gates
            .values()
            .find(|gate| gate.human_readable_string(false) == self.human_readable_string(false))
        {
            self.name = real.name.clone();
        } else {
            println!("TODO failed to find name");
        }
    }
}

fn get_number_from_prefix(values: &HashMap<String, bool>, prefix: &str) -> u64 {
    let mut values = values.iter().filter(|(name, _)| name.starts_with(prefix)).collect::<Vec<_>>();
    values.sort_by(|(a, _), (b, _)| a.cmp(b));

    let mut shift = 0;
    let mut result = 0;

    for (_, value) in values {
        result += if *value { 1 << shift } else { 0 };
        shift += 1;
    }

    result
}

#[allow(dead_code)]
fn do_it<F>(path: &str, z_func: F) -> Result<String>
where
    F: Fn(u64, u64) -> u64,
{
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

    // key = name, value = initial value
    let mut values = HashMap::new();
    // key = output, value = (input1, operation, input2)
    let mut gates = HashMap::new();

    let input_regex = Regex::new(r"^([a-zA-Z0-9]+): (0|1)$")?;
    let gate_regex = Regex::new(r"^([a-zA-Z0-9]+) (AND|OR|XOR) ([a-zA-Z0-9]+) -> ([a-zA-Z0-9]+)$")?;
    for line in file_contents {
        if let Some(captures) = input_regex.captures(&line) {
            let (_, [name, value]) = captures.extract();
            values.insert(name.to_string(), value == "1");
        } else if let Some(captures) = gate_regex.captures(&line) {
            let (_, [input1, op, input2, output]) = captures.extract();
            let op = match op {
                "AND" => Operation::And,
                "OR" => Operation::Or,
                "XOR" => Operation::Xor,
                _ => Err(format!("invalid operation: {:?}", op))?,
            };
            gates.insert(output.to_string(), (input1.to_string(), op, input2.to_string()));
        } else {
            Err(format!("error parsing line: {:?}", line))?;
        }
    }

    let wires = HashSet::from_iter(values.keys().cloned());

    let z_regex = Regex::new(r"^z[0-9]+$")?;
    let mut gates = gates
        .keys()
        .map(|name| Gate::new(&wires, &gates, name))
        .collect::<Result<Vec<_>>>()?;
    gates.sort_by(|a, b| a.name.cmp(&b.name));
    let gates_map = HashMap::from_iter(gates.iter().map(|gate| (gate.name.clone(), gate.clone())));
    // let mut all_wrong_names = HashMap::new();
    for gate in gates.iter().filter(|x| z_regex.is_match(&x.name)) {
        let bit = gate.name[1..].parse()?;
        let (expected, _) = Gate::new_adder(bit)?;
        // TODO should be doing a tree diff?
        if expected.human_readable_string(false) != gate.human_readable_string(false) {
            let mut expected = expected;
            expected.fix_names(&gates_map);

            println!("TODO difference at {}", gate.name);
            println!("TODO actual {}", gate.human_readable_string(true));
            println!("TODO expected {}", expected.human_readable_string(true));
            // if let Some((actual, expected)) = Gate::diff(gate, &expected) {
            //     println!("TODO diff, actual = {:?}", actual);
            //     println!("TODO diff, should have been = {:?}", expected);

            //     /*
            //     TODO find the gate that matches the expected side of the diff
            //     */
            // }
            // println!();

            // let mut names = Vec::new();
            // gate.get_all_names(&mut names);
            // println!("TODO {} is wrong, depends on {:?}", gate.name, names);
            // for name in names {
            //     all_wrong_names.entry(name).and_modify(|e| *e += 1).or_insert(1);
            // }
        }
    }
    // let mut all_wrong_names = all_wrong_names.iter().collect::<Vec<_>>();
    // all_wrong_names.sort_by(|(_, a), (_, b)| a.cmp(b));
    // for (name, count) in all_wrong_names.iter() {
    //     println!("TODO name {} shows up {} times", name, count);
    // }

    todo!()
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day24.txt", |x, y| x + y).unwrap(), "");
    }
}
