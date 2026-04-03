use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use std::sync::Arc;

use crate::AppState;
use crate::models::*;

pub async fn list_menus(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Menu>>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client.query(
        "SELECT id, label, url, parent_id, sort_order, is_visible, open_in_new_tab, created_at
         FROM menus ORDER BY sort_order ASC, created_at ASC",
        &[],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let menus: Vec<Menu> = rows.iter().map(|row| Menu {
        id: row.get(0),
        label: row.get(1),
        url: row.get(2),
        parent_id: row.get(3),
        sort_order: row.get(4),
        is_visible: row.get(5),
        open_in_new_tab: row.get(6),
        created_at: row.get(7),
    }).collect();

    Ok(Json(menus))
}

pub async fn create_menu(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateMenu>,
) -> Result<(StatusCode, Json<Menu>), (StatusCode, Json<serde_json::Value>)> {
    let err = |msg: &str| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg})));

    if body.label.trim().is_empty() {
        return Err(err("Label is required"));
    }

    let client = state.pool.get().await.map_err(|_| err("Database error"))?;

    let is_visible = body.is_visible.unwrap_or(true);
    let open_in_new_tab = body.open_in_new_tab.unwrap_or(false);

    let max_order: i32 = client.query_one(
        "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM menus", &[],
    ).await.map_err(|_| err("Database error"))?.get(0);

    let row = client.query_one(
        "INSERT INTO menus (label, url, parent_id, sort_order, is_visible, open_in_new_tab)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, label, url, parent_id, sort_order, is_visible, open_in_new_tab, created_at",
        &[&body.label, &body.url, &body.parent_id, &max_order, &is_visible, &open_in_new_tab],
    ).await.map_err(|_| err("Failed to create menu"))?;

    Ok((StatusCode::CREATED, Json(Menu {
        id: row.get(0),
        label: row.get(1),
        url: row.get(2),
        parent_id: row.get(3),
        sort_order: row.get(4),
        is_visible: row.get(5),
        open_in_new_tab: row.get(6),
        created_at: row.get(7),
    })))
}

pub async fn update_menu(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateMenu>,
) -> Result<Json<Menu>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let current = client.query_opt(
        "SELECT id, label, url, parent_id, sort_order, is_visible, open_in_new_tab, created_at
         FROM menus WHERE id = $1",
        &[&id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let label = body.label.unwrap_or_else(|| current.get(1));
    let url = body.url.unwrap_or_else(|| current.get(2));
    let parent_id: Option<Uuid> = if body.parent_id.is_some() { body.parent_id } else { current.get(3) };
    let is_visible = body.is_visible.unwrap_or_else(|| current.get(5));
    let open_in_new_tab = body.open_in_new_tab.unwrap_or_else(|| current.get(6));

    let row = client.query_one(
        "UPDATE menus SET label=$1, url=$2, parent_id=$3, is_visible=$4, open_in_new_tab=$5
         WHERE id=$6
         RETURNING id, label, url, parent_id, sort_order, is_visible, open_in_new_tab, created_at",
        &[&label, &url, &parent_id, &is_visible, &open_in_new_tab, &id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Menu {
        id: row.get(0),
        label: row.get(1),
        url: row.get(2),
        parent_id: row.get(3),
        sort_order: row.get(4),
        is_visible: row.get(5),
        open_in_new_tab: row.get(6),
        created_at: row.get(7),
    }))
}

pub async fn delete_menu(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let result = client.execute("DELETE FROM menus WHERE id = $1", &[&id])
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result == 0 { Err(StatusCode::NOT_FOUND) } else { Ok(StatusCode::NO_CONTENT) }
}

pub async fn reorder_menus(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ReorderRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for item in &body.items {
        client.execute(
            "UPDATE menus SET sort_order = $1 WHERE id = $2",
            &[&item.sort_order, &item.id],
        ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(Json(serde_json::json!({"message": "Reordered successfully"})))
}
