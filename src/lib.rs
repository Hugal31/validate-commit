#[macro_use]
extern crate error_chain;

mod parse;

use std::{fs::File, io::Read, str::FromStr};

use parse::parse_commit_message;

pub use errors::*;

pub mod errors {
    error_chain! {
        errors {
            CommitTypeError(t: String) {
                description("invalid commit type")
                display("invalid commit type '{}'", t)
            }
            FormatError(message: String, line: usize, pos: usize) {
                description("format error")
                display("{} at line {} pos {}", message, line, pos)
            }
        }

        foreign_links {
            Io(::std::io::Error);
        }
    }
}

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

impl FromStr for CommitType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
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
            _ => Err(ErrorKind::CommitTypeError(s.to_string()).into()),
        }
    }
}

pub fn validate_commit_file(path: &str) -> Result<()> {
    let mut file = File::open(path)?;
    let mut message = String::with_capacity(64);
    file.read_to_string(&mut message)?;
    validate_commit_message(&message)
}

pub fn validate_commit_message(input: &str) -> Result<()> {
    if input.starts_with("Merge ") || input.starts_with("WIP") {
        return Ok(());
    }

    let lines: Vec<_> = input.lines().collect();

    let message = parse_commit_message(input).map_err(|e| prettify_format_error(e, &lines))?;

    for (idx, line) in lines.iter().enumerate() {
        if line.len() > 100 {
            return Err(prettify_format_error(
                ErrorKind::FormatError(
                    "lines must not be longuer than 100 characters".to_string(),
                    idx,
                    100,
                ).into(),
                &lines,
            ));
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
        return Err("first letter of subject must not be capitalized".into());
    }

    Ok(())
}

fn prettify_format_error(error: Error, lines: &[&str]) -> Error {
    match error.0 {
        ErrorKind::FormatError(ref message, line_nb, pos) => format!(
            "line {1}: {2}\n{3}\n{4: >0$}",
            pos, line_nb, message, lines[line_nb], '^'
        ).into(),
        _ => error,
    }
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
