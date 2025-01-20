use super::{
    constant::AUTHORIZATION_BEARER_PREFIX,
    lazy::AUTH_TOKEN,
    model::AppConfig,
};
use crate::common::models::{
    config::{ConfigData, ConfigUpdateRequest},
    ApiStatus, ErrorResponse, NormalResponse,
};
use axum::{
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    Json,
};

// 定义处理更新操作的宏
macro_rules! handle_update {
    ($request:expr, $field:ident, $update_fn:expr, $field_name:expr) => {
        if let Some($field) = $request.$field {
            if let Err(e) = $update_fn($field) {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        status: ApiStatus::Failed,
                        code: Some(500),
                        error: Some(format!("更新 {} 失败: {}", $field_name, e)),
                        message: None,
                    }),
                ));
            }
        }
    };
}

// 定义处理重置操作的宏
macro_rules! handle_reset {
    ($request:expr, $field:ident, $reset_fn:expr, $field_name:expr) => {
        if $request.$field.is_some() {
            if let Err(e) = $reset_fn() {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        status: ApiStatus::Failed,
                        code: Some(500),
                        error: Some(format!("重置 {} 失败: {}", $field_name, e)),
                        message: None,
                    }),
                ));
            }
        }
    };
}

pub async fn handle_config_update(
    headers: HeaderMap,
    Json(request): Json<ConfigUpdateRequest>,
) -> Result<Json<NormalResponse<ConfigData>>, (StatusCode, Json<ErrorResponse>)> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix(AUTHORIZATION_BEARER_PREFIX))
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                status: ApiStatus::Failed,
                code: Some(401),
                error: Some("未提供认证令牌".to_string()),
                message: None,
            }),
        ))?;

    if auth_header != AUTH_TOKEN.as_str() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                status: ApiStatus::Failed,
                code: Some(401),
                error: Some("无效的认证令牌".to_string()),
                message: None,
            }),
        ));
    }

    match request.action.as_str() {
        "get" => Ok(Json(NormalResponse {
            status: ApiStatus::Success,
            data: Some(ConfigData {
                page_content: AppConfig::get_page_content(&request.path),
                enable_stream_check: AppConfig::get_stream_check(),
                include_stop_stream: AppConfig::get_stop_stream(),
                vision_ability: AppConfig::get_vision_ability(),
                enable_slow_pool: AppConfig::get_slow_pool(),
                enable_all_claude: AppConfig::get_allow_claude(),
                check_usage_models: AppConfig::get_usage_check(),
            }),
            message: None,
        })),

        "update" => {
            // 处理页面内容更新
            if !request.path.is_empty() && request.content.is_some() {
                let content = request.content.unwrap();
                if let Err(e) = AppConfig::update_page_content(&request.path, content) {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            status: ApiStatus::Failed,
                            code: Some(500),
                            error: Some(format!("更新页面内容失败: {}", e)),
                            message: None,
                        }),
                    ));
                }
            }

            handle_update!(
                request,
                enable_stream_check,
                AppConfig::update_stream_check,
                "enable_stream_check"
            );
            handle_update!(
                request,
                include_stop_stream,
                AppConfig::update_stop_stream,
                "include_stop_stream"
            );
            handle_update!(
                request,
                vision_ability,
                AppConfig::update_vision_ability,
                "vision_ability"
            );
            handle_update!(
                request,
                enable_slow_pool,
                AppConfig::update_slow_pool,
                "enable_slow_pool"
            );
            handle_update!(
                request,
                enable_all_claude,
                AppConfig::update_allow_claude,
                "enable_all_claude"
            );
            handle_update!(
                request,
                check_usage_models,
                AppConfig::update_usage_check,
                "check_usage_models"
            );

            Ok(Json(NormalResponse {
                status: ApiStatus::Success,
                data: None,
                message: Some("配置已更新".to_string()),
            }))
        }

        "reset" => {
            // 重置页面内容
            if !request.path.is_empty() {
                if let Err(e) = AppConfig::reset_page_content(&request.path) {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            status: ApiStatus::Failed,
                            code: Some(500),
                            error: Some(format!("重置页面内容失败: {}", e)),
                            message: None,
                        }),
                    ));
                }
            }

            handle_reset!(
                request,
                enable_stream_check,
                AppConfig::reset_stream_check,
                "enable_stream_check"
            );
            handle_reset!(
                request,
                include_stop_stream,
                AppConfig::reset_stop_stream,
                "include_stop_stream"
            );
            handle_reset!(
                request,
                vision_ability,
                AppConfig::reset_vision_ability,
                "vision_ability"
            );
            handle_reset!(
                request,
                enable_slow_pool,
                AppConfig::reset_slow_pool,
                "enable_slow_pool"
            );
            handle_reset!(
                request,
                enable_all_claude,
                AppConfig::reset_allow_claude,
                "enable_all_claude"
            );
            handle_reset!(
                request,
                check_usage_models,
                AppConfig::reset_usage_check,
                "check_usage_models"
            );

            Ok(Json(NormalResponse {
                status: ApiStatus::Success,
                data: None,
                message: Some("配置已重置".to_string()),
            }))
        }

        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: ApiStatus::Failed,
                code: Some(400),
                error: Some("无效的操作类型".to_string()),
                message: None,
            }),
        )),
    }
}
