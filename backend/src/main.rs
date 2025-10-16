use std::net::SocketAddr;

use axum::serve;
use pdf_editor_backend::{build_router, init_tracing, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let state = AppState::default();
    let app = build_router(state);

    let addr: SocketAddr = ([127, 0, 0, 1], 8787).into();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{}", listener.local_addr()?);
    serve(listener, app).await?;

    Ok(())
}
