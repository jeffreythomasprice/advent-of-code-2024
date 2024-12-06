// TODO redo this, current solution is brute force and takes several minutes

use std::{
    collections::HashMap,
    env,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseIntError,
    path::Path,
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

#[derive(Debug, Clone)]
struct Rule {
    left: u32,
    right: u32,
}

impl Rule {
    fn new(left: &str, right: &str) -> Result<Self> {
        Ok(Self {
            left: left.parse()?,
            right: right.parse()?,
        })
    }
}

#[derive(Debug)]
struct Rules(HashMap<u32, Vec<Rule>>);

impl Rules {
    fn new(rules: Vec<Rule>) -> Self {
        Self({
            let mut result = HashMap::new();
            for rule in rules.into_iter() {
                result.entry(rule.left).or_insert_with(Vec::new).push(rule);
            }
            result
        })
    }

    fn check(&self, left: u32, right: u32) -> bool {
        match self.0.get(&left) {
            Some(rules) => rules.iter().any(|rule| rule.right == right),
            None => false,
        }
    }

    fn possible_choices(&self, left: u32) -> Option<&Vec<Rule>> {
        match self.0.get(&left) {
            Some(rules) => Some(rules),
            None => None,
        }
    }
}

#[derive(Debug, Clone)]
struct Sequence(Vec<u32>);

impl Sequence {
    fn new(v: Vec<u32>) -> Self {
        Self(v)
    }

    fn is_valid(&self, rules: &Rules) -> bool {
        for i in 0..(self.0.len() - 1) {
            let j = i + 1;
            let left = self.0[i];
            let right = self.0[j];
            if !rules.check(left, right) {
                return false;
            }
        }
        true
    }

    fn new_with_numbers(numbers: &[u32], rules: &Rules) -> Option<Sequence> {
        let mut to_visit = numbers.iter().map(|x| (None, *x)).collect::<Vec<_>>();
        let mut current = Vec::new();
        loop {
            match to_visit.pop() {
                // we have something to try
                Some((prev, next)) => {
                    // pop off current while the right hand side of current doesn't match the left hand side of the next rule
                    match prev {
                        // we had some previous node, pop until our current head matches the previous
                        Some(prev) => loop {
                            match current.last() {
                                Some((_, current_head)) => {
                                    if *current_head != prev {
                                        current.pop();
                                    } else {
                                        break;
                                    }
                                }
                                None => break,
                            };
                        },
                        // we didn't have a previous node, that means we're starting a new root attempt, so clear the current one
                        None => current.clear(),
                    };

                    // add it to our possible solution
                    current.push((prev, next));
                    let possible_result =
                        Sequence::new(current.iter().map(|(_, x)| *x).collect::<Vec<_>>());

                    // no more things to add
                    if possible_result.0.len() == numbers.len() {
                        // success, we're done
                        if possible_result.is_valid(rules) {
                            return Some(possible_result);
                        }
                        // not a solution
                        else {
                            current.pop();
                            continue;
                        }
                    }

                    // there must be at least one more number in this sequence before we can check if we're done
                    // add all possible remaining choices
                    if let Some(possible_choices) = rules.possible_choices(next) {
                        for next in possible_choices.iter().filter(|next| {
                            let is_valid_number = numbers.contains(&next.right);
                            let not_already_visited =
                                !current.iter().any(|(_, x)| *x == next.right);
                            is_valid_number && not_already_visited
                        }) {
                            to_visit.push((Some(next.left), next.right));
                        }
                    }
                }
                // we're out of possibilities
                None => {
                    return None;
                }
            };
        }
    }
}

#[allow(dead_code)]
fn do_it(path: &str) -> Result<u32> {
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

    let divider_regex = Regex::new(r"^(\d+)\|(\d+)$")?;
    let sequence_regex = Regex::new(r"^(\d+)(?:,(\d+))*$")?;

    let mut iter = file_contents.into_iter();
    let rules = iter
        .by_ref()
        .take_while(|line| divider_regex.is_match(line))
        .collect::<Vec<_>>();
    let sequences = iter
        .by_ref()
        .take_while(|line| sequence_regex.is_match(line))
        .collect::<Vec<_>>();
    let remainder = iter.collect::<Vec<_>>();
    if !remainder.is_empty() {
        Err(format!("unmatched line at end of input: {:?}", remainder))?;
    }

    let rules = rules
        .into_iter()
        .map(|line| {
            let (_, [left, right]) = divider_regex
                .captures(&line)
                .ok_or("shold be impossible, already matched")?
                .extract();
            Rule::new(left, right)
        })
        .collect::<Result<Vec<_>>>()?;

    let rules = Rules::new(rules);

    let sequences = sequences
        .into_iter()
        .map(|line| {
            line.split(",")
                .map(|num| Ok(num.trim().parse::<u32>()?))
                .collect::<Result<Vec<_>>>()
                .map(Sequence::new)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(sequences
        .iter()
        .filter(|sequence| !sequence.is_valid(&rules))
        .map(|sequence| Sequence::new_with_numbers(&sequence.0, &rules))
        .collect::<Option<Vec<_>>>()
        .ok_or("failed to find a valid ordering for at least one sequence")?
        .iter()
        .map(|sequence| sequence.0[sequence.0.len() / 2])
        .sum())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day05-sample.txt").unwrap(), 123);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day05.txt").unwrap(), 6142);
    }
}
