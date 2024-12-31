use askama_axum::Template;

#[derive(Template)]
#[template(path = "file_listing.html")]
pub struct FileListingTemplate {
    pub relative_path: String,
    pub parent_relative_path: Option<String>,
    pub files: Vec<FileDto>
}

pub struct FileDto {
    pub name: String,
    pub relative_path: String,
    pub size: String,
    pub is_directory: bool,
    pub is_heard: bool
}