use axum::{
    routing::{get, post},
    Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use std::net::SocketAddr;
use std::sync::Arc;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    db: Pool<Sqlite>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:telemetry.db".into());
    
    // Create DB file if not exists
    if !std::path::Path::new("telemetry.db").exists() {
        std::fs::File::create("telemetry.db")?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS telemetry (
            id INTEGER PRIMARY KEY,
            received_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            payload TEXT
        )"
    )
    .execute(&pool)
    .await?;

    let state = Arc::new(AppState { db: pool });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/upload", post(upload_telemetry))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn upload_telemetry(
    State(state): State<Arc<AppState>>,
    body: String,
) -> impl IntoResponse {
    // In a real system, we would validate the auth token here.
    // We store the encrypted payload directly.
    
    match sqlx::query("INSERT INTO telemetry (payload) VALUES (?)")
        .bind(body)
        .execute(&state.db)
        .await
    {
        Ok(_) => (StatusCode::OK, "Uploaded"),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error")
        }
    }
}
