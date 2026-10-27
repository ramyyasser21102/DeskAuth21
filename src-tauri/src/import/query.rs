//! Query-string helpers shared by both URI formats.

use std::borrow::Cow;
use std::str::FromStr;

pub fn query_param(query: &str, key: &str) -> Option<String> {
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(name, _)| *name == key)
        .and_then(|(_, raw_value)| urlencoding::decode(raw_value).ok())
        .map(Cow::into_owned)
}

/// Parsed numeric parameter, or `default` when absent or unparsable.
pub fn numeric_param<T: FromStr>(query: &str, key: &str, default: T) -> T {
    query_param(query, key)
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_and_percent_decodes_values() {
        assert_eq!(query_param("a=1&b=x%40y", "b").as_deref(), Some("x@y"));
        assert_eq!(query_param("a=1", "missing"), None);
    }

    #[test]
    fn numeric_param_falls_back_to_default() {
        assert_eq!(numeric_param("digits=8", "digits", 6u32), 8);
        assert_eq!(numeric_param("digits=abc", "digits", 6u32), 6);
        assert_eq!(numeric_param("", "digits", 6u32), 6);
    }
}
