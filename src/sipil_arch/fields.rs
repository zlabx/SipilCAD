//! A list of texts in one line of text, for the places that keep a definition as
//! a single string (a custom property of a project). Fields are joined with `|`;
//! a `%` or `|` inside a field is written `%25` or `%7C`. Pure: `std` only.

fn escape(s: &str) -> String {
    s.replace('%', "%25").replace('|', "%7C")
}

fn unescape(s: &str) -> String {
    s.replace("%7C", "|").replace("%25", "%")
}

pub fn join(parts: &[String]) -> String {
    parts
        .iter()
        .map(|p| escape(p))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn split(text: &str) -> Vec<String> {
    text.split('|').map(unescape).collect()
}
