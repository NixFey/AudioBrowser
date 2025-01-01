use axum::{routing::{get, put}, Router, response::Result, Form};
use tower_http::services::ServeDir;
use std::env;
use std::path::{Path, PathBuf};
use axum::extract::{Query, State};
use axum::http::{HeaderValue};
use axum::response::IntoResponse;
use serde::Deserialize;

mod templates;
use templates::{FileListingTemplate, FileListingEntryTemplate, HomeTemplate};

#[tokio::main]
async fn main() {
    println!("Starting server on http://localhost:3000 ...");

    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).expect("Failed to load configuration");

    let app = Router::new()
        .route("/", get(home))
        .route("/list", get(list_files))
        .route("/toggle-status", put(set_heard))
        .nest_service("/file", ServeDir::new(config.clone().files_base_path))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(config.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct ListFilesQueryParams {
    path: Option<String>,
    push_history: Option<bool>
}

async fn home<'a>(Query(query): Query<ListFilesQueryParams>) -> impl IntoResponse {
    HomeTemplate {
        current_relative_path: query.path.unwrap_or(String::from("")),
    }
}

fn get_file_heard(path: &PathBuf) -> bool {
    xattr::get(path, "user.heard").map_or_else(|_| false, |val| {
        val.is_some_and(|v| v.first().is_some_and(|c| *c == 1))
    })
}

async fn get_file_list<'a>(path: Option<String>, state: Config, push_history: bool) -> Result<impl IntoResponse, String> {
    let path = path.as_ref();
    let base = &state.files_base_path;
    let mut path_buf = if path.is_none_or(|p| p.is_empty()) {
        &base
    } else {
        &base.join(path.unwrap())
    };

    if !path_buf.canonicalize().is_ok_and(|f| f.starts_with(&state.files_base_path)) {
        path_buf = &base
    }

    if !path_buf.exists() {
        return Err("Path does not exist".into())
    }

    if !path_buf.is_dir() {
        return Err("Provided path is not a directory".into())
    }

    let mut files = path_buf.read_dir()
        .map_err(|e| format!("Unable to get list of files: {err}", err = e.to_string()))?
        .filter(|de| !de.as_ref().is_ok_and(|e| e.file_name().to_str().unwrap().starts_with('.')))
        .collect::<Vec<_>>();

    files.sort_by_key(|de| de.as_ref().unwrap().file_name());

    let file_names = files.iter().filter_map(|f| f.as_ref().ok().map(|e|
        FileListingEntryTemplate {
            name: e.file_name().to_string_lossy().to_string(),
            relative_path: e.path().strip_prefix(base).unwrap_or(Path::new(".")).to_string_lossy().to_string(),
            size: format!("{} KB", e.metadata().unwrap().len() / 1024),
            is_directory: e.file_type().unwrap().is_dir(),
            is_heard: get_file_heard(&e.path())
        }
    )).collect::<Vec<_>>();

    let relative_path = path_buf.strip_prefix(base).unwrap_or(path_buf.as_path()).to_string_lossy().to_string();
    let parent_relative_path = path_buf.parent().and_then(|f| f.strip_prefix(base).ok().map(|f| f.to_string_lossy().to_string()));
    let mut resp = FileListingTemplate {
        relative_path: relative_path.clone(),
        parent_relative_path,
        files: file_names
    }.into_response();

    if push_history {
        resp.headers_mut().insert("HX-Push-Url", HeaderValue::try_from(format!("?path={}", relative_path.clone())).unwrap());
    }

    Ok(resp)
}

async fn list_files<'a>(Query(query): Query<ListFilesQueryParams>, State(state): State<Config>) -> Result<impl IntoResponse, String> {
    Ok(get_file_list(query.path, state, query.push_history.is_some_and(|v| v)).await?)
}

#[derive(Deserialize)]
struct SetHeardParams {
    path: String,
    heard: Option<bool>,
}

async fn set_heard<'a>(State(state): State<Config>, Form(body): Form<SetHeardParams>) -> Result<impl IntoResponse, String> {
    let base = &state.files_base_path;
    let file_path = &base.join(&body.path);

    if !file_path.starts_with(&base) {
        return Err("Invalid path".into())
    }

    if !file_path.is_file() {
        return Err("Provided path is not a file".into())
    }

    let new_value = if body.heard.is_some() {
        body.heard.unwrap()
    } else {
        !get_file_heard(file_path)
    };

    xattr::set(file_path, "user.heard", &[new_value.into()]).unwrap();

    Ok(FileListingEntryTemplate {
        name: file_path.file_name().unwrap().to_string_lossy().to_string(),
        relative_path: file_path.strip_prefix(base).unwrap_or(Path::new(".")).to_string_lossy().to_string(),
        size: format!("{} KB", file_path.metadata().unwrap().len() / 1024),
        is_directory: file_path.metadata().unwrap().is_dir(),
        is_heard: get_file_heard(&file_path)
    })
}

#[derive(Clone)]
pub struct Config {
    pub files_base_path: PathBuf,
}

impl Config {
    pub fn build(_args: &[String]) -> Result<Config, &'static str> {
        let files_base_path_str = env::var("FILES_BASE_PATH")
            .map_err(|_| "FILES_BASE_PATH env var not set or unable to be read")?;
        let files_base_path = PathBuf::from(files_base_path_str).to_path_buf();
        // check if the path exists
        if !files_base_path.exists() {
            return Err("Files base path doesn't exist!")
        }

        Ok(Config {
            files_base_path: std::path::PathBuf::from(files_base_path),
        })
    }
}