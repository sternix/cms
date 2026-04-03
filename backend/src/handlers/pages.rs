use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use std::sync::Arc;

use crate::AppState;
use crate::models::*;

fn sanitize_html(input: &str) -> String {
    ammonia::clean(input)
}

fn make_slug(title: &str) -> String {
    slug::slugify(title)
}

// ── Public Endpoints ──

pub async fn list_public_pages(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<PaginatedResponse<Page>>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let page: i64 = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1).max(1);
    let per_page: i64 = params.get("per_page").and_then(|p| p.parse().ok()).unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let total_row = client.query_one(
        "SELECT COUNT(*) FROM pages WHERE is_visible = true", &[],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let total: i64 = total_row.get(0);

    let rows = client.query(
        "SELECT id, title, slug, content, excerpt, meta_title, meta_description, tags,
                is_visible, is_pinned, sort_order, created_at, updated_at
         FROM pages WHERE is_visible = true
         ORDER BY is_pinned DESC, sort_order ASC, created_at DESC
         LIMIT $1 OFFSET $2",
        &[&per_page, &offset],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data = rows.iter().map(row_to_page).collect();

    Ok(Json(PaginatedResponse {
        data,
        total,
        page,
        per_page,
        total_pages: (total as f64 / per_page as f64).ceil() as i64,
    }))
}

pub async fn get_public_page(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Page>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = client.query_opt(
        "SELECT id, title, slug, content, excerpt, meta_title, meta_description, tags,
                is_visible, is_pinned, sort_order, created_at, updated_at
         FROM pages WHERE slug = $1 AND is_visible = true",
        &[&slug],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(row) => Ok(Json(row_to_page(&row))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn search_pages(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<PaginatedResponse<Page>>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let search_term = query.q.trim();
    if search_term.is_empty() {
        return Ok(Json(PaginatedResponse {
            data: vec![], total: 0, page, per_page, total_pages: 0,
        }));
    }

    // Convert search term to tsquery - split words and join with &
    let tsquery = search_term
        .split_whitespace()
        .map(|w| format!("{}:*", w.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(" & ");

    let total_row = client.query_one(
        "SELECT COUNT(*) FROM pages WHERE is_visible = true AND search_vector @@ to_tsquery('simple', $1)",
        &[&tsquery],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let total: i64 = total_row.get(0);

    let rows = client.query(
        "SELECT id, title, slug, content, excerpt, meta_title, meta_description, tags,
                is_visible, is_pinned, sort_order, created_at, updated_at
         FROM pages WHERE is_visible = true AND search_vector @@ to_tsquery('simple', $1)
         ORDER BY ts_rank(search_vector, to_tsquery('simple', $1)) DESC
         LIMIT $2 OFFSET $3",
        &[&tsquery, &per_page, &offset],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data = rows.iter().map(row_to_page).collect();

    Ok(Json(PaginatedResponse {
        data, total, page, per_page,
        total_pages: (total as f64 / per_page as f64).ceil() as i64,
    }))
}

pub async fn list_tags(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client.query(
        "SELECT DISTINCT unnest(tags) as tag, COUNT(*) as count
         FROM pages WHERE is_visible = true
         GROUP BY tag ORDER BY count DESC",
        &[],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tags: Vec<serde_json::Value> = rows.iter().map(|row| {
        serde_json::json!({
            "name": row.get::<_, String>(0),
            "count": row.get::<_, i64>(1),
        })
    }).collect();

    Ok(Json(tags))
}

pub async fn pages_by_tag(
    State(state): State<Arc<AppState>>,
    Path(tag): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<PaginatedResponse<Page>>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let page: i64 = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1).max(1);
    let per_page: i64 = params.get("per_page").and_then(|p| p.parse().ok()).unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let total_row = client.query_one(
        "SELECT COUNT(*) FROM pages WHERE is_visible = true AND $1 = ANY(tags)",
        &[&tag],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let total: i64 = total_row.get(0);

    let rows = client.query(
        "SELECT id, title, slug, content, excerpt, meta_title, meta_description, tags,
                is_visible, is_pinned, sort_order, created_at, updated_at
         FROM pages WHERE is_visible = true AND $1 = ANY(tags)
         ORDER BY is_pinned DESC, sort_order ASC, created_at DESC
         LIMIT $2 OFFSET $3",
        &[&tag, &per_page, &offset],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data = rows.iter().map(row_to_page).collect();

    Ok(Json(PaginatedResponse {
        data, total, page, per_page,
        total_pages: (total as f64 / per_page as f64).ceil() as i64,
    }))
}

// ── Admin Endpoints ──

pub async fn admin_list_pages(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<PaginatedResponse<Page>>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let page: i64 = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1).max(1);
    let per_page: i64 = params.get("per_page").and_then(|p| p.parse().ok()).unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let total_row = client.query_one("SELECT COUNT(*) FROM pages", &[])
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let total: i64 = total_row.get(0);

    let rows = client.query(
        "SELECT id, title, slug, content, excerpt, meta_title, meta_description, tags,
                is_visible, is_pinned, sort_order, created_at, updated_at
         FROM pages ORDER BY is_pinned DESC, sort_order ASC, created_at DESC
         LIMIT $1 OFFSET $2",
        &[&per_page, &offset],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data = rows.iter().map(row_to_page).collect();

    Ok(Json(PaginatedResponse {
        data, total, page, per_page,
        total_pages: (total as f64 / per_page as f64).ceil() as i64,
    }))
}

pub async fn admin_get_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Page>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = client.query_opt(
        "SELECT id, title, slug, content, excerpt, meta_title, meta_description, tags,
                is_visible, is_pinned, sort_order, created_at, updated_at
         FROM pages WHERE id = $1",
        &[&id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(row) => Ok(Json(row_to_page(&row))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_page(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreatePage>,
) -> Result<(StatusCode, Json<Page>), (StatusCode, Json<serde_json::Value>)> {
    let err = |msg: &str| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg})));

    if body.title.trim().is_empty() {
        return Err(err("Title is required"));
    }
    if body.title.len() > 500 {
        return Err(err("Title too long"));
    }

    let client = state.pool.get().await.map_err(|_| err("Database error"))?;

    let slug = body.slug.as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| make_slug(&body.title));

    let content = sanitize_html(&body.content);
    let excerpt = body.excerpt.as_deref().unwrap_or("").to_string();
    let meta_title = body.meta_title.as_deref().unwrap_or("").to_string();
    let meta_description = body.meta_description.as_deref().unwrap_or("").to_string();
    let tags: Vec<String> = body.tags.unwrap_or_default();
    let is_visible = body.is_visible.unwrap_or(true);

    // Get next sort order
    let max_order = client.query_one(
        "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM pages", &[],
    ).await.map_err(|_| err("Database error"))?;
    let sort_order: i32 = max_order.get(0);

    let row = client.query_one(
        "INSERT INTO pages (title, slug, content, excerpt, meta_title, meta_description, tags, is_visible, sort_order)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING id, title, slug, content, excerpt, meta_title, meta_description, tags,
                   is_visible, is_pinned, sort_order, created_at, updated_at",
        &[&body.title, &slug, &content, &excerpt, &meta_title, &meta_description, &tags, &is_visible, &sort_order],
    ).await.map_err(|e| {
        if e.to_string().contains("duplicate key") {
            err("A page with this slug already exists")
        } else {
            err("Failed to create page")
        }
    })?;

    let page = row_to_page(&row);

    // Create initial revision
    client.execute(
        "INSERT INTO revisions (page_id, title, content, excerpt, meta_title, meta_description, tags, revision_number)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 1)",
        &[&page.id, &page.title, &page.content, &page.excerpt, &page.meta_title, &page.meta_description, &page.tags],
    ).await.ok();

    Ok((StatusCode::CREATED, Json(page)))
}

pub async fn update_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdatePage>,
) -> Result<Json<Page>, (StatusCode, Json<serde_json::Value>)> {
    let err = |msg: &str| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg})));

    let client = state.pool.get().await.map_err(|_| err("Database error"))?;

    // Fetch current
    let current = client.query_opt(
        "SELECT id, title, slug, content, excerpt, meta_title, meta_description, tags,
                is_visible, is_pinned, sort_order, created_at, updated_at
         FROM pages WHERE id = $1",
        &[&id],
    ).await.map_err(|_| err("Database error"))?;

    let current = current.ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Page not found"}))))?;
    let current = row_to_page(&current);

    let title = body.title.unwrap_or(current.title);
    let slug = body.slug.unwrap_or(current.slug);
    let content = body.content.map(|c| sanitize_html(&c)).unwrap_or(current.content);
    let excerpt = body.excerpt.unwrap_or(current.excerpt);
    let meta_title = body.meta_title.unwrap_or(current.meta_title);
    let meta_description = body.meta_description.unwrap_or(current.meta_description);
    let tags = body.tags.unwrap_or(current.tags);
    let is_visible = body.is_visible.unwrap_or(current.is_visible);

    let row = client.query_one(
        "UPDATE pages SET title=$1, slug=$2, content=$3, excerpt=$4, meta_title=$5,
         meta_description=$6, tags=$7, is_visible=$8, updated_at=NOW()
         WHERE id=$9
         RETURNING id, title, slug, content, excerpt, meta_title, meta_description, tags,
                   is_visible, is_pinned, sort_order, created_at, updated_at",
        &[&title, &slug, &content, &excerpt, &meta_title, &meta_description, &tags, &is_visible, &id],
    ).await.map_err(|e| {
        if e.to_string().contains("duplicate key") {
            err("A page with this slug already exists")
        } else {
            err("Failed to update page")
        }
    })?;

    let page = row_to_page(&row);

    // Create revision
    let rev_num: i32 = client.query_one(
        "SELECT COALESCE(MAX(revision_number), 0) + 1 FROM revisions WHERE page_id = $1",
        &[&id],
    ).await.map_err(|_| err("Database error"))?.get(0);

    client.execute(
        "INSERT INTO revisions (page_id, title, content, excerpt, meta_title, meta_description, tags, revision_number)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        &[&id, &page.title, &page.content, &page.excerpt, &page.meta_title, &page.meta_description, &page.tags, &rev_num],
    ).await.ok();

    Ok(Json(page))
}

pub async fn delete_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let result = client.execute("DELETE FROM pages WHERE id = $1", &[&id])
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

pub async fn toggle_visibility(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<ToggleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    client.execute(
        "UPDATE pages SET is_visible = $1, updated_at = NOW() WHERE id = $2",
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
        "UPDATE pages SET is_pinned = $1, updated_at = NOW() WHERE id = $2",
        &[&body.value, &id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"is_pinned": body.value})))
}

pub async fn reorder_pages(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ReorderRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for item in &body.items {
        client.execute(
            "UPDATE pages SET sort_order = $1, updated_at = NOW() WHERE id = $2 AND is_pinned = false",
            &[&item.sort_order, &item.id],
        ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(serde_json::json!({"message": "Reordered successfully"})))
}

// ── Revisions ──

pub async fn list_revisions(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Revision>>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client.query(
        "SELECT id, page_id, title, content, excerpt, meta_title, meta_description, tags, revision_number, created_at
         FROM revisions WHERE page_id = $1 ORDER BY revision_number DESC",
        &[&id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let revisions: Vec<Revision> = rows.iter().map(|row| Revision {
        id: row.get(0),
        page_id: row.get(1),
        title: row.get(2),
        content: row.get(3),
        excerpt: row.get(4),
        meta_title: row.get(5),
        meta_description: row.get(6),
        tags: row.get(7),
        revision_number: row.get(8),
        created_at: row.get(9),
    }).collect();

    Ok(Json(revisions))
}

pub async fn get_revision(
    State(state): State<Arc<AppState>>,
    Path((id, rev_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Revision>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = client.query_opt(
        "SELECT id, page_id, title, content, excerpt, meta_title, meta_description, tags, revision_number, created_at
         FROM revisions WHERE id = $1 AND page_id = $2",
        &[&rev_id, &id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(row) => Ok(Json(Revision {
            id: row.get(0),
            page_id: row.get(1),
            title: row.get(2),
            content: row.get(3),
            excerpt: row.get(4),
            meta_title: row.get(5),
            meta_description: row.get(6),
            tags: row.get(7),
            revision_number: row.get(8),
            created_at: row.get(9),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn restore_revision(
    State(state): State<Arc<AppState>>,
    Path((id, rev_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Page>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rev = client.query_opt(
        "SELECT title, content, excerpt, meta_title, meta_description, tags
         FROM revisions WHERE id = $1 AND page_id = $2",
        &[&rev_id, &id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let title: String = rev.get(0);
    let content: String = rev.get(1);
    let excerpt: String = rev.get(2);
    let meta_title: String = rev.get(3);
    let meta_description: String = rev.get(4);
    let tags: Vec<String> = rev.get(5);

    let row = client.query_one(
        "UPDATE pages SET title=$1, content=$2, excerpt=$3, meta_title=$4,
         meta_description=$5, tags=$6, updated_at=NOW()
         WHERE id=$7
         RETURNING id, title, slug, content, excerpt, meta_title, meta_description, tags,
                   is_visible, is_pinned, sort_order, created_at, updated_at",
        &[&title, &content, &excerpt, &meta_title, &meta_description, &tags, &id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create a new revision for the restore
    let rev_num: i32 = client.query_one(
        "SELECT COALESCE(MAX(revision_number), 0) + 1 FROM revisions WHERE page_id = $1",
        &[&id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.get(0);

    client.execute(
        "INSERT INTO revisions (page_id, title, content, excerpt, meta_title, meta_description, tags, revision_number)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        &[&id, &title, &content, &excerpt, &meta_title, &meta_description, &tags, &rev_num],
    ).await.ok();

    Ok(Json(row_to_page(&row)))
}

fn row_to_page(row: &tokio_postgres::Row) -> Page {
    Page {
        id: row.get(0),
        title: row.get(1),
        slug: row.get(2),
        content: row.get(3),
        excerpt: row.get(4),
        meta_title: row.get(5),
        meta_description: row.get(6),
        tags: row.get(7),
        is_visible: row.get(8),
        is_pinned: row.get(9),
        sort_order: row.get(10),
        created_at: row.get(11),
        updated_at: row.get(12),
    }
}
