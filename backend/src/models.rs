use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Pages ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: String,
    pub meta_title: String,
    pub meta_description: String,
    pub tags: Vec<String>,
    pub is_visible: bool,
    pub is_pinned: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePage {
    pub title: String,
    pub slug: Option<String>,
    pub content: String,
    pub excerpt: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_visible: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePage {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub excerpt: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_visible: Option<bool>,
}

// ── Revisions ──

#[derive(Debug, Serialize, Deserialize)]
pub struct Revision {
    pub id: Uuid,
    pub page_id: Uuid,
    pub title: String,
    pub content: String,
    pub excerpt: String,
    pub meta_title: String,
    pub meta_description: String,
    pub tags: Vec<String>,
    pub revision_number: i32,
    pub created_at: DateTime<Utc>,
}

// ── Sliders ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Slider {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub image_url: String,
    pub link_url: String,
    pub is_visible: bool,
    pub is_pinned: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSlider {
    pub title: String,
    pub description: Option<String>,
    pub image_url: String,
    pub link_url: Option<String>,
    pub is_visible: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSlider {
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub link_url: Option<String>,
    pub is_visible: Option<bool>,
}

// ── Media ──

#[derive(Debug, Serialize, Deserialize)]
pub struct Media {
    pub id: Uuid,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TransformRequest {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub crop_x: Option<u32>,
    pub crop_y: Option<u32>,
    pub crop_width: Option<u32>,
    pub crop_height: Option<u32>,
}

// ── Auth ──

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub display_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub captcha_id: String,
    pub captcha_answer: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

// ── Captcha ──

#[derive(Debug, Serialize)]
pub struct CaptchaResponse {
    pub id: String,
    pub image: String,
}

// ── Analytics ──

#[derive(Debug, Serialize, Deserialize)]
pub struct PageVisit {
    pub id: Uuid,
    pub page_path: String,
    pub referrer: Option<String>,
    pub user_agent: Option<String>,
    pub ip_hash: String,
    pub visited_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TrackVisitRequest {
    pub page_path: String,
    pub referrer: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsSummary {
    pub total_visits: i64,
    pub unique_visitors: i64,
    pub total_pages: i64,
    pub visits_today: i64,
    pub visits_this_month: i64,
}

#[derive(Debug, Serialize)]
pub struct DailyStats {
    pub date: String,
    pub visits: i64,
    pub unique_visitors: i64,
}

#[derive(Debug, Serialize)]
pub struct MonthlyStats {
    pub month: String,
    pub visits: i64,
    pub unique_visitors: i64,
}

#[derive(Debug, Serialize)]
pub struct TopPage {
    pub page_path: String,
    pub visits: i64,
}

// ── Reorder ──

#[derive(Debug, Deserialize)]
pub struct ReorderItem {
    pub id: Uuid,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct ReorderRequest {
    pub items: Vec<ReorderItem>,
}

// ── Visibility / Pin Toggle ──

#[derive(Debug, Deserialize)]
pub struct ToggleRequest {
    pub value: bool,
}

// ── Site Settings ──

#[derive(Debug, Serialize, Deserialize)]
pub struct SiteSettings {
    pub id: Uuid,
    pub site_name: String,
    pub site_description: String,
    pub logo_url: String,
    pub favicon_url: String,
    pub footer_text: String,
    pub social_links: serde_json::Value,
    pub custom_head_html: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettings {
    pub site_name: Option<String>,
    pub site_description: Option<String>,
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
    pub footer_text: Option<String>,
    pub social_links: Option<serde_json::Value>,
    pub custom_head_html: Option<String>,
}

// ── Menus ──

#[derive(Debug, Serialize, Deserialize)]
pub struct Menu {
    pub id: Uuid,
    pub label: String,
    pub url: String,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub is_visible: bool,
    pub open_in_new_tab: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMenu {
    pub label: String,
    pub url: String,
    pub parent_id: Option<Uuid>,
    pub is_visible: Option<bool>,
    pub open_in_new_tab: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMenu {
    pub label: Option<String>,
    pub url: Option<String>,
    pub parent_id: Option<Uuid>,
    pub is_visible: Option<bool>,
    pub open_in_new_tab: Option<bool>,
}

// ── Redirects ──

#[derive(Debug, Serialize, Deserialize)]
pub struct Redirect {
    pub id: Uuid,
    pub from_path: String,
    pub to_path: String,
    pub status_code: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRedirect {
    pub from_path: String,
    pub to_path: String,
    pub status_code: Option<i32>,
}

// ── Search ──

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

// ── CSRF ──

#[derive(Debug, Serialize)]
pub struct CsrfTokenResponse {
    pub csrf_token: String,
}
