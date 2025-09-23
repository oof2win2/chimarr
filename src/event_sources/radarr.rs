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

    let mut url = Url::parse(&radarr_url)?;
    url.query_pairs_mut().append_pair("apiKey", &radarr_apikey);
    url.path_segments_mut()
        .map_err(|_| anyhow!("Path cannot be a base"))?
        .push("api")
        .push("v3")
        .push(path);

    Ok(url)
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(dead_code)]
pub struct SingleStatus {
    pub source: String,
    pub r#type: String,
    pub message: String,
}
pub type RadarrStatus = Vec<SingleStatus>;

pub async fn get_status() -> anyhow::Result<RadarrStatus> {
    // Get the base Radarr URL with API key
    let url = get_radarr_url("health")?;

    let res = reqwest::get(url).await?;
    let body = res.text().await?;

    let body: RadarrStatus = serde_json::from_str(&body)?;

    Ok(body)
}

async fn poll_status(ctx: AppState) -> anyhow::Result<()> {
    let status = get_status().await?;

    let mut notification_manager = ctx.notifications.lock().unwrap();
    status
        .into_iter()
        .map(|notif| BareNotification {
            r#type: NotificationType::Info,
            message: notif.message,
        })
        .for_each(|notif| notification_manager.send_notification_sync(notif));

    Ok(())
}

async fn increment(ctx: AppState) -> () {
    let counter_value = {
        let mut counter = ctx.counter.lock().unwrap();
        *counter += 1;
        *counter
    };

    let notification = BareNotification {
        r#type: NotificationType::Info,
        message: format!("Counter is currently, {}", counter_value),
    };

    ctx.notifications
        .lock()
        .unwrap()
        .send_notification_sync(notification);
}

pub async fn enable(mut ctx: AppState) -> anyhow::Result<()> {
    let ctx_clone = ctx.clone();
    ctx.scheduler
        .add(Job::named("increment", "*/5 * * * * *", move || {
            let ctx = ctx_clone.clone();
            async move {
                println!("running every 5s");
                increment(ctx).await;
            }
        }));

    Ok(())
}

pub async fn disable(mut ctx: AppState) -> anyhow::Result<()> {
    ctx.scheduler.cancel_by_name("increment");
    Ok(())
}
