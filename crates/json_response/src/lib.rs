use std::fmt::Display;

use salvo::{async_trait, writing::Json, Depot, Request, Response, Writer};
use serde::{Deserialize, Serialize};

#[cfg(feature = "derive")]
pub use json_response_derive::*;

/// Trait for types that represent an error.
///
/// This trait provides a method to get an error code associated with the error.
pub trait ErrorCode: Serialize + Display + Send {
    /// Returns the error code associated with the error.
    fn error_code(&self) -> u16;
}

/// Trait for types that can be converted into a JSON response.
pub trait ToJson<T: Send, E: ErrorCode> {
    /// Converts the type into a `Json<ApiResponse<T, E>>`.
    fn to_json(self) -> Json<ApiResponse<T, E>>
    where
        T: Serialize + Sized;
}

/// Represents the status of an API response.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
pub struct ApiError<E: ErrorCode> {
    /// The error code.
    code: u16,
    /// The error message.
    message: String,

    #[serde(skip_serializing)]
    _inner: E,
}

impl<E: ErrorCode> ToJson<(), E> for ApiError<E> {
    fn to_json(self) -> Json<ApiResponse<(), E>> {
        Json(ApiResponse::<(), E>::error(self._inner))
    }
}

/// Represents a generic API response.
#[derive(Serialize, Debug, PartialEq)]
pub struct ApiResponse<V: Serialize, E: ErrorCode> {
    /// The status of the response.
    status: ApiResponseStatus,

    /// The data returned by the API (if successful).
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<V>,

    /// The error encountered during the request (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ApiError<E>>,
}

impl<T: Serialize, E: ErrorCode> ApiResponse<T, E> {
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

impl<T: Serialize + Send, E: ErrorCode> ToJson<T, E> for ApiResponse<T, E> {
    fn to_json(self) -> Json<ApiResponse<T, E>> {
        Json(self)
    }
}

#[async_trait]
impl<T: Serialize + Send, E: ErrorCode> Writer for ApiResponse<T, E> {
    async fn write(mut self, _: &mut Request, _: &mut Depot, res: &mut Response) {
        res.render(self.to_json());
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorCode;

    use std::fmt::Display;

    use serde::Serialize;
    use serde_json::{self, json};

    use crate::{ApiResponse, ApiResponseStatus};

    #[derive(ErrorCode, Serialize, Debug, PartialEq)]
    enum Error {
        #[error_code(200)]
        UnitError,

        #[error_code(300)]
        TupleError(u16),

        #[error_code(400)]
        StructError { _inner: String },
    }

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    #[test]
    fn test_success_response() {
        let response: ApiResponse<String, Error> =
            ApiResponse::success("Hello, world!".to_string());
        let expected = json!({
            "status": "Success",
            "data": "Hello, world!"
        });
        assert_eq!(serde_json::to_value(&response).unwrap(), expected);
    }

    #[test]
    fn test_unit_error_response() {
        let error = Error::UnitError;
        let response: ApiResponse<(), Error> = ApiResponse::error(error);
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
        let error = Error::TupleError(123);
        let response: ApiResponse<(), Error> = ApiResponse::error(error);
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
        let error = Error::StructError {
            _inner: "Internal Error".to_string(),
        };
        let response: ApiResponse<(), Error> = ApiResponse::error(error);
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
        let response: ApiResponse<(), Error> = ApiResponse {
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
