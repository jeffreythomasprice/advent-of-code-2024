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

#[allow(dead_code)]
fn do_it(path: &str) -> Result<usize> {
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

    // graph node name to graph node index
    let mut name_to_index = HashMap::new();
    // graph node index to graph node name
    let mut index_to_name = Vec::new();
    // graph node index to set of neighboring graph node indices
    let mut connections = Vec::new();
    let mut next = 0;
    for line in file_contents.iter() {
        let parts = line.split("-").collect::<Vec<_>>();
        if parts.len() != 2 {
            Err(format!("expected exactly one - in input, got {}", line))?;
        }

        let (a, b) = (parts[0], parts[1]);

        let a_i = *name_to_index.entry(a).or_insert_with(|| {
            let result = next;
            connections.push(HashSet::new());
            index_to_name.push(a.to_string());
            next += 1;
            result
        });
        let b_i = *name_to_index.entry(b).or_insert_with(|| {
            let result = next;
            connections.push(HashSet::new());
            index_to_name.push(b.to_string());
            next += 1;
            result
        });

        connections[a_i].insert(b_i);
        connections[b_i].insert(a_i);
    }

    // turn sets into vectors, so we can sort them
    let connections = connections
        .into_iter()
        .map(|c| {
            let mut result = c.into_iter().collect::<Vec<_>>();
            result.sort();
            result
        })
        .collect::<Vec<_>>();

    // iterate over all triplets
    // start with all indices
    let mut triplets = HashSet::new();
    for i1 in 0..connections.len() {
        // iterate over all pairs of neighbors of i1, such that they go in order i1 < i2 < i3
        let connections_i1 = &connections[i1];
        for j2 in 0..connections_i1.len() {
            for j3 in (j2 + 1)..connections_i1.len() {
                if j2 == j3 {
                    continue;
                }
                let i2 = connections_i1[j2];
                let i3 = connections_i1[j3];
                let name1 = &index_to_name[i1];
                let name2 = &index_to_name[i2];
                let name3 = &index_to_name[i3];
                if (name1.starts_with('t') || name2.starts_with('t') || name3.starts_with('t')) && connections[i2].contains(&i3) {
                    let mut sorted = [name1, name2, name3];
                    sorted.sort();
                    triplets.insert(sorted);
                }
            }
        }
    }
    Ok(triplets.len())
}

#[cfg(test)]
mod tests {
    use super::do_it;

    #[test]
    pub fn test_sample() {
        assert_eq!(do_it("day23-sample.txt").unwrap(), 7);
    }

    #[test]
    pub fn test_real() {
        assert_eq!(do_it("day23.txt").unwrap(), 1306);
    }
}
