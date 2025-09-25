use crate::{
    AppState,
    config::app,
    notifications::{BareNotification, NotificationType},
};
use anyhow::anyhow;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tokio_cron::Job;

fn get_radarr_url(path: &str) -> anyhow::Result<Url> {
    // Using the new hierarchical access
    let radarr_url = app::radarr::url()?;
    let radarr_apikey = app::radarr::apikey()?;

    let segments = path.split("/");

    let mut url = Url::parse(&radarr_url)?;
    url.query_pairs_mut().append_pair("apiKey", &radarr_apikey);
    let mut path_segments = url
        .path_segments_mut()
        .map_err(|_| anyhow!("Path cannot be a base"))?;
    // add the API v3 requirement
    path_segments.push("api").push("v3");
    for segment in segments {
        if !segment.is_empty() {
            path_segments.push(segment);
        }
    }
    drop(path_segments);

    Ok(url)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum RadarrServiceStatusType {
    WARNING,
    INFO,
    ERROR,
}
impl Into<NotificationType> for RadarrServiceStatusType {
    fn into(self) -> NotificationType {
        match self {
            RadarrServiceStatusType::WARNING => NotificationType::Warning,
            RadarrServiceStatusType::INFO => NotificationType::Info,
            RadarrServiceStatusType::ERROR => NotificationType::Error,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(dead_code)]
pub struct SingleStatus {
    pub source: String,
    #[serde(rename(deserialize = "type"))]
    pub status_type: RadarrServiceStatusType,
    pub message: String,
}
pub type RadarrServiceStatus = Vec<SingleStatus>;

#[derive(Deserialize)]
pub struct DownloadClientTestValidationFailures {
    #[serde(rename(deserialize = "propertyName"))]
    pub property_name: String,
    #[serde(rename(deserialize = "errorMessage"))]
    pub error_message: String,
    pub severity: String,
}

#[derive(Deserialize)]
pub struct SingleDownloadClientTest {
    pub id: u32,
    #[serde(rename(deserialize = "isValid"))]
    pub is_valid: bool,
    #[serde(rename(deserialize = "validationFailures"))]
    pub validation_failures: DownloadClientTestValidationFailures,
}

#[derive(Debug, Serialize)]
pub struct RadarrStatus {
    pub service_status: RadarrServiceStatus,
}

pub async fn get_status() -> anyhow::Result<RadarrStatus> {
    let radarr_service_status = reqwest::get(get_radarr_url("/health")?)
        .await?
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get response text: {}", e))
        .and_then(|status| {
            serde_json::from_str(&status)
                .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))
        })?;

    Ok(RadarrStatus {
        service_status: radarr_service_status,
    })
}

async fn poll_status(ctx: AppState) -> anyhow::Result<()> {
    let status = get_status().await?;

    let notifications_to_send = status
        .service_status
        .into_iter()
        .map(|notif| BareNotification {
            notification_type: notif.status_type.into(),
            message: notif.message,
        });

    let mut notification_manager = ctx.notifications.lock().await;
    for notif in notifications_to_send {
        notification_manager.send_notification(notif).await;
    }

    Ok(())
}

pub async fn enable(mut ctx: AppState) -> anyhow::Result<()> {
    let ctx_clone = ctx.clone();
    ctx.scheduler
        .add(Job::named("increment", "*/10 * * * * *", move || {
            let ctx = ctx_clone.clone();
            async move {
                println!("running every 10s");
                let res = poll_status(ctx).await;
                if res.is_err() {
                    let err = res.err().unwrap();
                    eprintln!("Error fetching status: {}", err);
                }
            }
        }));

    Ok(())
}

pub async fn disable(mut ctx: AppState) -> anyhow::Result<()> {
    ctx.scheduler.cancel_by_name("increment");
    Ok(())
}
