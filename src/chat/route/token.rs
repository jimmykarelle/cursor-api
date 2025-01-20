use crate::{
    app::{
        constant::{
            AUTHORIZATION_BEARER_PREFIX, CONTENT_TYPE_TEXT_HTML_WITH_UTF8,
            CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8, ROUTE_TOKENINFO_PATH,
        },
        lazy::{AUTH_TOKEN, TOKEN_FILE, TOKEN_LIST_FILE},
        model::{AppConfig, AppState, PageContent, TokenUpdateRequest},
    },
    common::{
        models::{ApiStatus, NormalResponseNoData},
        utils::{
            extract_time, extract_time_ks, extract_user_id, generate_checksum_with_default, generate_checksum_with_repair, load_tokens, validate_token_and_checksum
        },
    },
};
use axum::{
    extract::{Query, State},
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderMap,
    },
    response::{IntoResponse, Response},
    Json,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct ChecksumQuery {
    #[serde(default, alias = "checksum")]
    pub bad_checksum: Option<String>,
}

#[derive(Serialize)]
pub struct ChecksumResponse {
    pub checksum: String,
}

pub async fn handle_get_checksum(
    Query(query): Query<ChecksumQuery>
) -> Json<ChecksumResponse> {
    match query.bad_checksum {
        None => Json(ChecksumResponse { checksum: generate_checksum_with_default() }),
        Some(bad_checksum) => Json(ChecksumResponse { checksum: generate_checksum_with_repair(&bad_checksum) })
    }
}

// 更新 TokenInfo 处理
pub async fn handle_update_tokeninfo(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<NormalResponseNoData> {
    // 重新加载 tokens
    let token_infos = load_tokens();

    // 更新应用状态
    {
        let mut state = state.lock().await;
        state.token_infos = token_infos;
    }

    Json(NormalResponseNoData {
        status: ApiStatus::Success,
        message: Some("Token list has been reloaded".to_string()),
    })
}

// 获取 TokenInfo 处理
pub async fn handle_get_tokeninfo(
    headers: HeaderMap,
) -> Result<Json<TokenInfoResponse>, StatusCode> {
    // 验证 AUTH_TOKEN
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix(AUTHORIZATION_BEARER_PREFIX))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if auth_header != AUTH_TOKEN.as_str() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token_file = TOKEN_FILE.as_str();
    let token_list_file = TOKEN_LIST_FILE.as_str();

    // 读取文件内容
    let tokens = std::fs::read_to_string(&token_file).unwrap_or_else(|_| String::new());
    let token_list = std::fs::read_to_string(&token_list_file).unwrap_or_else(|_| String::new());

    // 获取 tokens_count
    let tokens_count = {
        {
            tokens.len()
        }
    };

    Ok(Json(TokenInfoResponse {
        status: ApiStatus::Success,
        token_file: token_file.to_string(),
        token_list_file: token_list_file.to_string(),
        tokens: Some(tokens),
        tokens_count: Some(tokens_count),
        token_list: Some(token_list),
        message: None,
    }))
}

#[derive(Serialize)]
pub struct TokenInfoResponse {
    pub status: ApiStatus,
    pub token_file: String,
    pub token_list_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_list: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub async fn handle_update_tokeninfo_post(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(request): Json<TokenUpdateRequest>,
) -> Result<Json<TokenInfoResponse>, StatusCode> {
    // 验证 AUTH_TOKEN
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix(AUTHORIZATION_BEARER_PREFIX))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if auth_header != AUTH_TOKEN.as_str() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token_file = TOKEN_FILE.as_str();
    let token_list_file = TOKEN_LIST_FILE.as_str();

    // 写入文件
    std::fs::write(&token_file, &request.tokens).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(token_list) = &request.token_list {
        std::fs::write(&token_list_file, token_list)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // 重新加载 tokens
    let token_infos = load_tokens();
    let token_infos_len = token_infos.len();

    // 更新应用状态
    {
        let mut state = state.lock().await;
        state.token_infos = token_infos;
    }

    Ok(Json(TokenInfoResponse {
        status: ApiStatus::Success,
        token_file: token_file.to_string(),
        token_list_file: token_list_file.to_string(),
        tokens: None,
        tokens_count: Some(token_infos_len),
        token_list: None,
        message: Some("Token files have been updated and reloaded".to_string()),
    }))
}

pub async fn handle_tokeninfo_page() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_TOKENINFO_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_HTML_WITH_UTF8)
            .body(include_str!("../../../static/tokeninfo.min.html").to_string())
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

#[derive(Deserialize)]
pub struct TokenRequest {
    pub token: Option<String>,
}

#[derive(Serialize)]
pub struct BasicCalibrationResponse {
    pub status: ApiStatus,
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_time: Option<u64>,
}

pub async fn handle_basic_calibration(
    Json(request): Json<TokenRequest>,
) -> Json<BasicCalibrationResponse> {
    // 从请求头中获取并验证 auth token
    let auth_token = match request.token {
        Some(token) => token,
        None => {
            return Json(BasicCalibrationResponse {
                status: ApiStatus::Error,
                message: Some("未提供授权令牌".to_string()),
                user_id: None,
                create_at: None,
                checksum_time: None,
            })
        }
    };

    // 校验 token 和 checksum
    let (token, checksum) = match validate_token_and_checksum(&auth_token) {
        Some(parts) => parts,
        None => {
            return Json(BasicCalibrationResponse {
                status: ApiStatus::Error,
                message: Some("无效令牌或无效校验和".to_string()),
                user_id: None,
                create_at: None,
                checksum_time: None,
            })
        }
    };

    // 提取用户ID和创建时间
    let user_id = extract_user_id(&token);
    let create_at = extract_time(&token).map(|dt| dt.to_string());
    let checksum_time = extract_time_ks(&checksum[..8]);

    // 返回校验结果
    Json(BasicCalibrationResponse {
        status: ApiStatus::Success,
        message: Some("校验成功".to_string()),
        user_id,
        create_at,
        checksum_time,
    })
}
