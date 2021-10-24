use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Component, Path};

pub struct Count {
    list: Vec<usize>,
}

impl Count {
    pub fn display(&self) -> String {
        let output: Vec<String> = self.list.iter().map(|x| x.to_string()).collect();
        output.join(" ")
    }
}

fn parse_account(path: &Path, base: &Path) -> String {
    let relative = path.strip_prefix(base).unwrap();
    let component = relative.components().next().unwrap();

    match component {
        Component::Normal(c) => c.to_str().unwrap().to_string(),
        _ => panic!("failed converting path component to string"),
    }
}

pub fn count_all(watch_dir: &Path) -> Result<Count> {
    let path = watch_dir.join("**").join("*.msf").display().to_string();
    let counts: Vec<(String, usize)> = glob::glob(&path)
        .context("Unable to walk directory")?
        .map(|file| match file {
            Ok(p) => {
                let account = parse_account(&p, watch_dir);
                let re = Regex::new(r"\(\^A2=(?P<count>[[:xdigit:]]+)\)").unwrap();
                let contents = std::fs::read_to_string(p).expect("could not read file");
                let count = re.captures_iter(&contents).map(|cap| cap["count"].to_string()).last();

                // If a None was returned, some accounts names might not be caught later on.
                match count {
                    Some(string) => {
                        let count = usize::from_str_radix(&string, 16).unwrap_or(0);
                        Some((account, count))
                    }
                    None => Some((account, 0)),
                }
            }
            Err(_) => None,
        })
        .filter(|c| c.is_some())
        .flatten()
        .collect();

    let mut accounts: HashMap<String, usize> = HashMap::new();

    for (account, count) in counts {
        let current = accounts.entry(account).or_insert(0);
        *current += count;
    }

    let mut accounts: Vec<(String, usize)> = accounts.into_iter().collect();
    accounts.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(Count {
        list: accounts.into_iter().map(|x| x.1).collect(),
    })
}
