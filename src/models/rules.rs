//! Rules every PARA item and note shares.
//!
//! Names are never blank, optional text is trimmed (blank clears it), and rule
//! breaks are validation errors, so the API answers 400 and the CLI prints
//! `field: reason`.
#![allow(clippy::missing_errors_doc)]
use std::collections::HashMap;

use loco_rs::{
    model::ModelError,
    validation::{ModelValidationErrors, ValidationError},
};

/// A validation error on one field.
#[must_use]
pub fn invalid(field: &str, message: &str) -> ModelError {
    let error = ValidationError {
        code: field.to_string(),
        message: Some(message.to_string()),
        params: HashMap::new(),
    };
    ModelValidationErrors {
        errors: [(field.to_string(), vec![error])].into(),
    }
    .into()
}

/// A required name or title, trimmed; blank is an error.
pub fn text(field: &str, value: &str) -> Result<String, ModelError> {
    let value = value.trim();
    if value.is_empty() {
        Err(invalid(field, &format!("{field} cannot be blank")))
    } else {
        Ok(value.to_string())
    }
}

/// Optional text, trimmed; blank means "none".
#[must_use]
pub fn optional(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// `value` must be one of `allowed`.
pub fn one_of(field: &str, value: &str, allowed: &[&str]) -> Result<(), ModelError> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(invalid(
            field,
            &format!("{field} must be one of {}", allowed.join(", ")),
        ))
    }
}

/// An absolute `http(s)` URL.
pub fn url(field: &str, value: &str) -> Result<(), ModelError> {
    match url::Url::parse(value) {
        Ok(u) if matches!(u.scheme(), "http" | "https") && u.host().is_some() => Ok(()),
        _ => Err(invalid(field, &format!("{field} must be an http(s) URL"))),
    }
}

/// A pattern for SQL `LIKE ... ESCAPE '\'` that matches `s` literally.
#[must_use]
pub fn like_literal(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_rejects_blank() {
        assert_eq!(text("name", "  Health ").unwrap(), "Health");
        assert!(text("name", " \n").is_err());
        assert_eq!(optional(Some("  ".into())), None);
        assert_eq!(optional(Some(" x ".into())), Some("x".into()));
    }

    #[test]
    fn checks_urls_and_escapes_like() {
        assert!(url("url", "https://arxiv.org/abs/1706.03762").is_ok());
        assert!(url("url", "ftp://example.com").is_err());
        assert!(url("url", "not a url").is_err());
        assert_eq!(like_literal("50%_a\\b"), "50\\%\\_a\\\\b");
    }
}
