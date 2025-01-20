use crate::app::{
    constant::{
        CONTENT_TYPE_TEXT_CSS_WITH_UTF8, CONTENT_TYPE_TEXT_HTML_WITH_UTF8,
        CONTENT_TYPE_TEXT_JS_WITH_UTF8, CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8, ROUTE_ABOUT_PATH,
        ROUTE_CONFIG_PATH, ROUTE_README_PATH, ROUTE_SHARED_JS_PATH, ROUTE_SHARED_STYLES_PATH,
    },
    model::{AppConfig, PageContent},
};
use axum::{
    body::Body,
    extract::Path,
    http::{
        header::{CONTENT_TYPE, LOCATION},
        StatusCode,
    },
    response::{IntoResponse, Response},
};

pub async fn handle_env_example() -> impl IntoResponse {
    Response::builder()
        .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8)
        .body(include_str!("../../../.env.example").to_string())
        .unwrap()
}

// 配置页面处理函数
pub async fn handle_config_page() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_CONFIG_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_HTML_WITH_UTF8)
            .body(include_str!("../../../static/config.min.html").to_string())
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

pub async fn handle_static(Path(path): Path<String>) -> impl IntoResponse {
    match path.as_str() {
        "shared-styles.css" => {
            match AppConfig::get_page_content(ROUTE_SHARED_STYLES_PATH).unwrap_or_default() {
                PageContent::Default => Response::builder()
                    .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_CSS_WITH_UTF8)
                    .body(include_str!("../../../static/shared-styles.min.css").to_string())
                    .unwrap(),
                PageContent::Text(content) | PageContent::Html(content) => Response::builder()
                    .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_CSS_WITH_UTF8)
                    .body(content.clone())
                    .unwrap(),
            }
        }
        "shared.js" => {
            match AppConfig::get_page_content(ROUTE_SHARED_JS_PATH).unwrap_or_default() {
                PageContent::Default => Response::builder()
                    .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_JS_WITH_UTF8)
                    .body(include_str!("../../../static/shared.min.js").to_string())
                    .unwrap(),
                PageContent::Text(content) | PageContent::Html(content) => Response::builder()
                    .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_JS_WITH_UTF8)
                    .body(content.clone())
                    .unwrap(),
            }
        }
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not found".to_string())
            .unwrap(),
    }
}

pub async fn handle_about() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_ABOUT_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_HTML_WITH_UTF8)
            .body(include_str!("../../../static/readme.min.html").to_string())
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

pub async fn handle_readme() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_README_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, ROUTE_ABOUT_PATH)
            .body(Body::empty())
            .unwrap(),
        PageContent::Text(content) => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8)
            .body(Body::from(content.clone()))
            .unwrap(),
        PageContent::Html(content) => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_HTML_WITH_UTF8)
            .body(Body::from(content.clone()))
            .unwrap(),
    }
}
