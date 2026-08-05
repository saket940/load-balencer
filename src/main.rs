use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    response::Response,
    routing::any,
    Router,
};
use reqwest::Client;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::time::{sleep, Duration};

struct Backend {
    url: &'static str,
    healthy: AtomicBool,
}

struct AppState {
    client: Client,
    backends: Vec<Backend>,
    counter: AtomicUsize,
}

async fn health_check(state: Arc<AppState>) {
    loop {
        println!("Running health check...");

        for backend in &state.backends {
            let health_url = format!(
                "{}/api/internships",
                backend.url.trim_end_matches('/')
            );

            match state.client.get(&health_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    backend.healthy.store(true, Ordering::Relaxed);
                    println!("✅ {} is UP", backend.url);
                }
                Ok(resp) => {
                    backend.healthy.store(false, Ordering::Relaxed);
                    println!("❌ {} returned {}", health_url, resp.status());
                }
                Err(err) => {
                    backend.healthy.store(false, Ordering::Relaxed);
                    println!("❌ {} is DOWN ({})", health_url, err);
                }
            }
        }

        // Wait 270 seconds (4 minutes 30 seconds)
        sleep(Duration::from_secs(270)).await;
    }
}

// Round-robin proxy
async fn proxy(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let total = state.backends.len();

    for _ in 0..total {
        let index = state.counter.fetch_add(1, Ordering::Relaxed) % total;

        let backend = &state.backends[index];

        if !backend.healthy.load(Ordering::Relaxed) {
            continue;
        }

        let path = req
            .uri()
            .path_and_query()
            .map(|x| x.as_str())
            .unwrap_or("/");

        let url = format!("{}{}", backend.url, path);

        match state.client.get(url).send().await {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.bytes().await.unwrap_or_default();

                return Ok(Response::builder()
                    .status(status)
                    .body(Body::from(body))
                    .unwrap());
            }
            Err(_) => {
                backend.healthy.store(false, Ordering::Relaxed);
            }
        }
    }

    Err(StatusCode::SERVICE_UNAVAILABLE)
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        client: Client::new(),
        backends: vec![
            Backend {
                url: "https://web-update-alert.onrender.com",
                healthy: AtomicBool::new(true),
            },
          
        ],
        counter: AtomicUsize::new(0),
    });

    // Start background health checker
    let checker = state.clone();
    tokio::spawn(async move {
        health_check(checker).await;
    });

    let app = Router::new()
        .route("/{*path}", any(proxy))
        .with_state(state);
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap();
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    println!("Load Balancer running on http://0.0.0.0:8080");

    axum::serve(listener, app).await.unwrap();
}
