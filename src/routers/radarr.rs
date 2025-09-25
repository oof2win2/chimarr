use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use reqwest::StatusCode;

use crate::{AppState, modules::radarr};

#[axum::debug_handler]
async fn radarr_health(
    State(ctx): State<AppState>,
) -> (StatusCode, Json<Option<radarr::RadarrStatus>>) {
    let status = radarr::get_status().await;
    if status.is_err() {
        eprintln!("Error fetching status: {:?}", status);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
    }

    (StatusCode::OK, Json(Some(status.unwrap())))
}

async fn enable_radarr(State(ctx): State<AppState>) -> StatusCode {
    let res = radarr::enable(ctx).await;
    if res.is_ok() {
        StatusCode::OK
    } else {
        let error = res.err().unwrap();
        eprintln!("Error enabling radarr {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

async fn disable_radarr(State(ctx): State<AppState>) -> StatusCode {
    let res = radarr::disable(ctx).await;
    if res.is_ok() {
        StatusCode::OK
    } else {
        let error = res.err().unwrap();
        eprintln!("Error disabling radarr {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub fn get_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(radarr_health))
        .route("/enable", post(enable_radarr))
        .route("/disable", post(disable_radarr))
}
