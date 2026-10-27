//! Query-string helpers shared by both URI formats.

use std::borrow::Cow;

pub fn query_param(query: &str, key: &str) -> Option<String> {
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(name, _)| *name == key)
        .and_then(|(_, raw_value)| urlencoding::decode(raw_value).ok())
        .map(Cow::into_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_and_percent_decodes_values() {
        assert_eq!(query_param("a=1&b=x%40y", "b").as_deref(), Some("x@y"));
        assert_eq!(query_param("a=1", "missing"), None);
    }
}
