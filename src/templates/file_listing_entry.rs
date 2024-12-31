use askama_axum::Template;

#[derive(Template)]
#[template(path = "file_listing_entry.html")]
pub struct FileListingEntryTemplate {
    pub name: String,
    pub relative_path: String,
    pub size: String,
    pub is_directory: bool,
    pub is_heard: bool
}