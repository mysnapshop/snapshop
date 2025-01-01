use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref email_regex: Regex =
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    static ref password_regex: Regex = Regex::new(r"^[a-zA-Z0-9@_\-$+!*]+$").unwrap();
}

pub fn validate_email<'a>(haystack: &'a str) -> bool {
    email_regex.is_match(haystack)
}

pub fn validate_passowrd<'a>(haystack: &'a str) -> bool {
    password_regex.is_match(haystack)
}
