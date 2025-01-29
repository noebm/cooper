use askama::Template;
use axum::{
    extract::State,
    http::{StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use clap::Parser;
use std::{net::SocketAddr, path::PathBuf};
use tower_http::services::ServeDir;

#[derive(Parser)]
#[command(version, about)]
struct Options {
    /// Directory to serve. Defaults to current directory.
    #[arg(short, long)]
    serve_dir: Option<PathBuf>,

    #[arg(short, long, default_value_t = 8000)]
    port: u16,
}

#[derive(Template)]
#[template(path = "directory.html")]
struct DirectoryTemplate {
    directory_name: String,
    items: Vec<(PathBuf, String)>,
}

#[tokio::main]
async fn main() {
    let options = Options::parse();

    let serve_dir = options
        .serve_dir
        .unwrap_or_else(|| std::env::current_dir().expect("Could not retrieve current directory!"));

    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), options.port);

    println!(
        "Serving directory {} on http://{}",
        serve_dir.display(),
        addr
    );

    let directory = get(directory).with_state(serve_dir.clone());

    let app = Router::new().fallback_service(
        ServeDir::new(serve_dir)
            .append_index_html_on_directories(false)
            .not_found_service(directory),
    );

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn directory(State(root): State<PathBuf>, uri: Uri) -> Result<Response, Response> {
    println!("URI path {}", uri.path());
    println!("root {}", root.display());

    let relative_uri_path = uri
        .path()
        .strip_prefix("/")
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Invalid URI prefix").into_response())?;

    let directory = root.clone().join(relative_uri_path);

    println!("Reading {}", directory.display());

    let paths = std::fs::read_dir(directory)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;
    let mut entries = Vec::new();
    for entry_ in paths {
        let entry = match entry_ {
            Ok(entry) => entry,
            Err(msg) => {
                eprintln!("Error accessing directory entry: {}", msg);
                continue;
            }
        };

        let path = match entry.path().strip_prefix(&root) {
            Ok(path) => path.to_owned(),
            Err(msg) => {
                eprintln!("Error accessing directory entry: {}", msg);
                continue;
            }
        };

        let Some(filename) = path.file_name().map(|s| s.to_string_lossy().to_string()) else {
            eprintln!("Warning found path ending in '...': {}", path.display());
            continue;
        };

        entries.push((path, filename));
    }

    entries.sort();

    let directory = DirectoryTemplate {
        directory_name: uri.path().to_string(),
        items: entries,
    };
    Ok(HtmlTemplate(directory).into_response())
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
