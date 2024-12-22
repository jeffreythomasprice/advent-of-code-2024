use std::{
    collections::{HashMap, VecDeque},
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

fn multiply_step(input: u64, arg: u64) -> u64 {
    let next = input * arg;
    (input ^ next) % 16777216
}

fn divide_step(input: u64, arg: u64) -> u64 {
    let next = input / arg;
    (input ^ next) % 16777216
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

    let input = file_contents
        .iter()
        .map(|line| Ok(line.parse::<u64>()?))
        .collect::<Result<Vec<_>>>()?;

    // TODO testing
    // let input = vec![123u64];

    let mut last_sequence = VecDeque::new();
    // first index = sequence of last deltas
    // second index = which first secret number
    // value = max ones digit
    let mut best: HashMap<Vec<i64>, HashMap<u64, u64>> = HashMap::new();
    for number in input {
        let mut current = number;
        // println!("TODO start = {}", current);
        // TODO testing
        // for _ in 0..10 {
        for _ in 0..2000 {
            let next = multiply_step(current, 64);
            let next = divide_step(next, 32);
            let next = multiply_step(next, 2048);

            let next_ones = next % 10;
            let current_ones = current % 10;
            let delta = (next_ones as i64) - (current_ones as i64);

            current = next;

            // println!("current_ones = {}", current_ones);
            // println!("last = {:?}", last_sequence);

            // best = Some(match best {
            //     Some((best_value, best_sequence)) => {
            //         if best_value > current_ones {
            //             (best_value, best_sequence)
            //         } else {
            //             (current_ones, last_sequence.iter().map(|x| *x).collect::<Vec<_>>())
            //         }
            //     }
            //     None => (current_ones, last_sequence.iter().map(|x| *x).collect::<Vec<_>>()),
            // });

            {
                let last_sequence = last_sequence.iter().map(|x| *x).collect::<Vec<_>>();
                best.entry(last_sequence)
                    .or_insert(HashMap::new())
                    .entry(number)
                    .and_modify(|existing| *existing = (*existing).max(current_ones))
                    .or_insert(current_ones);
            }

            last_sequence.push_back(delta);
            while last_sequence.len() > 4 {
                last_sequence.pop_front();
            }
        }

        // println!("TODO best = {:?}", best);
    }

    // println!("TODO best = {:?}", best);
    // for (seq, x) in best.iter() {
    //     let value: u64 = x.values().sum();
    //     println!("TODO seq={:?}, sum={}", seq, value);
    // }
    {
        let mut best = best
            .iter()
            .map(|(seq, x)| {
                let sum = x.values().sum::<u64>();
                (seq, sum)
            })
            .collect::<Vec<_>>();
        best.sort_by(|(_, a), (_, b)| a.cmp(b));
        for (seq, sum) in best {
            println!("TODO seq={:?}, sum={}", seq, sum);
        }
    }

    Ok(best.values().map(|x| x.values().sum()).max().unwrap())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day22b-sample.txt").unwrap(), 23);
    }

    #[test]
    pub fn test_real() {
        // 2002, too high
        assert_eq!(do_it("day22.txt").unwrap(), 0);
    }
}
