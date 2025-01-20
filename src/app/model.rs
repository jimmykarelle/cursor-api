use crate::{
    app::constant::{
        ERR_INVALID_PATH, ERR_RESET_CONFIG, ERR_UPDATE_CONFIG, ROUTE_ABOUT_PATH, ROUTE_CONFIG_PATH,
        ROUTE_LOGS_PATH, ROUTE_README_PATH, ROUTE_ROOT_PATH, ROUTE_SHARED_JS_PATH,
        ROUTE_SHARED_STYLES_PATH, ROUTE_TOKENINFO_PATH, ROUTE_API_PATH,
    },
    common::models::userinfo::TokenProfile,
};
use crate::chat::model::Message;
use std::sync::{LazyLock, RwLock};
use serde::{Deserialize, Serialize};

// 页面内容类型枚举
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum PageContent {
    #[serde(rename = "default")]
    Default, // 默认行为
    #[serde(rename = "text")]
    Text(String), // 纯文本
    #[serde(rename = "html")]
    Html(String), // HTML 内容
}

impl Default for PageContent {
    fn default() -> Self {
        Self::Default
    }
}

mod usage_check;
pub use usage_check::UsageCheck;

// 静态配置
#[derive(Clone)]
pub struct AppConfig {
    stream_check: bool,
    stop_stream: bool,
    vision_ability: VisionAbility,
    slow_pool: bool,
    allow_claude: bool,
    pages: Pages,
    usage_check: UsageCheck,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum VisionAbility {
    #[serde(rename = "none", alias = "disabled")]
    None,
    #[serde(rename = "base64", alias = "base64-only")]
    Base64,
    #[serde(rename = "all", alias = "base64-http")]
    All,
}

impl VisionAbility {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "none" | "disabled" => Self::None,
            "base64" | "base64-only" => Self::Base64,
            "all" | "base64-http" => Self::All,
            _ => Self::default(),
        }
    }
}

impl Default for VisionAbility {
    fn default() -> Self {
        Self::Base64
    }
}

#[derive(Clone, Default)]
pub struct Pages {
    pub root_content: PageContent,
    pub logs_content: PageContent,
    pub config_content: PageContent,
    pub tokeninfo_content: PageContent,
    pub shared_styles_content: PageContent,
    pub shared_js_content: PageContent,
    pub about_content: PageContent,
    pub readme_content: PageContent,
    pub api_content: PageContent,
}

// 运行时状态
pub struct AppState {
    pub total_requests: u64,
    pub active_requests: u64,
    pub error_requests: u64,
    pub request_logs: Vec<RequestLog>,
    pub token_infos: Vec<TokenInfo>,
}

// 全局配置实例
pub static APP_CONFIG: LazyLock<RwLock<AppConfig>> = LazyLock::new(|| {
    RwLock::new(AppConfig::default())
});

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            stream_check: true,
            stop_stream: true,
            vision_ability: VisionAbility::Base64,
            slow_pool: false,
            allow_claude: false,
            pages: Pages::default(),
            usage_check: UsageCheck::Default,
        }
    }
}

macro_rules! config_methods {
    ($($field:ident: $type:ty, $default:expr;)*) => {
        $(
            paste::paste! {
                pub fn [<get_ $field>]() -> $type {
                    APP_CONFIG
                        .read()
                        .map(|config| config.$field.clone())
                        .unwrap_or($default)
                }

                pub fn [<update_ $field>](value: $type) -> Result<(), &'static str> {
                    if let Ok(mut config) = APP_CONFIG.write() {
                        config.$field = value;
                        Ok(())
                    } else {
                        Err(ERR_UPDATE_CONFIG)
                    }
                }

                pub fn [<reset_ $field>]() -> Result<(), &'static str> {
                    if let Ok(mut config) = APP_CONFIG.write() {
                        config.$field = $default;
                        Ok(())
                    } else {
                        Err(ERR_RESET_CONFIG)
                    }
                }
            }
        )*
    };
}

impl AppConfig {
    pub fn init(
        stream_check: bool,
        stop_stream: bool,
        vision_ability: VisionAbility,
        slow_pool: bool,
        allow_claude: bool,
    ) {
        if let Ok(mut config) = APP_CONFIG.write() {
            config.stream_check = stream_check;
            config.stop_stream = stop_stream;
            config.vision_ability = vision_ability;
            config.slow_pool = slow_pool;
            config.allow_claude = allow_claude;
        }
    }

    config_methods! {
        stream_check: bool, true;
        stop_stream: bool, true;
        slow_pool: bool, false;
        allow_claude: bool, false;
    }

    pub fn get_vision_ability() -> VisionAbility {
        APP_CONFIG
            .read()
            .map(|config| config.vision_ability.clone())
            .unwrap_or_default()
    }

    pub fn get_page_content(path: &str) -> Option<PageContent> {
        APP_CONFIG.read().ok().map(|config| match path {
            ROUTE_ROOT_PATH => config.pages.root_content.clone(),
            ROUTE_LOGS_PATH => config.pages.logs_content.clone(),
            ROUTE_CONFIG_PATH => config.pages.config_content.clone(),
            ROUTE_TOKENINFO_PATH => config.pages.tokeninfo_content.clone(),
            ROUTE_SHARED_STYLES_PATH => config.pages.shared_styles_content.clone(),
            ROUTE_SHARED_JS_PATH => config.pages.shared_js_content.clone(),
            ROUTE_ABOUT_PATH => config.pages.about_content.clone(),
            ROUTE_README_PATH => config.pages.readme_content.clone(),
            ROUTE_API_PATH => config.pages.api_content.clone(),
            _ => PageContent::default(),
        })
    }

    pub fn get_usage_check() -> UsageCheck {
        APP_CONFIG
            .read()
            .map(|config| config.usage_check.clone())
            .unwrap_or_default()
    }

    pub fn update_vision_ability(new_ability: VisionAbility) -> Result<(), &'static str> {
        if let Ok(mut config) = APP_CONFIG.write() {
            config.vision_ability = new_ability;
            Ok(())
        } else {
            Err(ERR_UPDATE_CONFIG)
        }
    }

    pub fn update_page_content(path: &str, content: PageContent) -> Result<(), &'static str> {
        if let Ok(mut config) = APP_CONFIG.write() {
            match path {
                ROUTE_ROOT_PATH => config.pages.root_content = content,
                ROUTE_LOGS_PATH => config.pages.logs_content = content,
                ROUTE_CONFIG_PATH => config.pages.config_content = content,
                ROUTE_TOKENINFO_PATH => config.pages.tokeninfo_content = content,
                ROUTE_SHARED_STYLES_PATH => config.pages.shared_styles_content = content,
                ROUTE_SHARED_JS_PATH => config.pages.shared_js_content = content,
                ROUTE_ABOUT_PATH => config.pages.about_content = content,
                ROUTE_README_PATH => config.pages.readme_content = content,
                ROUTE_API_PATH => config.pages.api_content = content,
                _ => return Err(ERR_INVALID_PATH),
            }
            Ok(())
        } else {
            Err(ERR_UPDATE_CONFIG)
        }
    }

    pub fn update_usage_check(rule: UsageCheck) -> Result<(), &'static str> {
        if let Ok(mut config) = APP_CONFIG.write() {
            config.usage_check = rule;
            Ok(())
        } else {
            Err(ERR_UPDATE_CONFIG)
        }
    }

    pub fn reset_vision_ability() -> Result<(), &'static str> {
        if let Ok(mut config) = APP_CONFIG.write() {
            config.vision_ability = VisionAbility::Base64;
            Ok(())
        } else {
            Err(ERR_RESET_CONFIG)
        }
    }

    pub fn reset_page_content(path: &str) -> Result<(), &'static str> {
        if let Ok(mut config) = APP_CONFIG.write() {
            match path {
                ROUTE_ROOT_PATH => config.pages.root_content = PageContent::default(),
                ROUTE_LOGS_PATH => config.pages.logs_content = PageContent::default(),
                ROUTE_CONFIG_PATH => config.pages.config_content = PageContent::default(),
                ROUTE_TOKENINFO_PATH => config.pages.tokeninfo_content = PageContent::default(),
                ROUTE_SHARED_STYLES_PATH => {
                    config.pages.shared_styles_content = PageContent::default()
                }
                ROUTE_SHARED_JS_PATH => config.pages.shared_js_content = PageContent::default(),
                ROUTE_ABOUT_PATH => config.pages.about_content = PageContent::default(),
                ROUTE_README_PATH => config.pages.readme_content = PageContent::default(),
                ROUTE_API_PATH => config.pages.api_content = PageContent::default(),
                _ => return Err(ERR_INVALID_PATH),
            }
            Ok(())
        } else {
            Err(ERR_RESET_CONFIG)
        }
    }

    pub fn reset_usage_check() -> Result<(), &'static str> {
        if let Ok(mut config) = APP_CONFIG.write() {
            config.usage_check = UsageCheck::default();
            Ok(())
        } else {
            Err(ERR_RESET_CONFIG)
        }
    }
}

impl AppState {
    pub fn new(token_infos: Vec<TokenInfo>) -> Self {
        Self {
            total_requests: 0,
            active_requests: 0,
            error_requests: 0,
            request_logs: Vec::new(),
            token_infos,
        }
    }
}

// 请求日志
#[derive(Serialize, Clone)]
pub struct RequestLog {
    pub id: u64,
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub model: String,
    pub token_info: TokenInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    pub timing: TimingInfo,
    pub stream: bool,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct TimingInfo {
    pub total: f64,    // 总用时(秒)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<f64>,  // 首字时间(秒)
}

// 聊天请求
#[derive(Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub stream: bool,
}

// 用于存储 token 信息
#[derive(Serialize, Clone)]
pub struct TokenInfo {
    pub token: String,
    pub checksum: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<TokenProfile>,
}

// TokenUpdateRequest 结构体
#[derive(Deserialize)]
pub struct TokenUpdateRequest {
    pub tokens: String,
    #[serde(default)]
    pub token_list: Option<String>,
}
