use axum::{
    Json, Router,
    routing::{get, post},
};
use reqwest::StatusCode;

use crate::event_sources::radarr;

async fn radarr_health() -> (StatusCode, Json<radarr::RadarrStatus>) {
    let status = radarr::get_status().await;
    if status.is_err() {
        eprintln!("Error fetching status: {:?}", status);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]));
    }

    (StatusCode::OK, Json(status.unwrap()))
}

pub fn get_router() -> Router {
    Router::new().route("/health", get(radarr_health))
}
