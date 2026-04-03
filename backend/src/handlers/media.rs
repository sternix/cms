use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use std::sync::Arc;

use crate::AppState;
use crate::models::*;

pub async fn list_media(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Media>>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = client.query(
        "SELECT id, filename, original_name, mime_type, size_bytes, width, height, url, created_at
         FROM media ORDER BY created_at DESC",
        &[],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let media: Vec<Media> = rows.iter().map(|row| Media {
        id: row.get(0),
        filename: row.get(1),
        original_name: row.get(2),
        mime_type: row.get(3),
        size_bytes: row.get(4),
        width: row.get(5),
        height: row.get(6),
        url: row.get(7),
        created_at: row.get(8),
    }).collect();

    Ok(Json(media))
}

pub async fn upload_media(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Media>), (StatusCode, Json<serde_json::Value>)> {
    let err = |msg: &str| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg})));

    while let Some(field) = multipart.next_field().await.map_err(|_| err("Invalid upload"))? {
        let name = field.name().unwrap_or("").to_string();
        if name != "file" {
            continue;
        }

        let original_name = field.file_name()
            .unwrap_or("unknown")
            .to_string();

        let content_type = field.content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        // Validate mime type
        let allowed_types = [
            "image/jpeg", "image/png", "image/gif", "image/webp", "image/svg+xml",
            "image/avif", "image/bmp", "image/tiff",
        ];
        if !allowed_types.contains(&content_type.as_str()) {
            return Err(err("File type not allowed. Allowed: JPEG, PNG, GIF, WebP, SVG, AVIF, BMP, TIFF"));
        }

        let data = field.bytes().await.map_err(|_| err("Failed to read file"))?;

        if data.len() > state.max_upload_size {
            return Err(err("File too large"));
        }

        let ext = original_name.rsplit('.').next().unwrap_or("bin");
        let filename = format!("{}.{}", Uuid::new_v4(), ext);
        let filepath = format!("{}/{}", state.upload_dir, filename);

        tokio::fs::write(&filepath, &data).await.map_err(|_| err("Failed to save file"))?;

        // Get image dimensions if applicable
        let (width, height) = if content_type.starts_with("image/") && content_type != "image/svg+xml" {
            match image::load_from_memory(&data) {
                Ok(img) => (Some(img.width() as i32), Some(img.height() as i32)),
                Err(_) => (None, None),
            }
        } else {
            (None, None)
        };

        let url = format!("/uploads/{}", filename);
        let size_bytes = data.len() as i64;

        let client = state.pool.get().await.map_err(|_| err("Database error"))?;

        let row = client.query_one(
            "INSERT INTO media (filename, original_name, mime_type, size_bytes, width, height, url)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, filename, original_name, mime_type, size_bytes, width, height, url, created_at",
            &[&filename, &original_name, &content_type, &size_bytes, &width, &height, &url],
        ).await.map_err(|_| err("Failed to save to database"))?;

        return Ok((StatusCode::CREATED, Json(Media {
            id: row.get(0),
            filename: row.get(1),
            original_name: row.get(2),
            mime_type: row.get(3),
            size_bytes: row.get(4),
            width: row.get(5),
            height: row.get(6),
            url: row.get(7),
            created_at: row.get(8),
        })));
    }

    Err(err("No file provided"))
}

pub async fn delete_media(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = client.query_opt(
        "SELECT filename FROM media WHERE id = $1", &[&id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(row) = row {
        let filename: String = row.get(0);
        let filepath = format!("{}/{}", state.upload_dir, filename);
        tokio::fs::remove_file(&filepath).await.ok();

        client.execute("DELETE FROM media WHERE id = $1", &[&id])
            .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn transform_media(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<TransformRequest>,
) -> Result<Json<Media>, (StatusCode, Json<serde_json::Value>)> {
    let err = |msg: &str| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg})));

    let client = state.pool.get().await.map_err(|_| err("Database error"))?;

    let row = client.query_opt(
        "SELECT filename, mime_type FROM media WHERE id = $1", &[&id],
    ).await.map_err(|_| err("Database error"))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Media not found"}))))?;

    let filename: String = row.get(0);
    let mime_type: String = row.get(1);

    if !mime_type.starts_with("image/") || mime_type == "image/svg+xml" {
        return Err(err("Can only transform raster images"));
    }

    let filepath = format!("{}/{}", state.upload_dir, filename);
    let data = tokio::fs::read(&filepath).await.map_err(|_| err("Failed to read file"))?;

    let mut img = image::load_from_memory(&data).map_err(|_| err("Failed to decode image"))?;

    // Crop first if requested
    if let (Some(cx), Some(cy), Some(cw), Some(ch)) = (body.crop_x, body.crop_y, body.crop_width, body.crop_height) {
        img = img.crop_imm(cx, cy, cw, ch);
    }

    // Resize if requested
    if let (Some(w), Some(h)) = (body.width, body.height) {
        img = img.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
    } else if let Some(w) = body.width {
        img = img.resize(w, u32::MAX, image::imageops::FilterType::Lanczos3);
    } else if let Some(h) = body.height {
        img = img.resize(u32::MAX, h, image::imageops::FilterType::Lanczos3);
    }

    // Save as new file
    let ext = filename.rsplit('.').next().unwrap_or("png");
    let new_filename = format!("{}.{}", Uuid::new_v4(), ext);
    let new_filepath = format!("{}/{}", state.upload_dir, new_filename);

    img.save(&new_filepath).map_err(|_| err("Failed to save transformed image"))?;

    let file_size = tokio::fs::metadata(&new_filepath).await
        .map(|m| m.len() as i64).unwrap_or(0);
    let new_url = format!("/uploads/{}", new_filename);
    let new_width = img.width() as i32;
    let new_height = img.height() as i32;

    // Update DB record
    let row = client.query_one(
        "UPDATE media SET filename=$1, url=$2, width=$3, height=$4, size_bytes=$5
         WHERE id=$6
         RETURNING id, filename, original_name, mime_type, size_bytes, width, height, url, created_at",
        &[&new_filename, &new_url, &new_width, &new_height, &file_size, &id],
    ).await.map_err(|_| err("Database error"))?;

    // Remove old file
    tokio::fs::remove_file(&filepath).await.ok();

    Ok(Json(Media {
        id: row.get(0),
        filename: row.get(1),
        original_name: row.get(2),
        mime_type: row.get(3),
        size_bytes: row.get(4),
        width: row.get(5),
        height: row.get(6),
        url: row.get(7),
        created_at: row.get(8),
    }))
}
