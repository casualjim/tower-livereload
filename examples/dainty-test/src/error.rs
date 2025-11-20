use axum::response::IntoResponse;
use http::StatusCode;
use serde::Serialize;
use serde_json::Value;
use tracing::error;
use uuid::Uuid;

/// A default error response for most API errors.
#[derive(Debug, Serialize)]
pub struct AppError {
  /// An error message.
  pub error: String,
  /// A unique error ID.
  pub error_id: Uuid,
  #[serde(skip)]
  pub status: StatusCode,
  /// Optional Additional error details.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error_details: Option<Value>,
}

impl AppError {
  pub fn new(error: &str) -> Self {
    error!("{error}");
    Self {
      error: error.to_string(),
      error_id: Uuid::now_v7(),
      status: StatusCode::BAD_REQUEST,
      error_details: None,
    }
  }

  pub fn with_status(mut self, status: StatusCode) -> Self {
    self.status = status;
    self
  }

  #[allow(unused)]
  pub fn into_pair(self) -> (StatusCode, axum::Json<Self>) {
    (self.status, axum::Json(self))
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> axum::response::Response {
    let status = self.status;
    (status, axum::Json(self)).into_response()
  }
}
