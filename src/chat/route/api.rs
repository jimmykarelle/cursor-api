use axum::response::{IntoResponse, Response};
use reqwest::header::CONTENT_TYPE;

use crate::{
    app::constant::{
        CONTENT_TYPE_TEXT_HTML_WITH_UTF8, CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8, ROUTE_API_PATH,
    },
    AppConfig, PageContent,
};

pub async fn handle_api_page() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_API_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_HTML_WITH_UTF8)
            .body(include_str!("../../../static/api.min.html").to_string())
            .unwrap(),
        PageContent::Text(content) => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8)
            .body(content.clone())
            .unwrap(),
        PageContent::Html(content) => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_HTML_WITH_UTF8)
            .body(content.clone())
            .unwrap(),
    }
}
