use axum::{
    extract::{Query, State},
    http::{StatusCode, HeaderMap},
    Json,
};
use sha2::{Sha256, Digest};
use std::sync::Arc;

use crate::AppState;
use crate::models::*;

pub async fn track_visit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<TrackVisitRequest>,
) -> Result<StatusCode, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Hash the IP for privacy
    let ip = headers.get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let mut hasher = Sha256::new();
    hasher.update(ip.as_bytes());
    hasher.update(b"cms_salt_2024");
    let ip_hash = hex::encode(hasher.finalize());

    let user_agent = headers.get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let page_path = html_escape::encode_text(&body.page_path).to_string();
    let referrer = body.referrer.as_deref().map(|r| html_escape::encode_text(r).to_string());

    client.execute(
        "INSERT INTO page_visits (page_path, referrer, user_agent, ip_hash)
         VALUES ($1, $2, $3, $4)",
        &[&page_path, &referrer, &user_agent, &ip_hash],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_summary(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AnalyticsSummary>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total: i64 = client.query_one("SELECT COUNT(*) FROM page_visits", &[])
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.get(0);

    let unique: i64 = client.query_one("SELECT COUNT(DISTINCT ip_hash) FROM page_visits", &[])
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.get(0);

    let pages: i64 = client.query_one("SELECT COUNT(*) FROM pages", &[])
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.get(0);

    let today: i64 = client.query_one(
        "SELECT COUNT(*) FROM page_visits WHERE visited_at::date = CURRENT_DATE", &[],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.get(0);

    let this_month: i64 = client.query_one(
        "SELECT COUNT(*) FROM page_visits WHERE date_trunc('month', visited_at) = date_trunc('month', CURRENT_DATE)", &[],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.get(0);

    Ok(Json(AnalyticsSummary {
        total_visits: total,
        unique_visitors: unique,
        total_pages: pages,
        visits_today: today,
        visits_this_month: this_month,
    }))
}

pub async fn get_daily(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<DailyStats>>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let days: i64 = params.get("days").and_then(|d| d.parse().ok()).unwrap_or(30).min(365);

    let rows = client.query(
        "SELECT visited_at::date as day, COUNT(*) as visits, COUNT(DISTINCT ip_hash) as unique_visitors
         FROM page_visits
         WHERE visited_at >= CURRENT_DATE - ($1 || ' days')::interval
         GROUP BY day ORDER BY day DESC",
        &[&days.to_string()],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stats: Vec<DailyStats> = rows.iter().map(|row| {
        let date: chrono::NaiveDate = row.get(0);
        DailyStats {
            date: date.to_string(),
            visits: row.get(1),
            unique_visitors: row.get(2),
        }
    }).collect();

    Ok(Json(stats))
}

pub async fn get_monthly(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<MonthlyStats>>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let months: i64 = params.get("months").and_then(|m| m.parse().ok()).unwrap_or(12).min(60);

    let rows = client.query(
        "SELECT to_char(date_trunc('month', visited_at), 'YYYY-MM') as month,
                COUNT(*) as visits, COUNT(DISTINCT ip_hash) as unique_visitors
         FROM page_visits
         WHERE visited_at >= CURRENT_DATE - ($1 || ' months')::interval
         GROUP BY month ORDER BY month DESC",
        &[&months.to_string()],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stats: Vec<MonthlyStats> = rows.iter().map(|row| MonthlyStats {
        month: row.get(0),
        visits: row.get(1),
        unique_visitors: row.get(2),
    }).collect();

    Ok(Json(stats))
}

pub async fn get_top_pages(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<TopPage>>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let limit: i64 = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(20).min(100);

    let rows = client.query(
        "SELECT page_path, COUNT(*) as visits FROM page_visits
         GROUP BY page_path ORDER BY visits DESC LIMIT $1",
        &[&limit],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let pages: Vec<TopPage> = rows.iter().map(|row| TopPage {
        page_path: row.get(0),
        visits: row.get(1),
    }).collect();

    Ok(Json(pages))
}
