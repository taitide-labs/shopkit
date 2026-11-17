use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ProblemDetails {
    pub status: u16,
    pub code: String,
    pub detail: String,
}

impl ProblemDetails {
    #[must_use]
    pub fn bad_request(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST.as_u16(),
            code: code.into(),
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn unprocessable(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
            code: code.into(),
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn not_found(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND.as_u16(),
            code: code.into(),
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn conflict(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT.as_u16(),
            code: code.into(),
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn not_implemented(slice_name: &str) -> Self {
        Self {
            status: StatusCode::NOT_IMPLEMENTED.as_u16(),
            code: "SLICE_NOT_IMPLEMENTED".to_owned(),
            detail: format!("垂直切片 {slice_name} 尚未启用"),
        }
    }

    #[must_use]
    pub fn internal(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            code: "INTERNAL_ERROR".to_owned(),
            detail: detail.into(),
        }
    }
}

impl IntoResponse for ProblemDetails {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}
