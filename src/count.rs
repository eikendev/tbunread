use regex::Regex;
use std::collections::HashMap;
use std::path::{Component, Path};

pub struct CountResult {
    list: Vec<usize>,
}

impl CountResult {
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

pub fn count_all(watch_dir: &Path) -> CountResult {
    let path = watch_dir.join("**").join("*.msf").display().to_string();
    let counts: Vec<(String, usize)> = glob::glob(&path)
        .expect("failed to walk directory")
        .map(|file| match file {
            Ok(p) => {
                let account = parse_account(&p, watch_dir);
                let re = Regex::new(r"\(\^A2=(?P<count>\d+)\)").unwrap();
                let contents = std::fs::read_to_string(p).expect("could not read file");
                let count = re.captures_iter(&contents).map(|cap| cap["count"].to_string()).last();

                count.map(|c| (account, c.parse().ok().unwrap_or(0)))
            }
            Err(_) => None,
        })
        .filter(|c| c.is_some())
        .map(|c| c.unwrap())
        .collect();

    let mut accounts: HashMap<String, Vec<usize>> = HashMap::new();

    for (account, count) in counts {
        if let Some(bucket) = accounts.get_mut(&account) {
            bucket.push(count);
        } else {
            let bucket = vec![count];
            accounts.insert(account, bucket);
        }
    }

    let mut accounts: Vec<(String, usize)> = accounts.into_iter().map(|x| (x.0, x.1.iter().sum())).collect();
    accounts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    CountResult {
        list: accounts.into_iter().map(|x| x.1).collect(),
    }
}
