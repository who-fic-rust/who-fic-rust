//! A delimiter-aware, quote-aware scanner that splits one line of the WHO
//! Simplified Linearization Output format into fields.
//!
//! The file is tab-separated, but individual fields (notably `Title` and
//! `BrowserLink`) use Excel/CSV-style double-quoting with `""` for an
//! embedded quote. A generic CSV parser is the wrong tool here because a
//! comma inside a quoted field must *not* split a field, while a bare tab
//! outside quotes *must*. This module implements exactly that rule, and
//! nothing else: it does not know about the meaning of any column.

use std::iter::Peekable;
use std::str::Chars;

/// A problem found while scanning a line into tab-delimited fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScanErrorKind {
    /// A quoted field's opening `"` was never matched by a closing `"`
    /// before the line ran out.
    UnterminatedQuote,
    /// Characters appeared between a quoted field's closing `"` and the
    /// next tab (or end of line).
    TrailingDataAfterQuotedField,
}

/// Splits `line` into its tab-delimited fields, unescaping any quoted
/// fields along the way. An empty line yields a single empty field, mirroring
/// how an empty bare field is scanned.
pub(crate) fn split_tab_csv_line(line: &str) -> Result<Vec<String>, ScanErrorKind> {
    let mut fields = Vec::new();
    let mut chars = line.chars().peekable();
    loop {
        let field = scan_field(&mut chars)?;
        fields.push(field);
        if chars.peek() == Some(&'\t') {
            chars.next();
        } else {
            break;
        }
    }
    Ok(fields)
}

/// Scans a single field starting at the current position of `chars`,
/// leaving `chars` positioned just before the next tab delimiter (or at the
/// end of input).
fn scan_field(chars: &mut Peekable<Chars<'_>>) -> Result<String, ScanErrorKind> {
    if chars.peek() == Some(&'"') {
        scan_quoted_field(chars)
    } else {
        Ok(scan_bare_field(chars))
    }
}

/// Scans a bare (unquoted) field: everything up to the next tab or end of
/// input, verbatim.
fn scan_bare_field(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut out = String::new();
    while let Some(&c) = chars.peek() {
        if c == '\t' {
            break;
        }
        out.push(c);
        chars.next();
    }
    out
}

/// Scans a quoted field: consumes the opening `"`, unescapes `""` to `"`,
/// and stops at the closing `"`. The closing `"` must be immediately
/// followed by a tab or the end of input.
fn scan_quoted_field(chars: &mut Peekable<Chars<'_>>) -> Result<String, ScanErrorKind> {
    chars.next(); // consume opening quote
    let mut out = String::new();
    loop {
        match chars.next() {
            Some('"') => {
                if chars.peek() == Some(&'"') {
                    out.push('"');
                    chars.next();
                } else {
                    break;
                }
            }
            Some(c) => out.push(c),
            None => return Err(ScanErrorKind::UnterminatedQuote),
        }
    }
    match chars.peek() {
        None | Some('\t') => Ok(out),
        Some(_) => Err(ScanErrorKind::TrailingDataAfterQuotedField),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_bare_fields() {
        assert_eq!(
            split_tab_csv_line("a\tb\tc").unwrap(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn splits_empty_line_as_one_empty_field() {
        assert_eq!(split_tab_csv_line("").unwrap(), vec!["".to_string()]);
    }

    #[test]
    fn handles_empty_fields() {
        assert_eq!(
            split_tab_csv_line("a\t\tc").unwrap(),
            vec!["a".to_string(), "".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn unescapes_quoted_field_with_embedded_quotes() {
        let line = "a\t\"=hyperlink(\"\"https://example\"\",\"\"browser\"\")\"\tc";
        let fields = split_tab_csv_line(line).unwrap();
        assert_eq!(
            fields,
            vec![
                "a".to_string(),
                "=hyperlink(\"https://example\",\"browser\")".to_string(),
                "c".to_string(),
            ]
        );
    }

    #[test]
    fn quoted_field_may_contain_a_tab() {
        let line = "a\t\"has\ta\ttab\"\tc";
        let fields = split_tab_csv_line(line).unwrap();
        assert_eq!(
            fields,
            vec!["a".to_string(), "has\ta\ttab".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn unterminated_quote_is_an_error() {
        assert_eq!(
            split_tab_csv_line("a\t\"unterminated"),
            Err(ScanErrorKind::UnterminatedQuote)
        );
    }

    #[test]
    fn trailing_data_after_quoted_field_is_an_error() {
        assert_eq!(
            split_tab_csv_line("a\t\"quoted\"extra\tb"),
            Err(ScanErrorKind::TrailingDataAfterQuotedField)
        );
    }
}
