use lazy_static::lazy_static;
use std::env;
use std::sync::RwLock;

pub struct Env;

lazy_static! {
    static ref INITIALIZED: RwLock<bool> = RwLock::new(false);
}

pub fn init(path: &'static str) {
    dotenv::from_path(path).unwrap();
    let mut init = INITIALIZED.write().unwrap();
    *init = true;
}

pub fn get(key: &str) -> Option<String> {
    ensure_initialzed();
    match env::var(key) {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

pub fn set(key: &str, value: String) {
    ensure_initialzed();
    unsafe {
        env::set_var(key, value);
    }
}

fn ensure_initialzed() {
    let init = INITIALIZED.read().unwrap();
    if !*init {
        dotenv::dotenv().expect("crate:env could not be initialzed");
    }
}

#[cfg(test)]
mod tests {}
