use std::{
    collections::{HashMap, HashSet},
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

#[derive(Debug)]
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
            Some(rules) => rules.iter().find(|rule| rule.right == right).is_some(),
            None => false,
        }
    }

    fn possible_choices(&self, left: u32) -> Vec<u32> {
        match self.0.get(&left) {
            Some(rules) => rules.iter().map(|rule| rule.right).collect::<Vec<_>>(),
            None => Vec::new(),
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
        /*
        to_visit = stack()
        push all remainder to to_visit

        current = stack()
        while true {
            current.push(to_visit.pop())
            if current is the right length and is solved {
                return current
            }
            if current is the right length and is not solved {
                current.pop()
                continue
            }
            find all nodes we can visit from current based on the rules, and what isn't already in the current stack
            push all those numbers onto to_visit
        }
        */

        println!("TODO start, remainder = {:?}", numbers);
        println!("TODO rules = {:?}", rules);

        let mut to_visit = numbers.into_iter().map(|x| *x).collect::<Vec<_>>();
        let mut current = Vec::new();
        loop {
            println!("TODO start of loop, to_visit = {:?}", to_visit);
            match to_visit.pop() {
                // we have something to try
                Some(next) => {
                    // add it to our possible solution
                    current.push(next);
                    println!("TODO pushed {} to current, new current {:?}", next, current);
                    let possible_result = Sequence::new(current.clone());

                    // no more things to add
                    if possible_result.0.len() == numbers.len() {
                        // success, we're done
                        if possible_result.is_valid(rules) {
                            println!("TODO success = {:?}", possible_result);
                            return Some(possible_result);
                        }
                        // not a solution
                        else {
                            current.pop();
                            println!("TODO not a solution, popped, new current = {:?}", current);
                            continue;
                        }
                    }

                    // there must be at least one more number in this sequence before we can check if we're done
                    // add all possible remaining choices
                    let possible_choices = rules.possible_choices(next);
                    let possible_choices = possible_choices
                        .iter()
                        .filter(|next| numbers.contains(next) && !current.contains(*next))
                        .collect::<Vec<_>>();
                    if possible_choices.is_empty() {
                        current.pop();
                        println!(
                            "TODO no choices from here, backing up, current = {:?}",
                            current
                        );
                    } else {
                        for next in possible_choices {
                            println!("TODO pushed {} to to_visit", next);
                            to_visit.push(*next);
                        }
                        println!("TODO after pushing to to_visit = {:?}", to_visit);
                    }
                }
                // we're out of possibilities
                None => {
                    println!("TODO to_visit is empty");
                    return None;
                }
            };
        }
    }

    fn new_with_numbers_2(numbers: &[u32], rules: &Rules) -> Option<Sequence> {
        fn f(partial: Vec<u32>, remaining: HashSet<u32>, rules: &Rules) -> Option<Sequence> {
            // println!("TODO partial={partial:?}, remaining={remaining:?}");
            if remaining.is_empty() {
                let possible_result = Sequence::new(partial);
                if possible_result.is_valid(rules) {
                    return Some(possible_result);
                } else {
                    return None;
                }
            }

            let possible_choices = rules.possible_choices(*partial.last()?);
            for next in possible_choices
                .iter()
                .filter(|next| remaining.contains(next))
            {
                let mut partial = partial.clone();
                partial.push(*next);
                let mut remaining = remaining.clone();
                remaining.remove(next);
                if let Some(result) = f(partial, remaining, rules) {
                    return Some(result);
                }
            }
            return None;
        }

        let numbers = {
            let mut r = HashSet::new();
            for x in numbers {
                r.insert(*x);
            }
            r
        };

        for first_number in numbers.iter() {
            let mut remainder = numbers.clone();
            remainder.remove(first_number);
            if let Some(result) = f(vec![*first_number], remainder, rules) {
                return Some(result);
            }
        }
        None
    }

    fn new_with_numbers_3(numbers: &[u32], rules: &Rules) -> Option<Sequence> {
        genetic_algorithm(
            || Sequence(numbers.iter().map(|x| *x).collect()),
            |x| {
                let i = rand::random::<usize>() % x.0.len();
                let j = rand::random::<usize>() % x.0.len();
                x.0.swap(i, j);
            },
            20,
            |x| {
                let mut result = 0;
                for i in 0..(x.0.len() - 1) {
                    let j = i + 1;
                    if rules.check(x.0[i], x.0[j]) {
                        result += 1;
                    }
                }
                result
            },
            |x| x.is_valid(rules),
        )
    }
}

fn genetic_algorithm<T, N, P, S, G>(
    make_new: N,
    permute: P,
    population_size: usize,
    score: S,
    is_a_solution: G,
) -> Option<T>
where
    T: Clone,
    N: Fn() -> T,
    P: Fn(&mut T),
    S: Fn(&T) -> u32,
    G: Fn(&T) -> bool,
{
    /*
    generate a bunch of new items at random up to the initial population
    while none of our population are solutions {
        generate a new population by picking elements from existing and randomly permuting them
        existing elements are more likely to be chosen if they have high scores
    }
    */

    let mut population = Vec::new();
    for _ in 0..population_size {
        population.push(make_new());
    }

    loop {
        // check each existing member of the population to see if it's a solution, and to score it
        let mut with_scores = Vec::new();
        let mut total_score = 0;
        let mut best_score = 0;
        let mut best_element = None;
        for p in population.iter() {
            // we're done
            if is_a_solution(p) {
                return Some(p.clone());
            }
            // score it
            let s = score(p);
            with_scores.push((s, p));
            total_score += s;
            if s > best_score {
                best_score = s;
                best_element = Some(p);
            }
        }
        println!("TODO JEFF current best score = {}", best_score);

        let mut new_population = Vec::new();
        if let Some(p) = best_element {
            new_population.push(p.clone())
        };
        for _ in 0..population_size {
            let mut choice = rand::random::<u32>() % total_score;
            for (s, t) in with_scores.iter() {
                if choice <= *s {
                    let mut next = (*t).clone();
                    permute(&mut next);
                    new_population.push(next);
                    break;
                }
                choice -= s;
            }
        }
        // println!("TODO next generation has {} elements", new_population.len());
        population = new_population;
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
                .map(|v| Sequence::new(v))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(sequences
        .iter()
        .filter(|sequence| !sequence.is_valid(&rules))
        .map(|sequence| Sequence::new_with_numbers_3(&sequence.0, &rules))
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
        assert_eq!(do_it("day05.txt").unwrap(), 0);
    }
}
