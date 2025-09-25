use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use std::sync::{Arc, Mutex};
use tokio_cron::Scheduler;

use crate::notifications::NotificationManager;

mod config;
mod dispatchers;
mod modules;
mod notifications;
mod routers;

async fn health(State(ctx): State<AppState>) -> (StatusCode, String) {
    let count = ctx.counter.lock().unwrap();
    (
        StatusCode::OK,
        format!("All services are healthy. Count is {}", count).to_owned(),
    )
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

#[derive(Clone)]
struct AppState {
    scheduler: Scheduler,
    counter: Arc<Mutex<i32>>,
    notifications: Arc<Mutex<NotificationManager>>,
}

#[tokio::main]
async fn main() {
    config::init_config("./config.json").unwrap();

    let notification_manager = NotificationManager::new().await;

    let state = AppState {
        scheduler: Scheduler::utc(),
        counter: Arc::new(Mutex::new(1)),
        notifications: Arc::new(Mutex::new(notification_manager)),
    };

    let host = config::app::server::host().unwrap();
    let port = config::app::server::port().unwrap();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/health", get(health))
        .nest("/radarr", routers::radarr::get_router())
        .fallback(handler_404)
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind((host, port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
