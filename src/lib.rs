#[macro_use]
extern crate error_chain;

use std::{
    fs::File,
    io::Read,
};

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

pub fn validate_commit_file(path: &str) -> Result<()> {
    let mut file = File::open(path)?;
    let mut message = String::with_capacity(64);
    file.read_to_string(&mut message)?;
    validate_commit_message(&message)
}

pub fn validate_commit_message(message: &str) -> Result<()> {
    let mut lines = message.lines();

    let first_line = lines.next().unwrap();
    validate_first_line(first_line)
}

const COMMIT_TYPES: [&str; 8] = [
    "feat",
    "fix",
    "docs",
    "style",
    "refactor",
    "perf",
    "test",
    "chore",
];

fn validate_first_line(line: &str) -> Result<()> {
    let column_pos = line.find(':').ok_or("first line must contain a column")?;
    let commit_type = &line[0..column_pos];

    // Check the commit type
    if !COMMIT_TYPES.contains(&commit_type) {
        return Err(ErrorKind::CommitTypeError(commit_type.to_string()).into());
    }

    // Check if the column is followed by a space
    match line.get(column_pos + 1..column_pos + 2) {
        Some(" ") => (),
        _         => return Err(ErrorKind::FormatError("the column must be followed by a space".to_string(),
                                                       1,
                                                       column_pos + 1).into()),
    }

    // Check if the commit message is not empty
    let subject_pos = column_pos + 2;
    let subject = &line[subject_pos..];
    if subject.is_empty() {
        return Err("empty commit subject".into());
    }

    // Check if the subject is trimmed
    if subject != subject.trim() {
        return Err("subject is not trimmed".into());
    }

    // Check if the first letter is not capitalized
    if subject.chars().next().unwrap().is_uppercase() {
        return Err(ErrorKind::FormatError("first letter of subject must not be capitalized".to_string(),
                                          1,
                                          subject_pos).into());
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
    fn discard_not_trimmed_subject() {
        assert!(validate_commit_message("feat: add commit message validation ").is_err());
        assert!(validate_commit_message("feat:  add commit message validation").is_err());
    }
}
