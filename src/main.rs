use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
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
    items: Vec<String>,
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

    let app = Router::new()
        .route("/", get(root).with_state(serve_dir.clone()))
        .fallback_service(ServeDir::new(serve_dir));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root(State(directory): State<PathBuf>) -> impl IntoResponse {
    let directory_name = directory
        .file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap();

    let paths = std::fs::read_dir(directory).unwrap();
    let mut items = Vec::new();
    for path in paths {
        let path = path.unwrap();
        items.push(path.file_name().into_string().unwrap());
    }

    let directory = DirectoryTemplate {
        directory_name,
        items,
    };
    HtmlTemplate(directory)
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
