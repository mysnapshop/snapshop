#[cfg(feature = "derive")]
pub use json_error_derive::*;

// Trait for getting the error code
pub trait ErrorCode {
    fn error_code(&self) -> i32;
}

#[cfg(test)]
mod test {
    use json_error_derive::ErrorCode;

    fn test_error_code() {
        #[derive(ErrorCode)]
        enum Enum {
            #[error_code(200)]
            String,
        }
    }
}
