use axum::{
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};

mod config;
mod event_sources;
mod routers;

async fn health() -> (StatusCode, String) {
    (StatusCode::OK, "All services are healthy".to_owned())
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

#[tokio::main]
async fn main() {
    config::init_config("./config.json").unwrap();

    let host = config::app::server::host().unwrap();
    let port = config::app::server::port().unwrap();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/health", get(health))
        .nest("/radarr", routers::radarr::get_router())
        .fallback(handler_404);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind((host, port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
