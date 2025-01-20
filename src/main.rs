mod app;
mod chat;
mod common;

use app::{
    config::handle_config_update,
    constant::{
        EMPTY_STRING, PKG_VERSION, ROUTE_ABOUT_PATH, ROUTE_API_PATH, ROUTE_BASIC_CALIBRATION_PATH,
        ROUTE_CONFIG_PATH, ROUTE_ENV_EXAMPLE_PATH, ROUTE_GET_CHECKSUM, ROUTE_GET_TOKENINFO_PATH,
        ROUTE_GET_USER_INFO_PATH, ROUTE_HEALTH_PATH, ROUTE_LOGS_PATH, ROUTE_README_PATH,
        ROUTE_ROOT_PATH, ROUTE_STATIC_PATH, ROUTE_TOKENINFO_PATH, ROUTE_UPDATE_TOKENINFO_PATH,
    },
    lazy::{AUTH_TOKEN, ROUTE_CHAT_PATH, ROUTE_MODELS_PATH},
    model::*,
};
use axum::{
    routing::{get, post},
    Router,
};
use chat::{
    route::{
        get_user_info, handle_about, handle_api_page, handle_basic_calibration, handle_config_page,
        handle_env_example, handle_get_checksum, handle_get_tokeninfo, handle_health, handle_logs,
        handle_logs_post, handle_readme, handle_root, handle_static, handle_tokeninfo_page,
        handle_update_tokeninfo, handle_update_tokeninfo_post,
    },
    service::{handle_chat, handle_models},
};
use common::utils::{
    load_tokens, parse_bool_from_env, parse_string_from_env, parse_usize_from_env,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer};

#[tokio::main]
async fn main() {
    // 设置自定义 panic hook
    std::panic::set_hook(Box::new(|info| {
        // std::env::set_var("RUST_BACKTRACE", "1");
        if let Some(msg) = info.payload().downcast_ref::<String>() {
            eprintln!("{}", msg);
        } else if let Some(msg) = info.payload().downcast_ref::<&str>() {
            eprintln!("{}", msg);
        }
    }));

    // 加载环境变量
    dotenvy::dotenv().ok();

    if AUTH_TOKEN.is_empty() {
        panic!("AUTH_TOKEN must be set")
    };

    // 初始化全局配置
    AppConfig::init(
        parse_bool_from_env("ENABLE_STREAM_CHECK", true),
        parse_bool_from_env("INCLUDE_STOP_REASON_STREAM", true),
        VisionAbility::from_str(&parse_string_from_env("VISION_ABILITY", EMPTY_STRING)),
        parse_bool_from_env("ENABLE_SLOW_POOL", false),
        parse_bool_from_env("PASS_ANY_CLAUDE", false),
    );

    // 加载 tokens
    let token_infos = load_tokens();

    // 初始化应用状态
    let state = Arc::new(Mutex::new(AppState::new(token_infos)));

    // 设置路由
    let app = Router::new()
        .route(ROUTE_ROOT_PATH, get(handle_root))
        .route(ROUTE_HEALTH_PATH, get(handle_health))
        .route(ROUTE_TOKENINFO_PATH, get(handle_tokeninfo_page))
        .route(ROUTE_MODELS_PATH.as_str(), get(handle_models))
        .route(ROUTE_GET_CHECKSUM, get(handle_get_checksum))
        .route(ROUTE_UPDATE_TOKENINFO_PATH, get(handle_update_tokeninfo))
        .route(ROUTE_GET_TOKENINFO_PATH, post(handle_get_tokeninfo))
        .route(
            ROUTE_UPDATE_TOKENINFO_PATH,
            post(handle_update_tokeninfo_post),
        )
        .route(ROUTE_CHAT_PATH.as_str(), post(handle_chat))
        .route(ROUTE_LOGS_PATH, get(handle_logs))
        .route(ROUTE_LOGS_PATH, post(handle_logs_post))
        .route(ROUTE_ENV_EXAMPLE_PATH, get(handle_env_example))
        .route(ROUTE_CONFIG_PATH, get(handle_config_page))
        .route(ROUTE_CONFIG_PATH, post(handle_config_update))
        .route(ROUTE_STATIC_PATH, get(handle_static))
        .route(ROUTE_ABOUT_PATH, get(handle_about))
        .route(ROUTE_README_PATH, get(handle_readme))
        .route(ROUTE_BASIC_CALIBRATION_PATH, post(handle_basic_calibration))
        .route(ROUTE_GET_USER_INFO_PATH, post(get_user_info))
        .route(ROUTE_API_PATH, get(handle_api_page))
        .layer(RequestBodyLimitLayer::new(
            1024 * 1024 * parse_usize_from_env("REQUEST_BODY_LIMIT_MB", 2),
        ))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // 启动服务器
    let port = parse_string_from_env("PORT", "3000");
    let addr = format!("0.0.0.0:{}", port);
    println!("服务器运行在端口 {}", port);
    println!("当前版本: v{}", PKG_VERSION);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
