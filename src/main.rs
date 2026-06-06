// Copyright 2026 Wayne Hong (h-alice) <contact@halice.art>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # EOMC datacenter MCP service
//!
//! A Model Context Protocol server that fetch data from upstream datacenter APIs
//! and provide them through MCP.
//!
//! ## Modes
//!
//! - **stdio (default).** A client spawns the binary as a subprocess and talks
//!   JSON-RPC over stdin/stdout. Suitable for local development.
//! - **Streamable HTTP (`--serve`).** Server mode served over axum at
//!   `POST/GET /mcp`, binding `BIND_ADDR` (default `0.0.0.0`) and
//!   `BIND_PORT` (default `8000`). Put TLS + auth in front via a reverse proxy.
//!
//! Both modes share the same tool logic, only bootstrap differs.

use std::io::IsTerminal;
use std::sync::Arc;

use rmcp::{
    transport::{
        stdio,
        streamable_http_server::{session::local::LocalSessionManager, StreamableHttpService},
    },
    ServiceExt,
};

mod appstate;
mod client;
mod config;
mod conventions;
mod server;
mod tools;

use appstate::AppState;
use client::ApiClient;
use config::Config;
use server::EomcServer;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Try to load `.env` file.
    dotenvy::dotenv().ok();

    // The log goes to stderr, BTW, this is the common pattern of
    // handling application logging.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env();
    let state = AppState::new(config);
    let client = Arc::new(ApiClient::new(state));

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--serve") {
        run_http(client).await
    } else {
        run_stdio(client).await
    }
}

/// Resolve the listen address from `BIND_ADDR` (default `0.0.0.0`) and
/// `BIND_PORT` (default `8000`), used by the `--serve` mode.
fn bind_address() -> String {
    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("BIND_PORT")
        .ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or(8000);
    format!("{addr}:{port}")
}

/// Serve over stdio: handshake on stdin/stdout, then block until the peer
/// disconnects (stdin EOF), at which point we shut down cleanly.
async fn run_stdio(client: Arc<ApiClient>) -> anyhow::Result<()> {
    tracing::info!("starting eomc-mcp server over stdio");
    let service = EomcServer::new(client).serve(stdio()).await?;
    service.waiting().await?;
    tracing::info!("eomc-mcp server shutting down");
    Ok(())
}

/// Serve over Streamable HTTP via axum.
///
/// A fresh [`EomcServer`] is built per session by the service factory,
/// the in-memory [`LocalSessionManager`] tracks `Mcp-Session-Id` across
/// a client's requests.
async fn run_http(client: Arc<ApiClient>) -> anyhow::Result<()> {
    let bind = bind_address();

    let service = StreamableHttpService::new(
        move || Ok(EomcServer::new(client.clone())),
        Arc::new(LocalSessionManager::default()),
        Default::default(),
    );
    let app = axum::Router::new().nest_service("/mcp", service);

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "eomc-mcp serving Streamable HTTP at POST/GET /mcp");
    // Shut down gracefully on Ctrl-C.
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// Resolves when SIGINT or (on Unix) SIGTERM is received.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("ctrl_c handler");
    };

    #[cfg(unix)]
    let sigterm = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c  => tracing::info!("received SIGINT"),
        _ = sigterm => tracing::info!("received SIGTERM"),
    }
}
