use gosub_shared::bytes::Position;
use gosub_shared::types::ParseError;

/// Possible parser error enumerated
pub enum ParserError {
    AbruptDoctypePublicIdentifier,
    AbruptDoctypeSystemIdentifier,
    AbruptClosingOfEmptyComment,
    AbsenceOfDigitsInNumericCharacterReference,
    CdataInHtmlContent,
    CharacterReferenceOutsideUnicodeRange,
    ControlCharacterInInputStream,
    ControlCharacterReference,
    EndTagWithAttributes,
    DuplicateAttribute,
    EndTagWithTrailingSolidus,
    EofBeforeTagName,
    EofInCdata,
    EofInComment,
    EofInDoctype,
    EofInScriptHtmlCommentLikeText,
    EofInTag,
    IncorrectlyClosedComment,
    IncorrectlyOpenedComment,
    InvalidCharacterSequenceAfterDoctypeName,
    InvalidFirstCharacterOfTagName,
    MissingAttributeValue,
    MissingDoctypeName,
    MissingDoctypePublicIdentifier,
    MissingDoctypeSystemIdentifier,
    MissingEndTagName,
    MissingQuoteBeforeDoctypePublicIdentifier,
    MissingQuoteBeforeDoctypeSystemIdentifier,
    MissingSemicolonAfterCharacterReference,
    MissingWhitespaceAfterDoctypePublicKeyword,
    MissingWhitespaceAfterDoctypeSystemKeyword,
    MissingWhitespaceBeforeDoctypeName,
    MissingWhitespaceBetweenAttributes,
    MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers,
    NestedComment,
    NoncharacterCharacterReference,
    NoncharacterInInputStream,
    NonVoidHtmlElementStartTagWithTrailingSolidus,
    NullCharacterReference,
    SurrogateCharacterReference,
    SurrogateInInputStream,
    UnexpectedCharacterAfterDoctypeSystemIdentifier,
    UnexpectedCharacterInAttributeName,
    UnexpectedCharacterInUnquotedAttributeValue,
    UnexpectedEqualsSignBeforeAttributeName,
    UnexpectedNullCharacter,
    UnexpectedQuestionMarkInsteadOfTagName,
    UnexpectedSolidusInTag,
    UnknownNamedCharacterReference,

    ExpectedDocTypeButGotChars,
    ExpectedDocTypeButGotStartTag,
    ExpectedDocTypeButGotEndTag,
}

impl ParserError {
    /// Parser errors as string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AbruptDoctypePublicIdentifier => "abrupt-doctype-public-identifier",
            Self::AbruptDoctypeSystemIdentifier => "abrupt-doctype-system-identifier",
            Self::AbsenceOfDigitsInNumericCharacterReference => {
                "absence-of-digits-in-numeric-character-reference"
            }
            Self::CdataInHtmlContent => "cdata-in-html-content",
            Self::CharacterReferenceOutsideUnicodeRange => {
                "character-reference-outside-unicode-range"
            }
            Self::ControlCharacterInInputStream => "control-character-in-input-stream",
            Self::ControlCharacterReference => "control-character-reference",
            Self::EndTagWithAttributes => "end-tag-with-attributes",
            Self::DuplicateAttribute => "duplicate-attribute",
            Self::EndTagWithTrailingSolidus => "end-tag-with-trailing-solidus",
            Self::EofBeforeTagName => "eof-before-tag-name",
            Self::EofInCdata => "eof-in-cdata",
            Self::EofInComment => "eof-in-comment",
            Self::EofInDoctype => "eof-in-doctype",
            Self::EofInScriptHtmlCommentLikeText => "eof-in-script-html-comment-like-text",
            Self::EofInTag => "eof-in-tag",
            Self::IncorrectlyClosedComment => "incorrectly-closed-comment",
            Self::IncorrectlyOpenedComment => "incorrectly-opened-comment",
            Self::InvalidCharacterSequenceAfterDoctypeName => {
                "invalid-character-sequence-after-doctype-name"
            }
            Self::InvalidFirstCharacterOfTagName => "invalid-first-character-of-tag-name",
            Self::MissingAttributeValue => "missing-attribute-value",
            Self::MissingDoctypeName => "missing-doctype-name",
            Self::MissingDoctypePublicIdentifier => "missing-doctype-public-identifier",
            Self::MissingDoctypeSystemIdentifier => "missing-doctype-system-identifier",
            Self::MissingEndTagName => "missing-end-tag-name",
            Self::MissingQuoteBeforeDoctypePublicIdentifier => {
                "missing-quote-before-doctype-public-identifier"
            }
            Self::MissingQuoteBeforeDoctypeSystemIdentifier => {
                "missing-quote-before-doctype-system-identifier"
            }
            Self::MissingSemicolonAfterCharacterReference => {
                "missing-semicolon-after-character-reference"
            }
            Self::MissingWhitespaceAfterDoctypePublicKeyword => {
                "missing-whitespace-after-doctype-public-keyword"
            }
            Self::MissingWhitespaceAfterDoctypeSystemKeyword => {
                "missing-whitespace-after-doctype-system-keyword"
            }
            Self::MissingWhitespaceBeforeDoctypeName => {
                "missing-whitespace-before-doctype-name"
            }
            Self::MissingWhitespaceBetweenAttributes => {
                "missing-whitespace-between-attributes"
            }
            Self::MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers => {
                "missing-whitespace-between-doctype-public-and-system-identifiers"
            }
            Self::NestedComment => "nested-comment",
            Self::NoncharacterCharacterReference => "noncharacter-character-reference",
            Self::NoncharacterInInputStream => "noncharacter-in-input-stream",
            Self::NonVoidHtmlElementStartTagWithTrailingSolidus => {
                "non-void-html-element-start-tag-with-trailing-solidus"
            }
            Self::NullCharacterReference => "null-character-reference",
            Self::SurrogateCharacterReference => "surrogate-character-reference",
            Self::SurrogateInInputStream => "surrogate-in-input-stream",
            Self::UnexpectedCharacterAfterDoctypeSystemIdentifier => {
                "unexpected-character-after-doctype-system-identifier"
            }
            Self::UnexpectedCharacterInAttributeName => {
                "unexpected-character-in-attribute-name"
            }
            Self::UnexpectedCharacterInUnquotedAttributeValue => {
                "unexpected-character-in-unquoted-attribute-value"
            }
            Self::UnexpectedEqualsSignBeforeAttributeName => {
                "unexpected-equals-sign-before-attribute-name"
            }
            Self::UnexpectedNullCharacter => "unexpected-null-character",
            Self::UnexpectedQuestionMarkInsteadOfTagName => {
                "unexpected-question-mark-instead-of-tag-name"
            }
            Self::UnexpectedSolidusInTag => "unexpected-solidus-in-tag",
            Self::UnknownNamedCharacterReference => "unknown-named-character-reference",
            Self::AbruptClosingOfEmptyComment => "abrupt-closing-of-empty-comment",

            Self::ExpectedDocTypeButGotChars => "expected-doctype-but-got-chars",
            Self::ExpectedDocTypeButGotStartTag => "expected-doctype-but-got-start-tag",
            Self::ExpectedDocTypeButGotEndTag => "expected-doctype-but-got-end-tag",
        }
    }
}

#[derive(Clone)]
pub struct ErrorLogger {
    /// List of errors that occurred during parsing
    errors: Vec<ParseError>,
}

impl Default for ErrorLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorLogger {
    /// Creates a new error logger
    #[must_use]
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Returns a cloned instance of the errors
    #[must_use]
    pub fn get_errors(&self) -> Vec<ParseError> {
        self.errors.clone()
    }

    /// Adds a new error to the error logger
    pub fn add_error(&mut self, pos: Position, message: &str) {
        // Check if the error already exists, if so, don't add it again
        for err in &self.errors {
            if err.line == pos.line && err.col == pos.col && err.message == *message {
                return;
            }
        }

        self.errors.push(ParseError {
            line: pos.line,
            col: pos.col,
            offset: pos.offset,
            message: message.to_string(),
        });

        // println!("Parse error ({}/{}): {}", pos.line, pos.col, message);
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_error_logger() {
        let mut logger = ErrorLogger::new();

        logger.add_error(Position::new(1, 1, 0), "test");
        logger.add_error(Position::new(1, 1, 0), "test");
        logger.add_error(Position::new(1, 1, 0), "test");
        logger.add_error(Position::new(1, 1, 0), "test");
        logger.add_error(Position::new(1, 1, 0), "test");

        assert_eq!(logger.get_errors().len(), 1);
    }

    #[test]
    fn test_error_logger2() {
        let mut logger = ErrorLogger::new();

        logger.add_error(Position::new(1, 1, 0), "test");
        logger.add_error(Position::new(1, 2, 0), "test");
        logger.add_error(Position::new(1, 3, 0), "test");
        logger.add_error(Position::new(1, 4, 0), "test");
        logger.add_error(Position::new(1, 5, 0), "test");

        assert_eq!(logger.get_errors().len(), 5);
    }

    #[test]
    fn test_error_logger3() {
        let mut logger = ErrorLogger::new();

        logger.add_error(Position::new(1, 1, 0), "test");
        logger.add_error(Position::new(1, 2, 0), "test");
        logger.add_error(Position::new(1, 3, 0), "test");
        logger.add_error(Position::new(1, 4, 0), "test");
        logger.add_error(Position::new(1, 5, 0), "test");
        logger.add_error(Position::new(1, 5, 0), "test");
        logger.add_error(Position::new(1, 5, 0), "test");
        logger.add_error(Position::new(1, 5, 0), "test");
        logger.add_error(Position::new(1, 5, 0), "test");

        assert_eq!(logger.get_errors().len(), 5);
    }

    #[test]
    fn test_error_logger4() {
        let mut logger = ErrorLogger::new();

        logger.add_error(Position::new(0, 1, 1), "test");
        logger.add_error(Position::new(0, 1, 2), "test");
        logger.add_error(Position::new(0, 1, 3), "test");
        logger.add_error(Position::new(0, 1, 4), "test");
        logger.add_error(Position::new(0, 1, 5), "test");
        logger.add_error(Position::new(0, 1, 5), "test");
        logger.add_error(Position::new(0, 1, 5), "test");
        logger.add_error(Position::new(0, 1, 5), "test");
        logger.add_error(Position::new(0, 1, 5), "test");
        logger.add_error(Position::new(0, 2, 1), "test");
        logger.add_error(Position::new(0, 2, 2), "test");
        logger.add_error(Position::new(0, 2, 3), "test");
        logger.add_error(Position::new(0, 2, 4), "test");
        logger.add_error(Position::new(0, 2, 5), "test");
        logger.add_error(Position::new(0, 2, 5), "test");
        logger.add_error(Position::new(0, 2, 5), "test");
        logger.add_error(Position::new(0, 2, 5), "test");
        logger.add_error(Position::new(0, 2, 5), "test");

        assert_eq!(logger.get_errors().len(), 10);
    }
}
