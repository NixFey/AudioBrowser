use askama_axum::Template;
use crate::templates::FileListingEntryTemplate;

#[derive(Template)]
#[template(path = "file_listing.html")]
pub struct FileListingTemplate {
    pub relative_path: String,
    pub parent_relative_path: Option<String>,
    pub files: Vec<FileListingEntryTemplate>
}