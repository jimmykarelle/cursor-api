use serde::{Deserialize, Serialize};

use crate::app::model::{PageContent, UsageCheck, VisionAbility};

#[derive(Serialize)]
pub struct ConfigData {
    pub page_content: Option<PageContent>,
    pub enable_stream_check: bool,
    pub include_stop_stream: bool,
    pub vision_ability: VisionAbility,
    pub enable_slow_pool: bool,
    pub enable_all_claude: bool,
    pub check_usage_models: UsageCheck,
}

#[derive(Deserialize)]
pub struct ConfigUpdateRequest {
    #[serde(default)]
    pub action: String, // "get", "update", "reset"
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub content: Option<PageContent>, // "default", "text", "html"
    #[serde(default)]
    pub enable_stream_check: Option<bool>,
    #[serde(default)]
    pub include_stop_stream: Option<bool>,
    #[serde(default)]
    pub vision_ability: Option<VisionAbility>,
    #[serde(default)]
    pub enable_slow_pool: Option<bool>,
    #[serde(default)]
    pub enable_all_claude: Option<bool>,
    #[serde(default)]
    pub check_usage_models: Option<UsageCheck>,
}
