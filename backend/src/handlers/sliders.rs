use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use std::sync::Arc;

use crate::AppState;
use crate::models::*;

pub async fn list_public_sliders(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Slider>>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client.query(
        "SELECT id, title, description, image_url, link_url, is_visible, is_pinned, sort_order, created_at, updated_at
         FROM sliders WHERE is_visible = true
         ORDER BY is_pinned DESC, sort_order ASC, created_at DESC",
        &[],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows.iter().map(row_to_slider).collect()))
}

pub async fn admin_list_sliders(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Slider>>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client.query(
        "SELECT id, title, description, image_url, link_url, is_visible, is_pinned, sort_order, created_at, updated_at
         FROM sliders ORDER BY is_pinned DESC, sort_order ASC, created_at DESC",
        &[],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows.iter().map(row_to_slider).collect()))
}

pub async fn create_slider(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateSlider>,
) -> Result<(StatusCode, Json<Slider>), (StatusCode, Json<serde_json::Value>)> {
    let err = |msg: &str| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg})));

    if body.title.trim().is_empty() {
        return Err(err("Title is required"));
    }
    if body.image_url.trim().is_empty() {
        return Err(err("Image URL is required"));
    }

    let client = state.pool.get().await.map_err(|_| err("Database error"))?;

    let description = body.description.unwrap_or_default();
    let link_url = body.link_url.unwrap_or_default();
    let is_visible = body.is_visible.unwrap_or(true);

    let max_order: i32 = client.query_one(
        "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM sliders", &[],
    ).await.map_err(|_| err("Database error"))?.get(0);

    let row = client.query_one(
        "INSERT INTO sliders (title, description, image_url, link_url, is_visible, sort_order)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, title, description, image_url, link_url, is_visible, is_pinned, sort_order, created_at, updated_at",
        &[&body.title, &description, &body.image_url, &link_url, &is_visible, &max_order],
    ).await.map_err(|_| err("Failed to create slider"))?;

    Ok((StatusCode::CREATED, Json(row_to_slider(&row))))
}

pub async fn update_slider(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateSlider>,
) -> Result<Json<Slider>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let current = client.query_opt(
        "SELECT id, title, description, image_url, link_url, is_visible, is_pinned, sort_order, created_at, updated_at
         FROM sliders WHERE id = $1",
        &[&id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let current = row_to_slider(&current);

    let title = body.title.unwrap_or(current.title);
    let description = body.description.unwrap_or(current.description);
    let image_url = body.image_url.unwrap_or(current.image_url);
    let link_url = body.link_url.unwrap_or(current.link_url);
    let is_visible = body.is_visible.unwrap_or(current.is_visible);

    let row = client.query_one(
        "UPDATE sliders SET title=$1, description=$2, image_url=$3, link_url=$4, is_visible=$5, updated_at=NOW()
         WHERE id=$6
         RETURNING id, title, description, image_url, link_url, is_visible, is_pinned, sort_order, created_at, updated_at",
        &[&title, &description, &image_url, &link_url, &is_visible, &id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(row_to_slider(&row)))
}

pub async fn delete_slider(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let result = client.execute("DELETE FROM sliders WHERE id = $1", &[&id])
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result == 0 { Err(StatusCode::NOT_FOUND) } else { Ok(StatusCode::NO_CONTENT) }
}

pub async fn toggle_visibility(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<ToggleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    client.execute(
        "UPDATE sliders SET is_visible = $1, updated_at = NOW() WHERE id = $2",
        &[&body.value, &id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"is_visible": body.value})))
}

pub async fn toggle_pin(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<ToggleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    client.execute(
        "UPDATE sliders SET is_pinned = $1, updated_at = NOW() WHERE id = $2",
        &[&body.value, &id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"is_pinned": body.value})))
}

pub async fn reorder_sliders(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ReorderRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for item in &body.items {
        client.execute(
            "UPDATE sliders SET sort_order = $1, updated_at = NOW() WHERE id = $2 AND is_pinned = false",
            &[&item.sort_order, &item.id],
        ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(Json(serde_json::json!({"message": "Reordered successfully"})))
}

fn row_to_slider(row: &tokio_postgres::Row) -> Slider {
    Slider {
        id: row.get(0),
        title: row.get(1),
        description: row.get(2),
        image_url: row.get(3),
        link_url: row.get(4),
        is_visible: row.get(5),
        is_pinned: row.get(6),
        sort_order: row.get(7),
        created_at: row.get(8),
        updated_at: row.get(9),
    }
}
