use lazy_static::lazy_static;
use regex::Regex;

#[cfg(test)]
use testcontainers::{runners::AsyncRunner, ContainerAsync};
#[cfg(test)]
use testcontainers_modules::mongo::Mongo;

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

#[cfg(test)]
pub async fn setup_test_db() -> (ContainerAsync<Mongo>, String) {
    let server = Mongo::new().start().await.unwrap();
    let host = server.get_host().await.unwrap();
    let port = server.get_host_port_ipv4(27017).await.unwrap();
    (server, format!("mongodb://{}:{}/", host, port))
}
