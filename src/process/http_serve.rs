use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use std::{fs, io, net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::services::ServeDir;
use tracing::{info, warn};

#[derive(Debug)]
struct HttpServeState {
    path: PathBuf,
}

pub async fn process_http_serve(path: PathBuf, port: u16) -> Result<()> {
    info!("Serving {:?} on port {}", path, port);
    let state = HttpServeState { path: path.clone() };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let router = Router::new()
        .nest_service("/tower", ServeDir::new(path))
        .route("/*path", get(file_handler))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

async fn file_handler(
    State(state): State<Arc<HttpServeState>>,
    Path(path): Path<String>,
) -> (StatusCode, String) {
    let p = std::path::Path::new(&state.path).join(path);
    info!("Reading file {:?}", p);

    if !p.exists() {
        return (
            StatusCode::NOT_FOUND,
            format!("File {} not found", p.display()),
        );
    }

    if p.is_dir() {
        match dirlist(p) {
            Ok(content) => {
                let out = format_html_list(content);
                return (StatusCode::OK, out);
            }
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
            }
        }
    }

    match tokio::fs::read_to_string(p).await {
        Ok(content) => {
            info!("Read {} bytes", content.len());
            (StatusCode::OK, content)
        }
        Err(e) => {
            warn!("Error reading file: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    }
}

fn dirlist(p: PathBuf) -> Result<Vec<String>, io::Error> {
    let e_result = fs::read_dir(p);
    let entries = match e_result {
        Ok(entries) => entries,
        Err(e) => return Err(e),
    };

    let mut file_list: Vec<String> = Vec::new();
    for entry in entries {
        let entry_result = entry;
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let file_name = match entry.file_name().into_string() {
            Ok(file_name) => file_name,
            Err(_) => continue,
        };
        file_list.push(file_name);
    }

    Ok(file_list)
}

fn format_html_list(items: Vec<String>) -> String {
    let mut html_list = String::new();
    for item in items {
        let tmp_i = format!("{}\n", item);
        html_list.push_str(tmp_i.as_str());
    }
    html_list
}
