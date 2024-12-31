use askama_axum::Template;

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    pub current_relative_path: String
}