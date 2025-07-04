use axum::extract::{Query, State};
use axum::http::HeaderValue;
use axum::response::{sse, sse::Event as SseEvent, IntoResponse, Sse};
use axum::{response::Result, routing::{get, put}, Form, Router};
use notify::{Event as NotifyEvent, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use std::convert::Infallible;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tower_http::services::ServeDir;

mod templates;
use templates::{FileListingEntryTemplate, FileListingTemplate, HomeTemplate};

#[tokio::main]
async fn main() {
    println!("Starting server on http://localhost:3000 ...");

    let args: Vec<String> = env::args().collect();

    let (b_tx, _) = broadcast::channel::<NotifyEvent>(5);
    let config = Config::build(&args, b_tx.clone()).expect("Failed to load configuration");

    // Spin up the watcher so we know when files got added or updated
    let mut watcher = RecommendedWatcher::new(move |e: notify::Result<Event>| {
        if let Ok(event) = e {
            // Tmp files are probably still being written to disk, so will have a lot of changes
            if !event.kind.is_access() && event.paths.iter().any(|p| p.extension() != Some(OsStr::new("tmp"))) {
                let send_res = b_tx.send(event);

                if !send_res.is_ok() {
                    eprintln!("Failed to notify watcher: {:?}", send_res.unwrap_err());
                }
            }
        }
    }, notify::Config::default().with_poll_interval(Duration::from_secs(2))).unwrap();
    watcher.watch(&config.files_base_path, RecursiveMode::Recursive).unwrap();

    let app = Router::new()
        .route("/", get(home))
        .route("/list", get(list_files))
        .route("/toggle-status", put(set_heard))
        .route("/events", get(sse_handler))
        .nest_service("/file", ServeDir::new(config.files_base_path.clone()))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(config);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn home<'a>(Query(query): Query<ListFilesQueryParams>) -> impl IntoResponse {
    HomeTemplate {
        current_relative_path: query.path.unwrap_or(String::from("")),
    }
}

async fn list_files<'a>(Query(query): Query<ListFilesQueryParams>, State(state): State<Config>) -> Result<impl IntoResponse, String> {
    Ok(get_file_list(query.path, state, query.push_history.is_some_and(|v| v)).await?)
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

async fn sse_handler(State(state): State<Config>) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
    let stream = BroadcastStream::new(state.b_tx.subscribe()).map(move |e| {
        if let Ok(evt) = e {
            let path = evt.paths.first().unwrap().strip_prefix(&state.files_base_path).unwrap().to_string_lossy().to_string();
            println!("Received SSE event for path: {}", path);
            Ok(SseEvent::default().event("fileUpdated").data(path))
        } else {
            Ok(SseEvent::default().event("watcher_error").data(format!("{:?}", e)))
        }
    });

    Sse::new(stream).keep_alive(sse::KeepAlive::default())
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

#[derive(Deserialize)]
struct ListFilesQueryParams {
    path: Option<String>,
    push_history: Option<bool>
}

#[derive(Deserialize)]
struct SetHeardParams {
    path: String,
    heard: Option<bool>,
}

#[derive(Clone)]
pub struct Config {
    pub files_base_path: PathBuf,
    pub b_tx: broadcast::Sender<NotifyEvent>,
}

impl Config {
    pub fn build(_args: &[String], b_tx: broadcast::Sender<NotifyEvent>) -> Result<Config, &'static str> {
        let files_base_path_str = env::var("FILES_BASE_PATH")
            .map_err(|_| "FILES_BASE_PATH env var not set or unable to be read")?;
        let files_base_path = PathBuf::from(files_base_path_str).to_path_buf();
        // check if the path exists
        if !files_base_path.exists() {
            return Err("Files base path doesn't exist!")
        }

        Ok(Config {
            files_base_path: std::path::PathBuf::from(files_base_path),
            b_tx,
        })
    }
}