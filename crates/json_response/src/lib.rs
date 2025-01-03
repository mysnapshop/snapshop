use std::fmt::{Debug, Display};

use salvo::{async_trait, writing::Json, Depot, Request, Response, Writer};
use serde::{Deserialize, Serialize};

#[cfg(feature = "derive")]
pub use json_response_derive::*;

/// Trait for types that represent an error.
///
/// This trait provides a method to get an error code associated with the error.
pub trait Error: Display + Debug + Serialize + Send + ErrorLogger + PartialEq {
    /// Returns the error code associated with the error.
    fn error_code(&self) -> u16;
}

/// This trait provides a method to log error associated with a request.
pub trait ErrorLogger {
    fn log_error(&self, req: &mut Request);
}

/// Trait for types that can be converted into a JSON response.
pub trait ToJson<T: Send, E: Error> {
    /// Converts the type into a `Json<ApiResponse<T, E>>`.
    fn to_json(self) -> Json<ApiResponse<T, E>>
    where
        T: Serialize + Sized;
}

/// Represents the status of an API response.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ApiResponseStatus {
    Success,
    Failed,
}

impl Display for ApiResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiResponseStatus::Success => f.write_str("success"),
            ApiResponseStatus::Failed => f.write_str("failed"),
        }
    }
}

/// Represents an API error.
#[derive(Serialize, Debug, PartialEq)]
pub struct ApiError<E: Error> {
    /// The error code.
    code: u16,
    /// The error message.
    message: String,

    #[serde(skip_serializing)]
    _inner: E,
}

impl<E: Error> ToJson<(), E> for ApiError<E> {
    fn to_json(self) -> Json<ApiResponse<(), E>> {
        Json(ApiResponse::<(), E>::error(self._inner))
    }
}

/// Represents a generic API response.
#[derive(Serialize, Debug, PartialEq)]
pub struct ApiResponse<V: Serialize, E: Error> {
    /// The status of the response.
    status: ApiResponseStatus,

    /// The data returned by the API (if successful).
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<V>,

    /// The error encountered during the request (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ApiError<E>>,
}

impl<T: Serialize, E: Error> ApiResponse<T, E> {
    /// Creates a new `ApiResponse` with a `Failed` status and the given error.
    pub fn error(err: E) -> ApiResponse<T, E> {
        ApiResponse {
            status: ApiResponseStatus::Failed,
            data: None,
            error: Some(ApiError {
                code: err.error_code(),
                message: format!("{}", err),
                _inner: err,
            }),
        }
    }

    /// Creates a new `ApiResponse` with a `Success` status and the given data.
    pub fn success(data: T) -> ApiResponse<T, E> {
        ApiResponse {
            status: ApiResponseStatus::Success,
            data: Some(data),
            error: None,
        }
    }
}

impl<T: Serialize + Send, E: Error> ToJson<T, E> for ApiResponse<T, E> {
    fn to_json(self) -> Json<ApiResponse<T, E>> {
        Json(self)
    }
}

#[async_trait]
impl<T: Serialize + Send, E: Error> Writer for ApiResponse<T, E> {
    async fn write(mut self, req: &mut Request, _: &mut Depot, res: &mut Response) {
        if let Some(err) = &self.error {
            err._inner.log_error(req);
        }
        res.render(self.to_json());
    }
}

#[derive(Serialize, Error, PartialEq, Debug)]
pub enum RequestError<E: Error> {
    #[error_code(401)]
    Unauthorized,
    #[error_code(400)]
    BadRequest(String),
    #[error_code(500)]
    InternalServerError(String),
    ServiceError(E),
}

impl<E: Error> Display for RequestError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unauthorized => f.write_str("Unauthorized"),
            Self::BadRequest(message) => f.write_str(format!("BadRequest:{}", message).as_str()),
            Self::InternalServerError(_) => f.write_str("InternalServerError"),
            Self::ServiceError(err) => f.write_str(format!("{}", err).as_str()),
        }
    }
}

impl<E: Error> ErrorLogger for RequestError<E> {
    fn log_error(&self, req: &mut Request) {
        match self {
            Self::InternalServerError(err) => {
                let span = tracing::span!(
                    tracing::Level::ERROR,
                    "InternalServerError",
                    method = %req.method(),
                    path = %req.uri().path(),
                    client_ip = %req.remote_addr().to_string(),
                );
                let _enter = span.enter();

                tracing::error!("{err}");
            }
            Self::ServiceError(err) => err.log_error(req),
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Error;

    use std::fmt::Display;

    use serde::Serialize;
    use serde_json::{self, json};

    use crate::{ApiResponse, ApiResponseStatus, ErrorLogger};

    #[derive(Error, Serialize, Debug, PartialEq)]
    enum MyError {
        #[error_code(200)]
        UnitError,

        #[error_code(300)]
        TupleError(u16),

        #[error_code(400)]
        StructError { _inner: String },
    }

    impl Display for MyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl ErrorLogger for MyError {
        fn log_error(&self, _: &mut salvo::Request) {
            ()
        }
    }

    #[test]
    fn test_success_response() {
        let response: ApiResponse<String, MyError> =
            ApiResponse::success("Hello, world!".to_string());
        let expected = json!({
            "status": "Success",
            "data": "Hello, world!"
        });
        assert_eq!(serde_json::to_value(&response).unwrap(), expected);
    }

    #[test]
    fn test_unit_error_response() {
        let error = MyError::UnitError;
        let response: ApiResponse<(), MyError> = ApiResponse::error(error);
        let expected = json!({
            "status": "Failed",
            "error": {
                "code": 200,
                "message": "UnitError",
            }
        });
        assert_eq!(serde_json::to_value(&response).unwrap(), expected);
    }

    #[test]
    fn test_tuple_error_response() {
        let error = MyError::TupleError(123);
        let response: ApiResponse<(), MyError> = ApiResponse::error(error);
        let expected = json!({
            "status": "Failed",
            "error": {
                "code": 300,
                "message": "TupleError(123)",
            }
        });
        assert_eq!(serde_json::to_value(&response).unwrap(), expected);
    }

    #[test]
    fn test_struct_error_response() {
        let error = MyError::StructError {
            _inner: "Internal Error".to_string(),
        };
        let response: ApiResponse<(), MyError> = ApiResponse::error(error);
        let expected = json!({
            "status": "Failed",
            "error": {
                "code": 400,
                "message": "StructError { _inner: \"Internal Error\" }",
            }
        });
        assert_eq!(serde_json::to_value(&response).unwrap(), expected);
    }

    #[test]
    fn test_empty_response() {
        let response: ApiResponse<(), MyError> = ApiResponse {
            status: ApiResponseStatus::Success,
            data: None,
            error: None,
        };
        let expected = json!({
            "status": "Success"
        });
        assert_eq!(serde_json::to_value(&response).unwrap(), expected);
    }
}
