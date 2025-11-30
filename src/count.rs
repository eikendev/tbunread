use anyhow::{Context, Result, bail};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path};

lazy_static! {
    static ref COUNT_REGEX: Regex = Regex::new(r"\(\^A2=(?P<count>[[:xdigit:]]+)\)").expect("valid Thunderbird regex");
}

pub struct Count {
    list: Vec<usize>,
}

impl Count {
    pub fn display(&self) -> String {
        let output: Vec<String> = self.list.iter().map(|x| x.to_string()).collect();
        output.join(" ")
    }
}

fn parse_account(path: &Path, base: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(base)
        .context("failed to strip watch directory prefix")?;

    let component = relative.components().next().context("path missing account component")?;

    match component {
        Component::Normal(c) => c
            .to_str()
            .map(|s| s.to_string())
            .context("failed converting path component to string"),
        _ => bail!("unsupported path component {component:?}"),
    }
}

pub fn count_all(watch_dir: &Path) -> Result<Count> {
    let pattern = watch_dir.join("**").join("*.msf");
    let pattern = pattern
        .to_str()
        .context("watch directory contains non-UTF-8 characters")?
        .to_string();

    let mut accounts: HashMap<String, usize> = HashMap::new();

    for entry in glob::glob(&pattern).context("Unable to walk directory")? {
        let path = entry?;
        let account = parse_account(&path, watch_dir)?;
        let contents = fs::read_to_string(&path).with_context(|| format!("could not read {}", path.display()))?;
        let count = COUNT_REGEX
            .captures_iter(&contents)
            .filter_map(|cap| usize::from_str_radix(&cap["count"], 16).ok())
            .last()
            .unwrap_or(0);
        *accounts.entry(account).or_default() += count;
    }

    let mut accounts: Vec<(String, usize)> = accounts.into_iter().collect();
    accounts.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(Count {
        list: accounts.into_iter().map(|x| x.1).collect(),
    })
}
