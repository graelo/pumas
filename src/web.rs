//! Web dashboard server using axum + ECharts.

use std::{path::PathBuf, sync::Arc};

use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    routing::get,
    Router,
};
use rust_embed::RustEmbed;
use serde_json::json;

use crate::{metrics, modules::soc::SocInfo};

/// Embedded web assets (compiled into the binary).
#[derive(RustEmbed)]
#[folder = "web/"]
struct Assets;

/// Shared state between metrics collection and HTTP handlers.
pub(crate) struct SharedMetricsState {
    pub(crate) metrics: std::sync::RwLock<Option<metrics::Metrics>>,
    pub(crate) soc_info: SocInfo,
    pub(crate) web_dir: Option<PathBuf>,
}

/// Run the axum HTTP server until shutdown.
pub(crate) async fn serve(
    state: Arc<SharedMetricsState>,
    listen_addr: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/metrics", get(metrics_handler))
        .route("/api/config", get(config_handler))
        .route("/assets/*path", get(assets_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    println!("Web dashboard listening on http://{listen_addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Serve index.html.
async fn index_handler(State(state): State<Arc<SharedMetricsState>>) -> Response<Body> {
    serve_file("index.html", &state).await
}

/// Serve static assets (JS, CSS, etc.).
async fn assets_handler(
    State(state): State<Arc<SharedMetricsState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response<Body> {
    serve_file(&path, &state).await
}

/// Try to read a file from web_dir first, then fall back to embedded assets.
async fn serve_file(filename: &str, state: &SharedMetricsState) -> Response<Body> {
    // Prefer web_dir override
    if let Some(ref web_dir) = state.web_dir {
        let file_path = web_dir.join(filename);
        if let Ok(data) = tokio::fs::read(&file_path).await {
            return ok_response(guess_mime(filename), data);
        }
    }

    // Fall back to embedded assets
    match Assets::get(filename) {
        Some(content) => ok_response(guess_mime(filename), content.data.to_vec()),
        None => not_found(),
    }
}

fn ok_response(mime: &str, data: Vec<u8>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", mime)
        .body(Body::from(data))
        .unwrap()
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("not found"))
        .unwrap()
}

fn json_response(body: String) -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap()
}

/// Return the latest metrics as JSON (with soc info).
async fn metrics_handler(
    State(state): State<Arc<SharedMetricsState>>,
) -> Response<Body> {
    let metrics_guard = state.metrics.read().unwrap();
    let body = json!({
        "soc": &state.soc_info,
        "metrics": &*metrics_guard,
    });
    json_response(serde_json::to_string(&body).unwrap())
}

/// Return soc info and dashboard config.
async fn config_handler(
    State(state): State<Arc<SharedMetricsState>>,
) -> Response<Body> {
    let body = json!({
        "soc": &state.soc_info,
    });
    json_response(serde_json::to_string(&body).unwrap())
}

fn guess_mime(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else {
        "application/octet-stream"
    }
}
