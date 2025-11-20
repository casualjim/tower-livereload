mod components;
mod pages;

use axum::{
  Json, Router,
  extract::{FromRequestParts, Query, State, rejection::JsonRejection},
  http::{StatusCode, header, request::Parts},
  response::IntoResponse,
  routing::{MethodFilter, MethodRouter},
};
use axum_htmx::AutoVaryLayer;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use serde::Deserialize;
use serde_json::{Value, json};
use sha3::{Digest, Sha3_256};
use std::convert::Infallible;
use uuid::Uuid;

use crate::{
  app::AppState,
  pgdb::{GetLayoutState, SaveLayoutState},
};

const ANONYMOUS_USER_ID: &str = "anonymous";

#[derive(Debug, Clone)]
struct ApiUserId(String);

impl<S> FromRequestParts<S> for ApiUserId
where
  S: Send + Sync,
{
  type Rejection = Infallible;

  #[allow(clippy::manual_async_fn)]
  fn from_request_parts(
    parts: &mut Parts,
    _state: &S,
  ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
    async move {
      let user_id = parts
        .headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or(ANONYMOUS_USER_ID)
        .to_string();

      Ok(ApiUserId(user_id))
    }
  }
}

#[derive(Debug, Deserialize)]
struct LayoutQuery {
  path: Option<String>,
  device: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LayoutUpdate {
  path: Option<String>,
  device: Option<String>,
  #[serde(flatten)]
  settings: Value,
}

// Hash URL paths + device type for context keys (matches TypeScript hashPath)
fn hash_path(path: &str, device_type: &str) -> String {
  let combined = format!("{}:{}", path, device_type);
  let mut hasher = Sha3_256::new();
  hasher.update(combined.as_bytes());
  let result = hasher.finalize();
  // Use first 16 chars for readability, matching TypeScript
  format!("{:x}", result)[..16].to_string()
}

pub fn router(app: AppState) -> Router<AppState> {
  let layout_route = MethodRouter::new()
    .on(MethodFilter::GET, get_layout_state)
    .on(MethodFilter::POST, update_layout_state)
    .fallback(method_not_allowed);

  Router::new()
    .route("/api/layout", layout_route)
    .merge(pages::routes(app.clone()))
    .layer(AutoVaryLayer)
    .layer(OtelInResponseLayer)
    .layer(OtelAxumLayer::default())
    .with_state(app)
}

async fn get_layout_state(
  State(app): State<AppState>,
  ApiUserId(user_id): ApiUserId,
  Query(query): Query<LayoutQuery>,
) -> impl IntoResponse {
  let db = app.try_pgconn().await.unwrap();
  let path = query.path.as_deref().unwrap_or("/");
  let device = query.device.as_deref().unwrap_or("desktop");
  let context_key = hash_path(path, device);

  let params = GetLayoutState::builder()
    .user_id(&user_id)
    .context_key(&context_key)
    .build();

  match params.query_opt(&db).await {
    Ok(Some(state)) => Json(state.settings),
    _ => Json(json!({})),
  }
}

async fn update_layout_state(
  State(app): State<AppState>,
  ApiUserId(user_id): ApiUserId,
  payload: Result<Json<LayoutUpdate>, JsonRejection>,
) -> impl IntoResponse {
  let payload = match payload {
    Ok(Json(payload)) => payload,
    Err(_) => {
      return (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": "Invalid JSON payload" })),
      );
    }
  };

  let db = app.try_pgconn().await.unwrap();
  let path = payload.path.as_deref().unwrap_or("/");
  let device = payload.device.as_deref().unwrap_or("desktop");
  let context_key = hash_path(path, device);

  // Just upsert - ON CONFLICT handles the merge
  let save_params = SaveLayoutState::builder()
    .id(Uuid::now_v7())
    .user_id(&user_id)
    .context_key(&context_key)
    .settings(&payload.settings)
    .build();

  let state = save_params.query_one(&db).await.unwrap();
  (StatusCode::OK, Json(state.settings))
}

async fn method_not_allowed() -> impl IntoResponse {
  (
    StatusCode::METHOD_NOT_ALLOWED,
    [(header::ALLOW, "GET, POST")],
    Json(json!({ "error": "Method not allowed" })),
  )
}
