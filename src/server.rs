use crate::channels::Channel;
use crate::config::{EndpointConfig, ServerConfig};
use crate::handler;
use bytes::Bytes;
use http::Response;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn run(server_config: ServerConfig, endpoints: Vec<EndpointConfig>, channels: Vec<Box<dyn Channel>>) {
    let endpoints = Arc::new(endpoints);
    let channels = Arc::new(channels);

    let addr = format!("{}:{}", server_config.host, server_config.port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {addr}: {e}"));
    log::info!("listening on {addr}");

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let io = hyper_util::rt::TokioIo::new(stream);
        let endpoints = endpoints.clone();
        let channels = channels.clone();

        tokio::spawn(async move {
            let service = service_fn(move |req| {
                let endpoints = endpoints.clone();
                let channels = channels.clone();
                async move {
                    let (parts, body) = req.into_parts();
                    log::info!("{} {} {}", parts.method, parts.uri.path(), parts.uri.query().unwrap_or(""));
                    let body_bytes = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|c| c.to_bytes())
                        .unwrap_or_default();
                    let req = http::Request::from_parts(parts, body_bytes);

                    let (status, body) = handler::handle_request(&req, &endpoints, &channels).await;
                    let response = Response::builder()
                        .status(status)
                        .header("content-type", "application/json; charset=utf-8")
                        .body(Full::new(Bytes::from(body)))
                        .unwrap();
                    Ok::<_, Infallible>(response)
                }
            });

            if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                log::error!("connection error: {e}");
            }
        });
    }
}
