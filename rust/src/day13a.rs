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

#[allow(dead_code)]
fn do_it(path: &str) -> Result<i64> {
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

    let button_a_regex = Regex::new(r"^Button A: X\+([0-9]+), Y\+([0-9]+)$")?;
    let button_b_regex = Regex::new(r"^Button B: X\+([0-9]+), Y\+([0-9]+)$")?;
    let prize_regex = Regex::new(r"^Prize: X=([0-9]+), Y=([0-9]+)$")?;
    let mut result = 0;
    for i in (0..file_contents.len()).step_by(3) {
        let button_a_string = &file_contents[i];
        let button_b_string = &file_contents[i + 1];
        let prize_string = &file_contents[i + 2];

        let (_, [button_a_x, button_a_y]) = button_a_regex
            .captures(&button_a_string)
            .ok_or_else(|| format!("expected button A string: {}", button_a_string))?
            .extract();
        let (_, [button_b_x, button_b_y]) = button_b_regex
            .captures(&button_b_string)
            .ok_or_else(|| format!("expected button B string: {}", button_b_string))?
            .extract();
        let (_, [prize_x, prize_y]) = prize_regex
            .captures(&prize_string)
            .ok_or_else(|| format!("expected prize string: {}", prize_string))?
            .extract();

        let button_a_x = button_a_x.parse::<i64>()?;
        let button_a_y = button_a_y.parse::<i64>()?;
        let button_b_x = button_b_x.parse::<i64>()?;
        let button_b_y = button_b_y.parse::<i64>()?;
        let prize_x = prize_x.parse::<i64>()?;
        let prize_y = prize_y.parse::<i64>()?;

        /*
        how many presses of each button gets to prize?
        max of either button is 100, prize is considered unreachable if button presses go over that limit
        button A consumes 3 tokens
        button B consumes 1 token
        how many tokens needed to get the prize, if it's reachable at all?

        A = number of times A is pressed
        B = number of times B is pressed

        Ax = X increment when A is pressed
        Ay = Y increment when A is pressed

        Bx = X increment when B is pressed
        By = Y increment when B is pressed

        Px, Py = coordinates of prize

        A*Ax + B*Bx = Px
        A*Ay + B*By = Py
        T = 3*A + B
        A <= 100
        B <= 100

        goal is to solve for min(T)

        B = (Px - A*Ax)/Bx
        B = (Py - A*Ay)/By
        A = (Px - B*Bx)/Ax
        A = (Py - B*By)/Ay

        given that you can solve for one number with the other, you can just iterate over possible values of one number
        so for all A in 0 to 100, inclusive {
            solve for B
            if B < 0, continue
            if B > 100, continue
            if B wouldn't be an integer, continue
            solve for T, keep min T found so far
        }
        if we found a min(T), we have Some(T), otherwise None
        */

        let mut min_tokens: Option<i64> = None;
        for a in 0..=100 {
            let b_x = {
                let numerator = prize_x - a * button_a_x;
                if numerator % button_b_x == 0 {
                    Some(numerator / button_b_x)
                } else {
                    None
                }
            };
            let b_y = {
                let numerator = prize_y - a * button_a_y;
                if numerator % button_b_y == 0 {
                    Some(numerator / button_b_y)
                } else {
                    None
                }
            };
            match (b_x, b_y) {
                // all rules match
                (Some(b_x), Some(b_y)) if b_x == b_y && b_x <= 100 => {
                    let t = a * 3 + b_x;
                    // keep only if it's smaller than the current vlaue
                    min_tokens = Some(if let Some(existing) = min_tokens {
                        existing.min(t)
                    } else {
                        t
                    });
                }
                // didn't match the rules, skip this one
                (_, _) => (),
            }
        }
        if let Some(t) = min_tokens {
            result += t;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day13-sample.txt").unwrap(), 480);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day13.txt").unwrap(), 39748);
    }
}
