use std::{
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
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

#[derive(Clone)]
struct VM {
    a: u64,
    b: u64,
    c: u64,
    program: Vec<u8>,
    instruction_pointer: usize,
    is_halted: bool,
}

impl VM {
    fn new(a: u64, b: u64, c: u64, program: Vec<u8>) -> Self {
        Self {
            a,
            b,
            c,
            program,
            instruction_pointer: 0,
            is_halted: false,
        }
    }

    fn step<F>(&mut self, mut output: F) -> Result<bool>
    where
        F: FnMut(u8) -> bool,
    {
        if let Some(instruction) = self.read_instruction() {
            match instruction {
                // adv
                0 => {
                    if let Some(data) = self.read_combo_data()? {
                        // println!("TODO adv {}", data);
                        self.a /= 2u64.pow(data as u32);
                        // println!("TODO a = {}", self.a);
                        Ok(true)
                    } else {
                        Ok(true)
                    }
                }
                // bxl
                1 => {
                    if let Some(data) = self.read_literal_data() {
                        // println!("TODO bxl {}", data);
                        self.b ^= data as u64;
                        // println!("TODO b = {}", self.b);
                        Ok(true)
                    } else {
                        Ok(true)
                    }
                }
                // bst
                2 => {
                    if let Some(data) = self.read_combo_data()? {
                        // println!("TODO bst {}", data);
                        self.b = data % 8;
                        // println!("TODO b = {}", self.b);
                        Ok(true)
                    } else {
                        Ok(true)
                    }
                }
                // jnz
                3 => {
                    if let Some(data) = self.read_literal_data() {
                        // println!("TODO jnz {}", data);
                        if self.a != 0 {
                            self.instruction_pointer = data as usize;
                            // println!("TODO after jump, ip = {}", self.instruction_pointer);
                        } else {
                            // println!("TODO did not jump, ip = {}", self.instruction_pointer);
                        }
                        Ok(true)
                    } else {
                        Ok(true)
                    }
                }
                // bxc
                4 => {
                    // println!("TODO bxc");
                    _ = self.read();
                    self.b = self.b ^ self.c;
                    // println!("TODO b = {}", self.b);
                    Ok(true)
                }
                // out
                5 => {
                    if let Some(data) = self.read_combo_data()? {
                        // println!("TODO out {}", data);
                        // println!("TODO outputting {}", (data % 8) as u8);
                        Ok(output((data % 8) as u8))
                    } else {
                        Ok(true)
                    }
                }
                // bdv
                6 => {
                    if let Some(data) = self.read_combo_data()? {
                        // println!("TODO bdv {}", data);
                        self.b = self.a / 2u64.pow(data as u32);
                        // println!("TODO b = {}", self.b);
                        Ok(true)
                    } else {
                        Ok(true)
                    }
                }
                // cdv
                7 => {
                    if let Some(data) = self.read_combo_data()? {
                        // println!("TODO cdv {}", data);
                        self.c = self.a / 2u64.pow(data as u32);
                        // println!("TODO c = {}", self.c);
                        Ok(true)
                    } else {
                        Ok(true)
                    }
                }
                _ => Err(format!("invalid instruction: {}", instruction))?,
            }
        } else {
            Ok(true)
        }
    }

    fn read_instruction(&mut self) -> Option<u8> {
        self.read()
    }

    fn read_literal_data(&mut self) -> Option<u8> {
        self.read()
    }

    fn read_combo_data(&mut self) -> Result<Option<u64>> {
        match self.read_literal_data() {
            Some(data) => match data {
                0 | 1 | 2 | 3 => Ok(Some(data as u64)),
                4 => Ok(Some(self.a)),
                5 => Ok(Some(self.b)),
                6 => Ok(Some(self.c)),
                _ => Err(format!("invalid combo data: {}", data))?,
            },
            None => Ok(None),
        }
    }

    fn read(&mut self) -> Option<u8> {
        if self.instruction_pointer >= self.program.len() {
            self.is_halted = true;
            None
        } else {
            let result = self.program[self.instruction_pointer];
            self.instruction_pointer += 1;
            Some(result)
        }
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

    // ignore empty lines
    let file_contents = file_contents
        .iter()
        .filter_map(|line| if line.is_empty() { None } else { Some(line) })
        .collect::<Vec<_>>();

    if file_contents.len() != 4 {
        Err(format!(
            "expected exactly 4 lines, got {}",
            file_contents.len()
        ))?;
    }

    let (_, [register_a]) = Regex::new("^Register A: ([0-9]+)$")?
        .captures(&file_contents[0])
        .ok_or("regex failed")?
        .extract();
    let (_, [register_b]) = Regex::new("^Register B: ([0-9]+)$")?
        .captures(&file_contents[1])
        .ok_or("regex failed")?
        .extract();
    let (_, [register_c]) = Regex::new("^Register C: ([0-9]+)$")?
        .captures(&file_contents[2])
        .ok_or("regex failed")?
        .extract();
    let (_, [program]) = Regex::new("^Program: ([0-9,]+)$")?
        .captures(&file_contents[3])
        .ok_or("regex failed")?
        .extract();

    let vm = VM::new(
        register_a.parse()?,
        register_b.parse()?,
        register_c.parse()?,
        program
            .split(",")
            .map(|x| Ok(x.parse()?))
            .collect::<Result<Vec<_>>>()?,
    );

    let goal = vm.program.clone();

    // TODO start at 0
    let mut a = 0;
    let mut output = Vec::with_capacity(goal.len());
    loop {
        let mut vm = vm.clone();
        vm.a = a;
        output.clear();
        while !vm.is_halted {
            if !vm.step(|out| {
                output.push(out);
                // println!("TODO new output: {:?}", output);
                // TODO put early exit back
                // goal[output.len() - 1] == out
                true
            })? {
                // println!("TODO aborting, output so far: {:?}", output);
                break;
            }
        }
        println!("TODO a: {}", a);
        println!("TODO goal: {:?}", goal);
        println!("TODO output: {:?}", output);
        if output == goal {
            return Ok(a);
        } else {
            a += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day17b-sample.txt").unwrap(), 117440);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day17.txt",).unwrap(), 0);
    }
}
