use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Vision(Vec<VisionMessageContent>),
}

#[derive(Serialize, Deserialize)]
pub struct VisionMessageContent {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<ImageUrl>,
}

#[derive(Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: MessageContent,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Role {
    #[serde(rename = "system", alias = "developer")]
    System,
    #[serde(rename = "user", alias = "human")]
    User,
    #[serde(rename = "assistant", alias = "ai")]
    Assistant,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub choices: Vec<Choice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Serialize)]
pub struct Choice {
    pub index: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<Delta>,
    pub finish_reason: Option<String>,
}

#[derive(Serialize)]
pub struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// 模型定义
#[derive(Serialize, Clone)]
pub struct Model {
    pub id: &'static str,
    pub created: &'static i64,
    pub object: &'static str,
    pub owned_by: &'static str,
}

use crate::app::model::{AppConfig, UsageCheck};
use super::constant::USAGE_CHECK_MODELS;

impl Model {
    pub fn is_usage_check(&self) -> bool {
        match AppConfig::get_usage_check() {
            UsageCheck::None => false,
            UsageCheck::Default => USAGE_CHECK_MODELS.contains(&self.id),
            UsageCheck::All => true,
            UsageCheck::Custom(models) => models.contains(&self.id),
        }
    }
}

#[derive(Serialize)]
pub struct ModelsResponse {
    pub object: &'static str,
    pub data: &'static [Model],
}
