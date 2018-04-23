use std::fmt;

use failure::{Backtrace, Context, Fail};

#[derive(Debug, Fail)]
pub enum CommitValidationError {
    #[fail(display = "{}", _0)]
    Format(#[cause] FormatError),
    #[fail(display = "{}", _0)]
    Io(#[cause] IOError),
}

impl From<FormatError> for CommitValidationError {
    fn from(error: FormatError) -> Self {
        CommitValidationError::Format(error)
    }
}

impl From<IOError> for CommitValidationError {
    fn from(error: IOError) -> Self {
        CommitValidationError::Io(error)
    }
}

#[derive(Debug)]
pub struct IOError {
    inner: Context<IOErrorKind>,
}

impl Fail for IOError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for IOError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl From<IOErrorKind> for IOError {
    fn from(c: IOErrorKind) -> Self {
        IOError {
            inner: Context::new(c),
        }
    }
}

impl From<Context<IOErrorKind>> for IOError {
    fn from(c: Context<IOErrorKind>) -> Self {
        IOError { inner: c }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum IOErrorKind {
    #[fail(display = "Error while opening commit file")]
    OpenFileError,
    #[fail(display = "Error while reading commit file")]
    ReadFileError,
}

#[derive(Debug, Fail)]
pub struct FormatError {
    #[cause]
    pub kind: FormatErrorKind,
    location: Option<Span>,
}

impl FormatError {
    pub(crate) fn with_span(kind: FormatErrorKind, line: &str, pos: usize) -> FormatError {
        FormatError {
            kind,
            location: Some(Span::new(line, pos)),
        }
    }

    pub(crate) fn at(self, line: &str, pos: usize) -> FormatError {
        FormatError::with_span(self.kind, line, pos)
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref location) = &self.location {
            write!(f, "{}\n{}", self.kind, location)
        } else {
            self.kind.fmt(f)
        }
    }
}

impl From<FormatErrorKind> for FormatError {
    fn from(kind: FormatErrorKind) -> Self {
        FormatError {
            kind,
            location: None,
        }
    }
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

impl FormatErrorKind {
    pub(crate) fn at(self, line: &str, pos: usize) -> FormatError {
        FormatError::with_span(self, line, pos)
    }
}

#[derive(Debug)]
struct Span {
    line: String,
    pos: usize,
}

impl Span {
    pub fn new(line: &str, pos: usize) -> Span {
        Span {
            line: line.to_owned(),
            pos,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n{: >2$}", self.line, '^', self.pos)
    }
}
