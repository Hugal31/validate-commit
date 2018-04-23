#[macro_use]
extern crate failure;

mod parse;

use std::{fs::File, io::Read, str::FromStr};

use parse::parse_commit_message;

pub use errors::*;

pub mod errors {
    use std::{io, fmt, result};

    pub type Result<T> = result::Result<T, CommitValidationError>;

    #[derive(Debug, Fail)]
    pub enum CommitValidationError {
        #[fail(display = "{}", _0)]
        Format(#[cause] FormatError),
        #[fail(display = "{}", _0)]
        FormatContext(#[cause] FormatErrorContext),
        #[fail(display = "{}", _0)]
        Io(#[cause] io::Error)
    }

    impl From<io::Error> for CommitValidationError {
        fn from(error: io::Error) -> Self {
            CommitValidationError::Io(error)
        }
    }

    impl From<FormatError> for CommitValidationError {
        fn from(error: FormatError) -> Self {
            CommitValidationError::Format(error)
        }
    }

    impl From<FormatErrorContext> for CommitValidationError {
        fn from(error: FormatErrorContext) -> Self {
            CommitValidationError::FormatContext(error)
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
    pub enum FormatError {
        #[fail(display = "First letter must no be capitalized")]
        CapitalizedFirstLetter,
        #[fail(display = "Empty commit subject")]
        EmptyCommitSubject,
        #[fail(display = "Empty commit type")]
        EmptyCommitType,
        #[fail(display = "Invalid commit type")]
        InvalidCommitType,
        #[fail(display = "Line must not be longer than {} characters", _0)]
        LineTooLong(usize),
        #[fail(display = "Missing parenthesis")]
        MissingParenthesis,
        #[fail(display = "Misplaced whitespace")]
        MisplacedWhitespace,
        #[fail(display = "First line must contain a column")]
        NoColumn,
        #[fail(display = "Second line must be empty")]
        NonEmptySecondLine,
    }

    impl FormatError {
        pub fn with_format_context(self, line: &str, line_nb: usize, pos: usize) -> FormatErrorContext {
            FormatErrorContext{
                error: self,
                line: line.to_string(),
                line_nb,
                pos
            }
        }
    }

    #[derive(Debug, Fail)]
    pub struct FormatErrorContext {
        #[cause]
        pub error: FormatError,
        pub line: String,
        pub line_nb: usize,
        pub pos: usize,
    }

    impl fmt::Display for FormatErrorContext {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "line {}: {}\n{}\n{: >4$}", self.line_nb, self.error, self.line, '^', self.pos)
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
            _ => Err(FormatError::InvalidCommitType),
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

    let message = parse_commit_message(input)?;

    for (idx, line) in lines.iter().enumerate() {
        if line.len() > 100 {
            return Err(FormatError::LineTooLong(100).with_format_context(line, idx, 100).into());
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
        return Err(FormatError::CapitalizedFirstLetter.with_format_context(lines[0], 0, Into::<&'static str>::into(message.header.commit_type).len()
            + message.header.scope.map_or(0, |s| s.len() + 2)
            + 2).into());
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
