//! Parsing of form values sent by the pages.

use crate::web::app_error::AppError;

/// Parses a comma-separated list of card numbers (`"3,5,5"`); blanks are ignored.
///
/// # Errors
/// [`AppError::BadRequest`] when an entry isn't a number.
pub fn card_list(text: &str) -> Result<Vec<u32>, AppError> {
    text.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse()
                .map_err(|_| AppError::BadRequest(format!("invalid card '{s}'")))
        })
        .collect()
}

/// Parses a single number field.
///
/// # Errors
/// [`AppError::BadRequest`] when the value isn't a number.
pub fn number<T: std::str::FromStr>(text: &str, field: &str) -> Result<T, AppError> {
    text.trim()
        .parse()
        .map_err(|_| AppError::BadRequest(format!("invalid {field}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_card_lists() {
        assert_eq!(card_list("3, 5,,5").unwrap(), vec![3, 5, 5]);
        assert!(card_list("").unwrap().is_empty());
        assert!(card_list("x").is_err());
    }
}
