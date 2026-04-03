use axum::{
    extract::State,
    http::{StatusCode, HeaderMap},
    Json,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::Utc;
use uuid::Uuid;
use std::sync::Arc;

use crate::AppState;
use crate::models::*;

/// Generate a simple math CAPTCHA (SVG-based, no external deps)
pub async fn generate_captcha(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CaptchaResponse>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Clean up expired captchas
    client.execute(
        "DELETE FROM captchas WHERE created_at < NOW() - INTERVAL '10 minutes'",
        &[],
    ).await.ok();

    let a: u32 = (rand::random::<u32>() % 20) + 1;
    let b: u32 = (rand::random::<u32>() % 20) + 1;
    let answer = a + b;
    let captcha_id = Uuid::new_v4().to_string();

    client.execute(
        "INSERT INTO captchas (id, answer) VALUES ($1, $2)",
        &[&captcha_id, &answer.to_string()],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate SVG captcha
    let svg = generate_captcha_svg(a, b);
    let image_data = format!("data:image/svg+xml;base64,{}", base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD, svg.as_bytes()
    ));

    Ok(Json(CaptchaResponse {
        id: captcha_id,
        image: image_data,
    }))
}

fn generate_captcha_svg(a: u32, b: u32) -> String {
    let text = format!("{} + {} = ?", a, b);
    // Add visual noise with random lines and distortion
    let mut lines = String::new();
    for i in 0..5 {
        let x1 = (i * 37 + 13) % 200;
        let y1 = (i * 23 + 7) % 60;
        let x2 = (i * 41 + 29) % 200;
        let y2 = (i * 31 + 17) % 60;
        lines.push_str(&format!(
            r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#999" stroke-width="1" opacity="0.5"/>"##,
            x1, y1, x2, y2
        ));
    }

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="60" viewBox="0 0 200 60">
  <rect width="200" height="60" fill="#f0f0f0" rx="4"/>
  {}
  <text x="100" y="38" font-family="monospace" font-size="24" font-weight="bold"
        text-anchor="middle" fill="#333" transform="rotate(-3, 100, 30)">{}</text>
  <text x="102" y="36" font-family="monospace" font-size="24" font-weight="bold"
        text-anchor="middle" fill="rgba(0,0,0,0.1)">{}</text>
</svg>"##,
        lines, text, text
    )
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    let err = |msg: &str| -> (StatusCode, Json<serde_json::Value>) {
        (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": msg})))
    };

    // Validate inputs
    if body.username.is_empty() || body.password.is_empty() {
        return Err(err("Username and password are required"));
    }
    if body.captcha_id.is_empty() || body.captcha_answer.is_empty() {
        return Err(err("Captcha is required"));
    }

    let client = state.pool.get().await.map_err(|_| err("Database error"))?;

    // Verify captcha
    let captcha_row = client.query_opt(
        "DELETE FROM captchas WHERE id = $1 AND created_at > NOW() - INTERVAL '10 minutes' RETURNING answer",
        &[&body.captcha_id],
    ).await.map_err(|_| err("Database error"))?;

    match captcha_row {
        Some(row) => {
            let stored_answer: String = row.get(0);
            if stored_answer != body.captcha_answer {
                return Err(err("Invalid captcha answer"));
            }
        }
        None => return Err(err("Captcha expired or invalid")),
    }

    // Verify user credentials
    let user_row = client.query_opt(
        "SELECT id, username, password_hash, display_name, role FROM users WHERE username = $1",
        &[&body.username],
    ).await.map_err(|_| err("Database error"))?;

    let user_row = user_row.ok_or_else(|| err("Invalid credentials"))?;

    let user_id: Uuid = user_row.get(0);
    let username: String = user_row.get(1);
    let password_hash: String = user_row.get(2);
    let display_name: String = user_row.get(3);
    let role: String = user_row.get(4);

    let password_valid = bcrypt::verify(&body.password, &password_hash)
        .map_err(|_| err("Invalid credentials"))?;

    if !password_valid {
        return Err(err("Invalid credentials"));
    }

    // Generate JWT
    let expiry_hours: i64 = std::env::var("JWT_EXPIRY_HOURS")
        .unwrap_or_else(|_| "24".to_string())
        .parse()
        .unwrap_or(24);

    let claims = Claims {
        sub: username.clone(),
        user_id: user_id.to_string(),
        role: role.clone(),
        exp: (Utc::now() + chrono::Duration::hours(expiry_hours)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    ).map_err(|_| err("Token generation failed"))?;

    Ok(Json(LoginResponse {
        token,
        user: UserInfo {
            id: user_id,
            username,
            display_name,
            role,
        },
    }))
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
) -> Result<Json<UserInfo>, StatusCode> {
    let claims = req.extensions().get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = client.query_one(
        "SELECT id, username, display_name, role FROM users WHERE id = $1",
        &[&Uuid::parse_str(&claims.user_id).map_err(|_| StatusCode::UNAUTHORIZED)?],
    ).await.map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(UserInfo {
        id: row.get(0),
        username: row.get(1),
        display_name: row.get(2),
        role: row.get(3),
    }))
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let claims = req.extensions().get::<Claims>().cloned()
        .ok_or((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Unauthorized"}))));
    let claims = claims?;

    // Read body manually since we already consumed extensions
    let body = axum::body::to_bytes(req.into_body(), 1024 * 16).await
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid body"}))))?;
    let body: ChangePasswordRequest = serde_json::from_slice(&body)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid JSON"}))))?;

    if body.new_password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Password must be at least 8 characters"}))));
    }

    let client = state.pool.get().await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Database error"}))))?;

    let user_id = Uuid::parse_str(&claims.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid user ID"}))))?;

    let row = client.query_one(
        "SELECT password_hash FROM users WHERE id = $1",
        &[&user_id],
    ).await.map_err(|_| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "User not found"}))))?;

    let current_hash: String = row.get(0);
    let valid = bcrypt::verify(&body.current_password, &current_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Verification error"}))))?;

    if !valid {
        return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Current password is incorrect"}))));
    }

    let new_hash = bcrypt::hash(&body.new_password, 12)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Hashing error"}))))?;

    client.execute(
        "UPDATE users SET password_hash = $1 WHERE id = $2",
        &[&new_hash, &user_id],
    ).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Update failed"}))))?;

    Ok(Json(serde_json::json!({"message": "Password changed successfully"})))
}

pub async fn get_csrf_token(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<CsrfTokenResponse>, StatusCode> {
    let client = state.pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Clean expired tokens
    client.execute("DELETE FROM csrf_tokens WHERE expires_at < NOW()", &[])
        .await.ok();

    let token = Uuid::new_v4().to_string();

    client.execute(
        "INSERT INTO csrf_tokens (token) VALUES ($1)",
        &[&token],
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let _ = headers; // Available for origin checking if needed

    Ok(Json(CsrfTokenResponse { csrf_token: token }))
}
