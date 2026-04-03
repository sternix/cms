use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::AppState;
use crate::models::*;

pub async fn get_public_settings(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = client.query_opt(
        "SELECT site_name, site_description, logo_url, favicon_url, footer_text, social_links
         FROM site_settings LIMIT 1",
        &[],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(row) => Ok(Json(serde_json::json!({
            "site_name": row.get::<_, String>(0),
            "site_description": row.get::<_, String>(1),
            "logo_url": row.get::<_, String>(2),
            "favicon_url": row.get::<_, String>(3),
            "footer_text": row.get::<_, String>(4),
            "social_links": row.get::<_, serde_json::Value>(5),
        }))),
        None => Ok(Json(serde_json::json!({
            "site_name": "CMS",
            "site_description": "",
            "logo_url": "",
            "favicon_url": "",
            "footer_text": "",
            "social_links": {},
        }))),
    }
}

pub async fn get_settings(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SiteSettings>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = client.query_opt(
        "SELECT id, site_name, site_description, logo_url, favicon_url, footer_text,
                social_links, custom_head_html, updated_at
         FROM site_settings LIMIT 1",
        &[],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(row) => Ok(Json(SiteSettings {
            id: row.get(0),
            site_name: row.get(1),
            site_description: row.get(2),
            logo_url: row.get(3),
            favicon_url: row.get(4),
            footer_text: row.get(5),
            social_links: row.get(6),
            custom_head_html: row.get(7),
            updated_at: row.get(8),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateSettings>,
) -> Result<Json<SiteSettings>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Fetch current
    let current = client.query_opt(
        "SELECT id, site_name, site_description, logo_url, favicon_url, footer_text,
                social_links, custom_head_html, updated_at
         FROM site_settings LIMIT 1",
        &[],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let current = current.ok_or(StatusCode::NOT_FOUND)?;

    let id: uuid::Uuid = current.get(0);
    let site_name = body.site_name.unwrap_or_else(|| current.get(1));
    let site_description = body.site_description.unwrap_or_else(|| current.get(2));
    let logo_url = body.logo_url.unwrap_or_else(|| current.get(3));
    let favicon_url = body.favicon_url.unwrap_or_else(|| current.get(4));
    let footer_text = body.footer_text.unwrap_or_else(|| current.get(5));
    let social_links = body.social_links.unwrap_or_else(|| current.get(6));
    let custom_head_html = body.custom_head_html.unwrap_or_else(|| current.get(7));

    let row = client.query_one(
        "UPDATE site_settings SET site_name=$1, site_description=$2, logo_url=$3,
         favicon_url=$4, footer_text=$5, social_links=$6, custom_head_html=$7, updated_at=NOW()
         WHERE id=$8
         RETURNING id, site_name, site_description, logo_url, favicon_url, footer_text,
                   social_links, custom_head_html, updated_at",
        &[&site_name, &site_description, &logo_url, &favicon_url, &footer_text, &social_links, &custom_head_html, &id],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SiteSettings {
        id: row.get(0),
        site_name: row.get(1),
        site_description: row.get(2),
        logo_url: row.get(3),
        favicon_url: row.get(4),
        footer_text: row.get(5),
        social_links: row.get(6),
        custom_head_html: row.get(7),
        updated_at: row.get(8),
    }))
}
