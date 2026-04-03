mod handlers;
mod middleware;
mod models;
mod migrations;

use axum::{
    Router,
    routing::{get, post, put, delete},
    extract::DefaultBodyLimit,
};
use deadpool_postgres::{Config, Pool, Runtime};
use std::sync::Arc;
use tokio_postgres::NoTls;
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;
use tower::Service;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::handlers::*;
use crate::middleware::csrf::CsrfLayer;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub jwt_secret: String,
    pub upload_dir: String,
    pub max_upload_size: usize,
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load .env if exists
    if let Ok(contents) = std::fs::read_to_string(".env") {
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                // SAFETY: called during single-threaded startup before spawning tasks
                unsafe { std::env::set_var(key.trim(), value.trim()); }
            }
        }
    }

    let mut cfg = Config::new();
    cfg.host = Some(env_or("DATABASE_HOST", "localhost"));
    cfg.port = Some(env_or("DATABASE_PORT", "5432").parse().unwrap_or(5432));
    cfg.dbname = Some(env_or("DATABASE_NAME", "cms"));
    cfg.user = Some(env_or("DATABASE_USER", "cms_user"));
    cfg.password = Some(env_or("DATABASE_PASSWORD", "password"));

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create database pool");

    // Run migrations
    {
        let client = pool.get().await.expect("Failed to get DB connection for migrations");
        migrations::run_migrations(&client).await.expect("Failed to run migrations");
        tracing::info!("Database migrations completed");
    }

    let upload_dir = env_or("UPLOAD_DIR", "uploads");
    std::fs::create_dir_all(&upload_dir).ok();

    let max_upload_size: usize = env_or("MAX_UPLOAD_SIZE_MB", "10")
        .parse::<usize>()
        .unwrap_or(10) * 1024 * 1024;

    let state = AppState {
        pool,
        jwt_secret: env_or("JWT_SECRET", "default_dev_secret_change_in_production"),
        upload_dir: upload_dir.clone(),
        max_upload_size,
    };

    let public_api = Router::new()
        .route("/api/pages", get(pages::list_public_pages))
        .route("/api/pages/{slug}", get(pages::get_public_page))
        .route("/api/search", get(pages::search_pages))
        .route("/api/tags", get(pages::list_tags))
        .route("/api/tags/{tag}", get(pages::pages_by_tag))
        .route("/api/sliders", get(sliders::list_public_sliders))
        .route("/api/analytics/track", post(analytics::track_visit))
        .route("/api/captcha", get(auth::generate_captcha))
        .route("/api/auth/login", post(auth::login))
        .route("/api/csrf-token", get(auth::get_csrf_token))
        .route("/api/site-settings", get(settings::get_public_settings));

    let admin_api = Router::new()
        .route("/api/admin/pages", get(pages::admin_list_pages))
        .route("/api/admin/pages", post(pages::create_page))
        .route("/api/admin/pages/{id}", get(pages::admin_get_page))
        .route("/api/admin/pages/{id}", put(pages::update_page))
        .route("/api/admin/pages/{id}", delete(pages::delete_page))
        .route("/api/admin/pages/{id}/visibility", put(pages::toggle_visibility))
        .route("/api/admin/pages/{id}/pin", put(pages::toggle_pin))
        .route("/api/admin/pages/reorder", put(pages::reorder_pages))
        .route("/api/admin/pages/{id}/revisions", get(pages::list_revisions))
        .route("/api/admin/pages/{id}/revisions/{rev_id}", get(pages::get_revision))
        .route("/api/admin/pages/{id}/revisions/{rev_id}/restore", post(pages::restore_revision))
        .route("/api/admin/sliders", get(sliders::admin_list_sliders))
        .route("/api/admin/sliders", post(sliders::create_slider))
        .route("/api/admin/sliders/{id}", put(sliders::update_slider))
        .route("/api/admin/sliders/{id}", delete(sliders::delete_slider))
        .route("/api/admin/sliders/{id}/visibility", put(sliders::toggle_visibility))
        .route("/api/admin/sliders/{id}/pin", put(sliders::toggle_pin))
        .route("/api/admin/sliders/reorder", put(sliders::reorder_sliders))
        .route("/api/admin/media", get(media::list_media))
        .route("/api/admin/media/upload", post(media::upload_media))
        .route("/api/admin/media/{id}", delete(media::delete_media))
        .route("/api/admin/media/{id}/transform", post(media::transform_media))
        .route("/api/admin/analytics/summary", get(analytics::get_summary))
        .route("/api/admin/analytics/daily", get(analytics::get_daily))
        .route("/api/admin/analytics/monthly", get(analytics::get_monthly))
        .route("/api/admin/analytics/top-pages", get(analytics::get_top_pages))
        .route("/api/admin/settings", get(settings::get_settings))
        .route("/api/admin/settings", put(settings::update_settings))
        .route("/api/admin/auth/me", get(auth::me))
        .route("/api/admin/auth/change-password", post(auth::change_password))
        .route("/api/admin/menus", get(menus::list_menus))
        .route("/api/admin/menus", post(menus::create_menu))
        .route("/api/admin/menus/{id}", put(menus::update_menu))
        .route("/api/admin/menus/{id}", delete(menus::delete_menu))
        .route("/api/admin/menus/reorder", put(menus::reorder_menus))
        .route("/api/admin/redirects", get(redirects::list_redirects))
        .route("/api/admin/redirects", post(redirects::create_redirect))
        .route("/api/admin/redirects/{id}", delete(redirects::delete_redirect))
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(state.clone()),
            crate::middleware::auth::require_auth,
        ));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(public_api)
        .merge(admin_api)
        .nest_service("/uploads", ServeDir::new(&upload_dir))
        .layer(CsrfLayer)
        .layer(cors)
        .layer(DefaultBodyLimit::max(max_upload_size))
        .with_state(Arc::new(state.clone()));

    let host = env_or("SERVER_HOST", "0.0.0.0");
    let port: u16 = env_or("SERVER_PORT", "3000").parse().unwrap_or(3000);
    let addr = format!("{}:{}", host, port);

    let ssl_enabled = env_or("SSL_ENABLED", "false") == "true";

    if ssl_enabled {
        let cert_path = env_or("SSL_CERT_PATH", "certs/cert.pem");
        let key_path = env_or("SSL_KEY_PATH", "certs/key.pem");

        let cert_file = std::fs::File::open(&cert_path)
            .expect(&format!("Failed to open cert file: {}", cert_path));
        let key_file = std::fs::File::open(&key_path)
            .expect(&format!("Failed to open key file: {}", key_path));

        let certs = rustls_pemfile::certs(&mut std::io::BufReader::new(cert_file))
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to parse certificates");
        let key = rustls_pemfile::private_key(&mut std::io::BufReader::new(key_file))
            .expect("Failed to read private key")
            .expect("No private key found");

        let tls_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .expect("Failed to build TLS config");

        let tls_acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls_config));
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        tracing::info!("CMS server running on https://{}", addr);

        loop {
            let (stream, _addr) = listener.accept().await.unwrap();
            let acceptor = tls_acceptor.clone();
            let app = app.clone();

            tokio::spawn(async move {
                if let Ok(tls_stream) = acceptor.accept(stream).await {
                    let io = hyper_util::rt::TokioIo::new(tls_stream);
                    let service = hyper::service::service_fn(move |req| {
                        let app = app.clone();
                        async move {
                            app.into_service().call(req).await
                        }
                    });
                    if let Err(err) = hyper_util::server::conn::auto::Builder::new(
                        hyper_util::rt::TokioExecutor::new(),
                    )
                    .serve_connection(io, service)
                    .await
                    {
                        tracing::error!("Error serving TLS connection: {}", err);
                    }
                }
            });
        }
    } else {
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        tracing::info!("CMS server running on http://{}", addr);
        axum::serve(listener, app).await.unwrap();
    }
}
