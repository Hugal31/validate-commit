//use std::str::FromStr;

use errors::{CommitValidationError, FormatError, Result};
use {CommitHeader, CommitMsg, CommitType};

pub fn parse_commit_message(message: &str) -> Result<CommitMsg> {
    let lines: Vec<_> = message.lines().collect();

    if lines.get(1).map_or(false, |l| !l.is_empty()) {
        return Err(FormatError::NonEmptySecondLine.with_format_context(lines[1], 1, 0).into());
    }

    Ok(CommitMsg {
        header: parse_commit_header(lines[0])?,
    })
}

fn parse_commit_header(line: &str) -> Result<CommitHeader> {
    let line = if line.starts_with("fixup! ") || line.starts_with("squash! ") {
        &line[line.find(' ').unwrap() + 1..]
    } else {
        line
    };

    let column_pos = line.find(':').ok_or(FormatError::NoColumn)?;
    let (commit_type, scope) = parse_commit_type_and_scope(&line[0..column_pos])?;
    let commit_type: CommitType = commit_type.parse()
        .map_err(|e: FormatError| e.with_format_context(line, 0, 0))?;

    match line.get(column_pos + 1..column_pos + 2) {
        Some(" ") => (),
        _ => {
            return Err(FormatError::MisplacedWhitespace.with_format_context(line, 0, column_pos + 1).into())
        }
    }

    let subject_pos = column_pos + 2;
    let subject = &line[subject_pos..];
    if subject.is_empty() {
        return Err(FormatError::EmptyCommitSubject.into());
    }

    // Check if the subject is trimmed
    if subject != subject.trim() {
        return Err(FormatError::MisplacedWhitespace.into());
    }

    Ok(CommitHeader {
        commit_type,
        scope,
        subject,
    })
}

fn parse_commit_type_and_scope(commit_type_and_scope: &str) -> Result<(&str, Option<&str>)> {
    if commit_type_and_scope.is_empty() {
        return Err(FormatError::EmptyCommitType.into());
    }

    let first_char = commit_type_and_scope.chars().next().unwrap();
    if first_char.is_whitespace() {
        return Err(FormatError::MisplacedWhitespace.with_format_context(commit_type_and_scope, 0, 0).into());
    }

    let last_char = commit_type_and_scope.chars().last().unwrap();
    if last_char.is_whitespace() {
        return Err(FormatError::MisplacedWhitespace
            .with_format_context(commit_type_and_scope, 0, commit_type_and_scope.len() - 1)
            .into());
    }

    Ok(if last_char == ')' {
        let opening_parenthesis: usize = commit_type_and_scope
            .find('(')
            .ok_or(CommitValidationError::from(FormatError::MissingParenthesis))?;
        (
            &commit_type_and_scope[..opening_parenthesis],
            Some(&commit_type_and_scope[opening_parenthesis + 1..commit_type_and_scope.len() - 1]),
        )
    } else {
        (commit_type_and_scope, None)
    })
}

#[cfg(test)]
mod tests {
    use super::parse_commit_message;
    use CommitType;

    #[test]
    fn test_parse_header() {
        assert!(parse_commit_message("refactor: add commit parsing").is_ok());

        let commit_msg = parse_commit_message("refactor(scope): add commit parsing");
        assert!(commit_msg.is_ok());

        let commit_msg = commit_msg.unwrap();
        assert_eq!(commit_msg.header.subject, "add commit parsing");
        assert_eq!(commit_msg.header.commit_type, CommitType::Refactor);
        assert_eq!(commit_msg.header.scope, Some("scope"));
    }

    #[test]
    fn test_discard_invalid_commit_type() {
        assert!(parse_commit_message("feet: add feeture").is_err());
    }

    #[test]
    fn discard_not_trimmed_subject() {
        assert!(parse_commit_message("feat: add commit message validation ").is_err());
        assert!(parse_commit_message("feat:  add commit message validation").is_err());
    }

    #[test]
    fn test_second_line_empty() {
        assert!(
            parse_commit_message(
                "feat: add commit message validation
- Validate commit type
- Validate subject"
            ).is_err()
        );
    }

    #[test]
    fn test_fixup_or_squash() {
        assert!(parse_commit_message("fixup! feat: add commit message validation").is_ok());
        assert!(parse_commit_message("squash! feat: add commit message validation").is_ok());
    }
}
