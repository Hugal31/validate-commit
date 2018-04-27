#[macro_use]
extern crate failure;

mod parse;

pub mod errors;

use std::{fs::File, io::Read, str::FromStr};

use failure::ResultExt;

use parse::parse_commit_message;

pub use errors::*;

#[derive(Debug, PartialEq)]
pub struct CommitMsg<'a> {
    pub header: CommitHeader<'a>,
}

#[derive(Debug, PartialEq)]
pub struct CommitHeader<'a> {
    pub commit_type: CommitType,
    pub scope: Option<&'a str>,
    pub subject: &'a str,
}

#[derive(Debug, PartialEq)]
pub enum CommitType {
    Feat,
    Fix,
    Docs,
    Style,
    Refactor,
    Perf,
    Test,
    Chore,
}

impl From<CommitType> for &'static str {
    fn from(t: CommitType) -> Self {
        use CommitType::*;

        match t {
            Feat => "feat",
            Fix => "fix",
            Docs => "docx",
            Style => "style",
            Refactor => "refactor",
            Perf => "perf",
            Test => "test",
            Chore => "chore",
        }
    }
}

impl FromStr for CommitType {
    type Err = FormatError;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        use CommitType::*;

        match s {
            "feat" => Ok(Feat),
            "fix" => Ok(Fix),
            "docs" => Ok(Docs),
            "style" => Ok(Style),
            "refactor" => Ok(Refactor),
            "perf" => Ok(Perf),
            "test" => Ok(Test),
            "chore" => Ok(Chore),
            _ => Err(FormatErrorKind::InvalidCommitType.into()),
        }
    }
}

pub fn validate_commit_file(path: &str) -> Result<(), CommitValidationError> {
    let message = read_commit_file(path)?;
    validate_commit_message(&message).map_err(|e| e.into())
}

fn read_commit_file(path: &str) -> Result<String, IOError> {
    let mut file = File::open(path).context(IOErrorKind::OpenFileError)?;
    let mut message = String::with_capacity(64);
    file.read_to_string(&mut message)
        .context(IOErrorKind::ReadFileError)?;
    Ok(message)
}

pub fn validate_commit_message(input: &str) -> Result<(), FormatError> {
    let lines: Vec<_> = input.lines()
        .filter(|l| !l.starts_with('#'))
        .collect();

    if lines[0].starts_with("Merge ") || lines[0].starts_with("WIP") {
        return Ok(());
    }

    let message = parse_commit_message(&lines)?;

    for line in &lines {
        if line.len() > 100 {
            return Err(FormatErrorKind::LineTooLong(100).at(line, 100));
        }
    }

    // Check if the first letter is not capitalized
    if message
        .header
        .subject
        .chars()
        .next()
        .unwrap()
        .is_uppercase()
    {
        let pos = lines[0].find(message.header.subject).unwrap();
        return Err(FormatErrorKind::CapitalizedFirstLetter.at(lines[0], pos));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_commit_message;

    #[test]
    fn validate_short_messages() {
        assert!(validate_commit_message("feat: add commit message validation").is_ok());
        assert!(validate_commit_message("fix: fix bug in commit message validation").is_ok());
        assert!(validate_commit_message("docs: add README.md").is_ok());
    }

    #[test]
    fn discard_invalid_commit_type() {
        assert!(validate_commit_message("feet: add commit message validation").is_err());
    }

    #[test]
    fn discard_missing_whitespace_before_subject() {
        assert!(validate_commit_message("feat:add commit message validation").is_err());
    }

    #[test]
    fn discard_missing_subject() {
        assert!(validate_commit_message("feat: ").is_err());
    }

    #[test]
    fn discard_capitalized_subject() {
        assert!(validate_commit_message("feat: Add commit message validation").is_err());
    }

    #[test]
    fn discard_too_long_lines() {
        assert!(validate_commit_message("feat: add commit message validation an other sweet features so this commit contains way too much things").is_err());
    }

    #[test]
    fn ignore_wip_and_merge_message() {
        assert!(validate_commit_message("Merge branch develop").is_ok());
        assert!(validate_commit_message("WIP: feat: add feature").is_ok());
    }
}
