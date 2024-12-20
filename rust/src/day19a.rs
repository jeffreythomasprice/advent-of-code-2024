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
    fn is_possible<'a>(
        &'a self,
        choices: &HashMap<char, Vec<Towel>>,
        unsolvable: &mut HashSet<&'a [char]>,
    ) -> bool {
        is_possible(&self.0, choices, unsolvable)
    }
}

fn is_possible<'a>(
    pattern: &'a [char],
    choices: &HashMap<char, Vec<Towel>>,
    unsolvable: &mut HashSet<&'a [char]>,
) -> bool {
    /*
    find all the choices that match the start of the pattern
    for all those, recurse with the remaining pattern

    result:
    if the pattern is empty, true
    if the pattern is non-empty, and no choices match, false
    if the pattern is non-empty, and some choices match, true if at any of the recursions return true, false otherwise
    */

    if pattern.is_empty() {
        true
    } else {
        if unsolvable.contains(pattern) {
            false
        } else {
            if let Some(possible_choices) = choices.get(&pattern[0]) {
                let result = possible_choices
                    .iter()
                    .find(|towel| {
                        pattern.starts_with(&towel.0)
                            && is_possible(&pattern[towel.0.len()..], choices, unsolvable)
                    })
                    .is_some();
                if !result {
                    unsolvable.insert(pattern);
                }
                result
            } else {
                false
            }
        }
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
    let choices = HashSet::<Towel>::from_iter(choices.into_iter())
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

    let mut unsolvable = HashSet::new();

    Ok(patterns
        .iter()
        .filter(|p| p.is_possible(&choices_by_first_latter, &mut unsolvable))
        .count())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day19-sample.txt").unwrap(), 6);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day19.txt").unwrap(), 298);
    }
}
