#[macro_use]
extern crate failure;

mod parse;

use std::{fs::File, io::Read, str::FromStr};

use failure::{Fail, ResultExt};

use parse::parse_commit_message;

pub use errors::*;

pub mod errors {
    use failure::{Context, Fail};
    use std::{fmt, result};

    pub type Result<T> = result::Result<T, CommitValidationError>;

    #[derive(Debug, Fail)]
    pub enum CommitValidationError {
        FormatError(#[cause] Context<FormatErrorKind>),
        IoError(#[cause] Context<IOErrorKind>),
    }

    impl fmt::Display for CommitValidationError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            use CommitValidationError::*;

            match self {
                FormatError(c) => {
                    if let Some(cause) = c.cause() {
                        write!(f, "{}\n{}", c, cause)
                    } else {
                        c.fmt(f)
                    }
                }
                IoError(c) => c.fmt(f),
            }
        }
    }

    impl From<FormatErrorKind> for CommitValidationError {
        fn from(error: FormatErrorKind) -> Self {
            CommitValidationError::FormatError(Context::new(error))
        }
    }

    impl From<Context<FormatErrorKind>> for CommitValidationError {
        fn from(error: Context<FormatErrorKind>) -> Self {
            CommitValidationError::FormatError(error)
        }
    }

    impl From<Context<IOErrorKind>> for CommitValidationError {
        fn from(error: Context<IOErrorKind>) -> Self {
            CommitValidationError::IoError(error)
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
    pub enum IOErrorKind {
        #[fail(display = "Error while opening commit file")]
        OpenFileError,
        #[fail(display = "Error while reading commit file")]
        ReadFileError,
    }

    #[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
    pub enum FormatErrorKind {
        #[fail(display = "First letter must not be capitalized")]
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

    #[derive(Fail, Debug)]
    pub struct FormatError {
        line: String,
        pos: usize,
    }

    impl FormatError {
        pub fn new(line: &str, pos: usize) -> FormatError {
            FormatError {
                line: line.to_owned(),
                pos,
            }
        }
    }

    impl fmt::Display for FormatError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}\n{: >2$}", self.line, '^', self.pos)
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
    type Err = FormatErrorKind;

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
            _ => Err(FormatErrorKind::InvalidCommitType),
        }
    }
}

pub fn validate_commit_file(path: &str) -> Result<()> {
    let mut file = File::open(path).context(IOErrorKind::OpenFileError)?;
    let mut message = String::with_capacity(64);
    file.read_to_string(&mut message)
        .context(IOErrorKind::ReadFileError)?;
    validate_commit_message(&message)
}

pub fn validate_commit_message(input: &str) -> Result<()> {
    if input.starts_with("Merge ") || input.starts_with("WIP") {
        return Ok(());
    }

    let lines: Vec<_> = input.lines().collect();

    let message = parse_commit_message(input)?;

    for line in &lines {
        if line.len() > 100 {
            return Err(FormatError::new(line, 100)
                .context(FormatErrorKind::LineTooLong(100))
                .into());
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
        let pos = Into::<&'static str>::into(message.header.commit_type).len()
            + message.header.scope.map_or(0, |s| s.len() + 2) + 3;
        return Err(FormatError::new(lines[0], pos)
            .context(FormatErrorKind::CapitalizedFirstLetter)
            .into());
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
