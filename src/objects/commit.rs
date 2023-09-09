use anyhow::{anyhow, bail, Result};
use std::{
    fmt::{self, Display},
    str,
};

pub struct CommitContents {
    pub raw: String,
    pub tree: String,
    pub parents: Vec<String>,
    pub author: Author,
    pub committer: Option<Author>,
    pub gpgsig: Option<String>,
    pub message: String,
}

impl CommitContents {
    pub fn parse(body: &[u8]) -> Result<Self> {
        let raw = str::from_utf8(body)?;
        let (metadata, message) = raw
            .split_once("\n\n")
            .ok_or(anyhow!("Missing commit message"))?;
        let mut lines = metadata.lines();

        let tree = lines.next().ok_or(anyhow!("Expected tree hash"))?;
        if !tree.starts_with("tree ") {
            bail!("Expected tree hash");
        }
        let tree = tree[5..].to_owned();

        // This will be blank for the initial commits.
        let mut parents = Vec::new();
        let mut line = lines.next().ok_or(anyhow!("Expected parent or author"))?;
        while line.starts_with("parent ") {
            parents.push(line[7..].to_owned());
            line = lines.next().expect("Expected parent or author");
        }

        if !line.starts_with("author ") {
            bail!("Expected author");
        }
        let author = Author::parse(&line[7..])?;

        let committer = match lines
            .next()
            .filter(|line| line.starts_with("committer "))
            .map(|line| Author::parse(&line[10..]))
        {
            Some(Ok(committer)) => Some(committer),
            Some(Err(_)) => bail!("Invalid commit (committer)"),
            None => None,
        };

        let gpgsig = match lines.next() {
            Some(line) => {
                if !line.starts_with("gpgsig") || !line.contains("--BEGIN PGP SIGNATURE--") {
                    bail!("Invalid commit (gpgsig)");
                }
                let mut gpgsig = line[7..].to_string();
                let mut ends = false;
                let err = lines
                    .map(|line| {
                        if !line.starts_with(' ') {
                            return Some("Unexpected lines");
                        }
                        gpgsig.push('\n');
                        gpgsig.push_str(line);
                        let last_line = line.contains("--END PGP SIGNATURE--");
                        ends = ends || last_line;
                        None
                    })
                    .find_map(|e| e);
                if !ends || err.is_some() {
                    bail!("Invalid commit (gpgsig)");
                }
                Some(gpgsig)
            }
            None => None,
        };

        Ok(Self {
            raw: raw.to_owned(),
            tree,
            parents,
            author,
            committer,
            gpgsig,
            message: message.to_owned(),
        })
    }
}

impl Display for CommitContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("tree {}\n", self.tree))?;
        for parent in self.parents.iter() {
            f.write_fmt(format_args!("parent {}\n", parent))?;
        }
        f.write_fmt(format_args!("author {}\n", self.author))?;
        if let Some(committer) = &self.committer {
            f.write_fmt(format_args!("committer {}\n", committer))?;
        }
        if let Some(gpgsig) = &self.gpgsig {
            f.write_fmt(format_args!("gpgsig {}\n", gpgsig))?;
        }
        f.write_fmt(format_args!("\n{}", self.message))
    }
}

pub struct Author {
    pub name: String,
    pub email: String,
    pub timestamp: u32,
    pub timezone: String,
}

impl Author {
    fn parse(line: &str) -> Result<Self> {
        let (name, remaining) = line
            .split_once(" <")
            .ok_or(anyhow!("Invalid author format"))?;
        let (email, remaining) = remaining
            .split_once("> ")
            .ok_or(anyhow!("Invalid author format"))?;
        let (timestamp, timezone) = remaining
            .split_once(' ')
            .ok_or(anyhow!("Invalid author format"))?;
        let timestamp = timestamp
            .parse::<u32>()
            .map_err(|_| anyhow!("Invalid author format (timestamp)"))?;
        let parsed_timezone: i32 = timezone
            .parse()
            .map_err(|_| anyhow!("Invalid author format (timezone)"))?;
        if !(-2400..2400).contains(&parsed_timezone) {
            bail!("Invalid author format (timezone)");
        }
        Ok(Self {
            name: name.to_string(),
            email: email.to_string(),
            timestamp,
            timezone: timezone.to_string(),
        })
    }
}
impl Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{} <{}> {} {}",
            self.name, self.email, self.timestamp, self.timezone
        ))
    }
}
