use crate::{
    app::{
        constant::{
            AUTHORIZATION_BEARER_PREFIX, CONTENT_TYPE_TEXT_HTML_WITH_UTF8,
            CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8, ROUTE_LOGS_PATH,
        },
        lazy::AUTH_TOKEN,
        model::{AppConfig, AppState, PageContent, RequestLog},
    },
    common::{models::ApiStatus, utils::extract_token},
};
use axum::{
    body::Body,
    extract::State,
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderMap, StatusCode,
    },
    response::{IntoResponse, Response},
    Json,
};
use chrono::Local;
use std::sync::Arc;
use tokio::sync::Mutex;

// 日志处理
pub async fn handle_logs() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_LOGS_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_HTML_WITH_UTF8)
            .body(Body::from(
                include_str!("../../../static/logs.min.html").to_string(),
            ))
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

pub async fn handle_logs_post(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
) -> Result<Json<LogsResponse>, StatusCode> {
    let auth_token = AUTH_TOKEN.as_str();

    // 获取认证头
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix(AUTHORIZATION_BEARER_PREFIX))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let state = state.lock().await;

    // 如果是管理员token,返回所有日志
    if auth_header == auth_token {
        return Ok(Json(LogsResponse {
            status: ApiStatus::Success,
            total: state.total_requests,
            active: Some(state.active_requests),
            error: Some(state.error_requests),
            logs: state.request_logs.clone(),
            timestamp: Local::now().to_string(),
        }));
    }

    // 解析 token
    let token_part = extract_token(auth_header).ok_or(StatusCode::UNAUTHORIZED)?;

    // 否则筛选出token匹配的日志
    let filtered_logs: Vec<RequestLog> = state
        .request_logs
        .iter()
        .filter(|log| log.token_info.token == token_part)
        .cloned()
        .collect();

    // 如果没有匹配的日志,返回未授权错误
    if filtered_logs.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(Json(LogsResponse {
        status: ApiStatus::Success,
        total: filtered_logs.len() as u64,
        active: None,
        error: None,
        logs: filtered_logs,
        timestamp: Local::now().to_string(),
    }))
}

#[derive(serde::Serialize)]
pub struct LogsResponse {
    pub status: ApiStatus,
    pub total: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<u64>,
    pub logs: Vec<RequestLog>,
    pub timestamp: String,
}
