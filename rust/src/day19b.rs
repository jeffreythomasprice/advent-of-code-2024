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

#[derive(Debug, PartialEq, Eq, Hash)]
struct Towel(Vec<char>);

#[derive(Debug)]
struct Pattern(Vec<char>);

impl Pattern {
    fn count_possible<'a>(
        &'a self,
        choices: &HashMap<char, Vec<Towel>>,
        answers: &mut HashMap<&'a [char], usize>,
    ) -> usize {
        count_possible(&self.0, choices, answers)
    }
}

fn count_possible<'a>(
    pattern: &'a [char],
    choices: &HashMap<char, Vec<Towel>>,
    answers: &mut HashMap<&'a [char], usize>,
) -> usize {
    /*
    find all the choices that match the start of the pattern
    for all those, recurse with the remaining pattern

    result:
    if the pattern is empty, there is one way to make this
    if the pattern is non-empty, return the sum of all the choices that match
    */

    if pattern.is_empty() {
        1
    } else if let Some(result) = answers.get(pattern) {
        *result
    } else if let Some(possible_choices) = choices.get(&pattern[0]) {
        let result = possible_choices
            .iter()
            .map(|towel| {
                if pattern.starts_with(&towel.0) {
                    count_possible(&pattern[towel.0.len()..], choices, answers)
                } else {
                    0
                }
            })
            .sum();
        answers.insert(pattern, result);
        result
    } else {
        answers.insert(pattern, 0);
        0
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

    let choices = file_contents[0]
        .split(",")
        .map(|x| Towel(x.trim().chars().collect::<Vec<_>>()))
        .collect::<Vec<_>>();

    // remove duplicate towels
    let choices = HashSet::<Towel>::from_iter(choices)
        .into_iter()
        .collect::<Vec<_>>();

    let mut choices_by_first_latter = HashMap::new();
    for choice in choices.into_iter() {
        choices_by_first_latter
            .entry(choice.0[0])
            .or_insert(Vec::new())
            .push(choice);
    }

    let patterns = file_contents[1..]
        .iter()
        .map(|pattern| Pattern(pattern.chars().collect::<Vec<_>>()))
        .collect::<Vec<_>>();

    let mut answers = HashMap::new();

    Ok(patterns
        .iter()
        .map(|p| p.count_possible(&choices_by_first_latter, &mut answers))
        .sum())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day19-sample.txt").unwrap(), 16);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day19.txt").unwrap(), 572248688842069);
    }
}
