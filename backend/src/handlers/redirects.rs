use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use std::sync::Arc;

use crate::AppState;
use crate::models::*;

pub async fn list_redirects(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Redirect>>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client.query(
        "SELECT id, from_path, to_path, status_code, created_at
         FROM redirects ORDER BY created_at DESC",
        &[],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let redirects: Vec<Redirect> = rows.iter().map(|row| Redirect {
        id: row.get(0),
        from_path: row.get(1),
        to_path: row.get(2),
        status_code: row.get(3),
        created_at: row.get(4),
    }).collect();

    Ok(Json(redirects))
}

pub async fn create_redirect(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateRedirect>,
) -> Result<(StatusCode, Json<Redirect>), (StatusCode, Json<serde_json::Value>)> {
    let err = |msg: &str| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg})));

    if body.from_path.trim().is_empty() || body.to_path.trim().is_empty() {
        return Err(err("Both from_path and to_path are required"));
    }

    let status_code = body.status_code.unwrap_or(301);
    if status_code != 301 && status_code != 302 {
        return Err(err("Status code must be 301 or 302"));
    }

    let client = state.pool.get().await.map_err(|_| err("Database error"))?;

    let row = client.query_one(
        "INSERT INTO redirects (from_path, to_path, status_code)
         VALUES ($1, $2, $3)
         RETURNING id, from_path, to_path, status_code, created_at",
        &[&body.from_path, &body.to_path, &status_code],
    ).await.map_err(|e| {
        if e.to_string().contains("duplicate key") {
            err("A redirect for this path already exists")
        } else {
            err("Failed to create redirect")
        }
    })?;

    Ok((StatusCode::CREATED, Json(Redirect {
        id: row.get(0),
        from_path: row.get(1),
        to_path: row.get(2),
        status_code: row.get(3),
        created_at: row.get(4),
    })))
}

pub async fn delete_redirect(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let result = client.execute("DELETE FROM redirects WHERE id = $1", &[&id])
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result == 0 { Err(StatusCode::NOT_FOUND) } else { Ok(StatusCode::NO_CONTENT) }
}
