use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode, Uri},
    response::IntoResponse,
    routing::{any, get_service},
    Router,
};
use http_body_util::BodyExt;
use reqwest::Client;
use std::{net::SocketAddr, sync::Arc};
use axum::extract::State;
use tokio::signal;
use tower_http::services::ServeDir;

static BACKEND_BASE: &str = "http://127.0.0.1:8080";

#[tokio::main]
async fn main() {
    let client = Arc::new(Client::new());

    let app = Router::new()
        .nest_service("/", get_service(ServeDir::new("public")))
        .route("/api/*path", any(proxy_api))
        .with_state(client);

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    println!("🌐 Web server running on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    println!("\nShutting down…");
}

async fn proxy_api(State(client): State<Arc<Client>>, mut req: Request) -> impl IntoResponse {
    let method: Method = req.method().clone();
    let uri: Uri = req.uri().clone();

    let path_q = uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or(uri.path());
    let url = format!("{BACKEND_BASE}{path_q}");

    let out_bytes = match req.body_mut().collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            eprintln!("Failed to read request body: {e}");
            return (StatusCode::BAD_REQUEST, "Invalid body".to_string()).into_response();
        }
    };

    println!("➡️  Proxying {} {}", method, path_q);
    if !out_bytes.is_empty() {
        if let Ok(s) = std::str::from_utf8(&out_bytes) {
            println!("   Body out: {s}");
        } else {
            println!("   Body out: <{} bytes>", out_bytes.len());
        }
    }

    let mut rb = client.request(method, &url).body(out_bytes);

    for (name, value) in req.headers().iter() {
        if name != &http::header::HOST {
            rb = rb.header(name, value);
        }
    }

    // Отправляем
    let resp = match rb.send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Backend request error: {e}");
            return (
                StatusCode::BAD_GATEWAY,
                format!("Proxy error: {e}"),
            )
                .into_response();
        }
    };

    let status = resp.status();
    let mut headers = http::HeaderMap::new();
    for (name, value) in resp.headers().iter() {
        headers.insert(name.clone(), value.clone());
    }
    let in_bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to read backend response body: {e}");
            Default::default()
        }
    };

    println!("⬅️  Response {} for {}", status, path_q);
    if !in_bytes.is_empty() {
        if let Ok(s) = std::str::from_utf8(&in_bytes) {
            println!("   Body in: {s}");
        } else {
            println!("   Body in: <{} bytes>", in_bytes.len());
        }
    }

    let mut builder = axum::response::Response::builder().status(status);
    if let Some(hm) = builder.headers_mut() {
        for (k, v) in headers.iter() {
            hm.insert(k, v.clone());
        }
    }
    match builder.body(Body::from(in_bytes)) {
        Ok(resp) => resp,
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Response build error").into_response(),
    }
}