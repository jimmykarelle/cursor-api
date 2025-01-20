mod checksum;
pub use checksum::*;
mod tokens;
pub use tokens::*;

use super::models::userinfo::{StripeProfile, TokenProfile, UsageProfile, UserProfile};
use crate::app::constant::{FALSE, TRUE};

pub fn parse_bool_from_env(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| match v.to_lowercase().as_str() {
            TRUE | "1" => true,
            FALSE | "0" => false,
            _ => default,
        })
        .unwrap_or(default)
}

pub fn parse_string_from_env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

pub fn parse_usize_from_env(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub async fn get_token_profile(auth_token: &str) -> Option<TokenProfile> {
    let user_id = extract_user_id(auth_token)?;

    // 构建请求客户端
    let client = super::client::build_usage_client(&user_id, auth_token);

    // 发送请求并获取响应
    // let response = client.send().await.ok()?;
    // let bytes = response.bytes().await?;
    // println!("Raw response bytes: {:?}", bytes);
    // let usage = serde_json::from_str::<UsageProfile>(&text).ok()?;
    let usage = client
        .send()
        .await
        .ok()?
        .json::<UsageProfile>()
        .await
        .ok()?;

    let user = get_user_profile(auth_token).await?;

    // 从 Stripe 获取用户资料
    let stripe = get_stripe_profile(auth_token).await?;

    // 映射响应数据到 TokenProfile
    Some(TokenProfile {
        usage,
        user,
        stripe,
    })
}

pub async fn get_stripe_profile(auth_token: &str) -> Option<StripeProfile> {
    let client = super::client::build_profile_client(auth_token);
    let response = client
        .send()
        .await
        .ok()?
        .json::<StripeProfile>()
        .await
        .ok()?;
    Some(response)
}

pub async fn get_user_profile(auth_token: &str) -> Option<UserProfile> {
    let user_id = extract_user_id(auth_token)?;

    // 构建请求客户端
    let client = super::client::build_userinfo_client(&user_id, auth_token);

    // 发送请求并获取响应
    let user_profile = client.send().await.ok()?.json::<UserProfile>().await.ok()?;

    Some(user_profile)
}

pub fn validate_token_and_checksum(auth_token: &str) -> Option<(String, String)> {
    // 找最后一个逗号
    let comma_pos = auth_token.rfind(',')?;
    let (token_part, checksum) = auth_token.split_at(comma_pos);
    let checksum = &checksum[1..]; // 跳过逗号

    // 解析 token - 为了向前兼容,忽略最后一个:或%3A前的内容
    let colon_pos = token_part.rfind(':');
    let encoded_colon_pos = token_part.rfind("%3A");

    let token = match (colon_pos, encoded_colon_pos) {
        (None, None) => token_part, // 最简单的构成: token,checksum
        (Some(pos1), None) => &token_part[(pos1 + 1)..],
        (None, Some(pos2)) => &token_part[(pos2 + 3)..],
        (Some(pos1), Some(pos2)) => {
            let pos = pos1.max(pos2);
            let start = if pos == pos2 { pos + 3 } else { pos + 1 };
            &token_part[start..]
        }
    };

    // 验证 token 和 checksum 有效性
    if validate_token(token) && validate_checksum(checksum) {
        Some((token.to_string(), checksum.to_string()))
    } else {
        None
    }
}

pub fn extract_token(auth_token: &str) -> Option<String> {
    // 解析 token
    let token_part = match auth_token.rfind(',') {
        Some(pos) => &auth_token[..pos],
        None => auth_token
    };

    let colon_pos = token_part.rfind(':');
    let encoded_colon_pos = token_part.rfind("%3A");

    let token = match (colon_pos, encoded_colon_pos) {
        (None, None) => token_part,
        (Some(pos1), None) => &token_part[(pos1 + 1)..],
        (None, Some(pos2)) => &token_part[(pos2 + 3)..],
        (Some(pos1), Some(pos2)) => {
            let pos = pos1.max(pos2);
            let start = if pos == pos2 { pos + 3 } else { pos + 1 };
            &token_part[start..]
        }
    };

    // 验证 token 有效性
    if validate_token(token) {
        Some(token.to_string())
    } else {
        None
    }
}

pub fn format_time_ms(seconds: f64) -> f64 {
    (seconds * 1000.0).round() / 1000.0
}
