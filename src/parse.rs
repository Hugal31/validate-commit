use errors::{FormatError, FormatErrorKind};
use {CommitHeader, CommitMsg, CommitType};

pub fn parse_commit_message(message: &str) -> Result<CommitMsg, FormatError> {
    let lines: Vec<_> = message.lines().collect();

    if lines.get(1).map_or(false, |l| !l.is_empty()) {
        return Err(FormatErrorKind::NonEmptySecondLine.into());
    }

    Ok(CommitMsg {
        header: parse_commit_header(lines[0])?,
    })
}

fn parse_commit_header(line: &str) -> Result<CommitHeader, FormatError> {
    let line = discard_autosquash(line);

    let column_pos = line.find(':').ok_or(FormatErrorKind::NoColumn)?;
    let (commit_type, scope) = parse_commit_type_and_scope(&line[0..column_pos])?;
    let commit_type: CommitType = commit_type.parse().map_err(|e: FormatError| e.at(line, 0))?;

    if line.get(column_pos + 1..column_pos + 2) != Some(" ") {
        return Err(FormatErrorKind::MissingWhitespace.at(line, column_pos + 1));
    }

    let subject_pos = column_pos + 2;
    let subject = &line[subject_pos..];
    if subject.is_empty() {
        return Err(FormatErrorKind::EmptyCommitSubject.into());
    }

    if !is_trimmed(subject) {
        return Err(FormatErrorKind::MisplacedWhitespace.into());
    }

    Ok(CommitHeader {
        commit_type,
        scope,
        subject,
    })
}

/// Return the string whitout `squash! ` or `fixup! `
fn discard_autosquash(line: &str) -> &str {
    if line.starts_with("fixup! ") {
        &line[7..]
    } else if line.starts_with("squash! ") {
        &line[8..]
    } else {
        line
    }
}

fn is_trimmed(s: &str) -> bool {
    s == s.trim()
}

fn parse_commit_type_and_scope(
    commit_type_and_scope: &str,
) -> Result<(&str, Option<&str>), FormatError> {
    if commit_type_and_scope.is_empty() {
        return Err(FormatErrorKind::EmptyCommitType.into());
    }

    let first_char = commit_type_and_scope.chars().next().unwrap();
    if first_char.is_whitespace() {
        return Err(FormatErrorKind::MisplacedWhitespace.at(commit_type_and_scope, 0));
    }

    let last_char = commit_type_and_scope.chars().last().unwrap();
    if last_char.is_whitespace() {
        return Err(FormatErrorKind::MisplacedWhitespace
            .at(commit_type_and_scope, commit_type_and_scope.len() - 1));
    }

    Ok(if last_char == ')' {
        let opening_parenthesis: usize = commit_type_and_scope
            .find('(')
            .ok_or_else(|| FormatError::from(FormatErrorKind::MissingParenthesis))?;
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
    use errors::*;

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
        let res = parse_commit_message("feet: add feeture");
        assert!(res.is_err());
        assert_eq!(FormatErrorKind::InvalidCommitType, res.unwrap_err().kind);
    }

    #[test]
    fn discard_not_trimmed_subject() {
        assert!(parse_commit_message("feat: add commit message validation ").is_err());
        let res = parse_commit_message("feat:  add commit message validation");
        assert!(res.is_err());
        assert_eq!(FormatErrorKind::MisplacedWhitespace, res.unwrap_err().kind);
    }

    #[test]
    fn discard_missing_whitespace() {
        let res = parse_commit_message("feat:add commit message validation");
        assert!(res.is_err());
        assert_eq!(FormatErrorKind::MissingWhitespace, res.unwrap_err().kind);
    }

    #[test]
    fn test_second_line_empty() {
        let res = parse_commit_message(
            "feat: add commit message validation
- Validate commit type
- Validate subject",
        );
        assert!(res.is_err());
        assert_eq!(FormatErrorKind::NonEmptySecondLine, res.unwrap_err().kind);
    }

    #[test]
    fn test_fixup_or_squash() {
        assert!(parse_commit_message("fixup! feat: add commit message validation").is_ok());
        assert!(parse_commit_message("squash! feat: add commit message validation").is_ok());
    }
}
