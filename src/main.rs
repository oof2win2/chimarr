use std::convert::Infallible;
use std::net::SocketAddr;

use anyhow::Result;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

mod config;
mod event_sources;

use event_sources::radarr;

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let status = radarr::get_status().await;
    if status.is_err() {
        eprintln!("Error fetching status: {:?}", status);
        return Ok(Response::new(Full::new(Bytes::from(
            "Error fetching status",
        ))));
    }

    let str = serde_json::to_string(&status.unwrap());
    if str.is_err() {
        return Ok(Response::new(Full::new(Bytes::from(
            "Error fetching status",
        ))));
    }

    Ok(Response::new(Full::new(Bytes::from(str.unwrap()))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    config::init_config("./config.json")?;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(hello))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
