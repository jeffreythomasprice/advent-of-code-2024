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
        let prize_x = prize_x.parse::<i64>()? + 10000000000000;
        let prize_y = prize_y.parse::<i64>()? + 10000000000000;

        /*
        it's just linear systems

        A*Ax + B*Bx = Px
        A*Ay + B*By = Py
        T = 3*A + B

        A*Ax + B*Bx = Px
        A = (Px - B*Bx)/Ax

        B = (Py - Px/Ax*Ay)/(By - Bx/Ax*Ay)

        substitue B back into the equation for A using the other axis, y
        A*Ay + B*By = Py
        A = (Py - B*By)/Ay
        */
        let b = ((prize_y as f64) - (prize_x as f64) / (button_a_x as f64) * (button_a_y as f64))
            / ((button_b_y as f64)
                - (button_b_x as f64) / (button_a_x as f64) * (button_a_y as f64));
        let a = ((prize_y as f64) - b * (button_b_y as f64)) / (button_a_y as f64);

        let a_guess = a.floor() as i64;
        let initial_a = a_guess - 50;
        let max_a = a_guess + 50;

        let mut min_tokens: Option<i64> = None;
        for a in initial_a..=max_a {
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
                (Some(b_x), Some(b_y)) if b_x == b_y && b_x >= 0 => {
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
    pub fn test_real() {
        assert_eq!(do_it("day13.txt").unwrap(), 74478585072604);
    }
}
