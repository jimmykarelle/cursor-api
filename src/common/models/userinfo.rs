use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(untagged)]
pub enum GetUserInfo {
    Usage(TokenProfile),
    Error{ error: String },
}

#[derive(Serialize, Clone)]
pub struct TokenProfile {
    pub usage: UsageProfile,
    pub user: UserProfile,
    pub stripe: StripeProfile,
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
pub enum MembershipType {
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "free_trial")]
    FreeTrial,
    #[serde(rename = "pro")]
    Pro,
    #[serde(rename = "enterprise")]
    Enterprise,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct StripeProfile {
    #[serde(rename(deserialize = "membershipType"))]
    pub membership_type: MembershipType,
    #[serde(rename(deserialize = "paymentId"), default, skip_serializing_if = "Option::is_none")]
    pub payment_id: Option<String>,
    #[serde(rename(deserialize = "daysRemainingOnTrial"))]
    pub days_remaining_on_trial: u32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ModelUsage {
    #[serde(rename(deserialize = "numRequests", serialize = "requests"))]
    pub num_requests: u32,
    #[serde(rename(deserialize = "numTokens", serialize = "tokens"))]
    pub num_tokens: u32,
    #[serde(
        rename(deserialize = "maxRequestUsage"),
        skip_serializing_if = "Option::is_none"
    )]
    pub max_requests: Option<u32>,
    #[serde(
        rename(deserialize = "maxTokenUsage"),
        skip_serializing_if = "Option::is_none"
    )]
    pub max_tokens: Option<u32>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UsageProfile {
    #[serde(rename(deserialize = "gpt-4"))]
    pub premium: ModelUsage,
    #[serde(rename(deserialize = "gpt-3.5-turbo"))]
    pub standard: ModelUsage,
    #[serde(rename(deserialize = "gpt-4-32k"))]
    pub unknown: ModelUsage,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UserProfile {
    pub email: String,
    // pub email_verified: bool,
    pub name: String,
    #[serde(rename(serialize = "id"))]
    pub sub: String,
    pub updated_at: DateTime<Local>,
    // pub picture: Option<String>,
}
