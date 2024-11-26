use std::collections::HashMap;

use tracing::warn;
use url::Url;
pub type Redirects = HashMap<String, String>;

/// Parse the contents of a redirects file (usually fetched from the internet), returning
/// a HashMap that maps redirect aliases -> URLs.
pub fn parse_redirects(contents: &str) -> Redirects {
    let mut map = HashMap::new();

    contents.lines().for_each(|l| {
        // Ignore empty lines, and lines beginning with a '#'.
        if l.starts_with("#") || l.is_empty() {
            return;
        }

        let parts: Vec<&str> = l.split(" ").collect();
        // Ignore lines with more than one space.
        if parts.len() != 2 {
            warn!("invalid redirect specification: '{}'", l);
            return;
        }

        // Check the URL for a given key is actually a valid URL.
        if Url::parse(parts[1]).is_ok() {
            map.insert(parts[0].to_string(), parts[1].to_string());
        } else {
            warn!("invalid url detected in redirects file: '{}'", parts[1]);
        }
    });

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        vec![
            "foo http://foo.bar",
            "# Comment",
            "bar http://bar.baz",
            "garbagethatshouldntbeparsed",
            "another line that shouldn't be parsed",
            "good-key but-a-bad-url",
        ],
        vec![
            ("foo", "http://foo.bar"),
            ("bar", "http://bar.baz"),
        ]
    )]
    #[case(
        vec![
            "key1 http://example.com",
            "key2 http://example.org",
            "# Another comment",
            "key3 http://example.net",
        ],
        vec![
            ("key1", "http://example.com"),
            ("key2", "http://example.org"),
            ("key3", "http://example.net"),
        ]
    )]
    fn test_parse_redirects(#[case] input: Vec<&str>, #[case] expected: Vec<(&str, &str)>) {
        let redirects = parse_redirects(&input.join("\n"));

        let mut expected_map = Redirects::new();
        for (key, value) in expected {
            expected_map.insert(key.to_string(), value.to_string());
        }

        assert_eq!(expected_map, redirects);
    }
}
