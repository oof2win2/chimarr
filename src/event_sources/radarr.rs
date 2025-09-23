use crate::config::app;
use anyhow::anyhow;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

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
    source: String,
    r#type: String,
    message: String,
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
