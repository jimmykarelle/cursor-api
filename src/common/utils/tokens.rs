use crate::{
    app::{
        constant::EMPTY_STRING,
        model::TokenInfo,
        lazy::{TOKEN_FILE, TOKEN_LIST_FILE},
    },
    common::utils::generate_checksum_with_default,
};

// 规范化文件内容并写入
fn normalize_and_write(content: &str, file_path: &str) -> String {
    let normalized = content.replace("\r\n", "\n");
    if normalized != content {
        if let Err(e) = std::fs::write(file_path, &normalized) {
            eprintln!("警告: 无法更新规范化的文件: {}", e);
        }
    }
    normalized
}

// 解析token
fn parse_token(token_part: &str) -> Option<String> {
    // 查找最后一个:或%3A的位置
    let colon_pos = token_part.rfind(':');
    let encoded_colon_pos = token_part.rfind("%3A");
    
    match (colon_pos, encoded_colon_pos) {
        (None, None) => Some(token_part.to_string()),
        (Some(pos1), None) => Some(token_part[(pos1 + 1)..].to_string()),
        (None, Some(pos2)) => Some(token_part[(pos2 + 3)..].to_string()),
        (Some(pos1), Some(pos2)) => {
            // 取较大的位置作为分隔点
            let pos = pos1.max(pos2);
            let start = if pos == pos2 { pos + 3 } else { pos + 1 };
            Some(token_part[start..].to_string())
        }
    }
}

// Token 加载函数
pub fn load_tokens() -> Vec<TokenInfo> {
    let token_file = TOKEN_FILE.as_str();
    let token_list_file = TOKEN_LIST_FILE.as_str();

    // 确保文件存在
    for file in [&token_file, &token_list_file] {
        if !std::path::Path::new(file).exists() {
            if let Err(e) = std::fs::write(file, EMPTY_STRING) {
                eprintln!("警告: 无法创建文件 '{}': {}", file, e);
            }
        }
    }

    // 读取和规范化 token 文件
    let token_entries = match std::fs::read_to_string(&token_file) {
        Ok(content) => {
            let normalized = content.replace("\r\n", "\n");
            normalized
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') || !validate_token(line) {
                        return None;
                    }
                    parse_token(line)
                })
                .collect::<Vec<_>>()
        }
        Err(e) => {
            eprintln!("警告: 无法读取token文件 '{}': {}", token_file, e);
            Vec::new()
        }
    };

    // 读取和规范化 token-list 文件
    let mut token_map: std::collections::HashMap<String, String> =
        match std::fs::read_to_string(&token_list_file) {
            Ok(content) => {
                let normalized = normalize_and_write(&content, &token_list_file);
                normalized
                    .lines()
                    .filter_map(|line| {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') {
                            return None;
                        }

                        let parts: Vec<&str> = line.split(',').collect();
                        match parts[..] {
                            [token_part, checksum] => {
                                let token = parse_token(token_part)?;
                                Some((token, checksum.to_string()))
                            }
                            _ => {
                                eprintln!("警告: 忽略无效的token-list行: {}", line);
                                None
                            }
                        }
                    })
                    .collect()
            }
            Err(e) => {
                eprintln!("警告: 无法读取token-list文件: {}", e);
                std::collections::HashMap::new()
            }
        };

    // 更新或添加新token
    for token in token_entries {
        if !token_map.contains_key(&token) {
            // 为新token生成checksum
            let checksum = generate_checksum_with_default();
            token_map.insert(token, checksum);
        }
    }

    // 更新 token-list 文件
    let token_list_content = token_map
        .iter()
        .map(|(token, checksum)| {
            format!("{},{}", token, checksum)
        })
        .collect::<Vec<_>>()
        .join("\n");

    if let Err(e) = std::fs::write(&token_list_file, token_list_content) {
        eprintln!("警告: 无法更新token-list文件: {}", e);
    }

    // 转换为 TokenInfo vector
    token_map
        .into_iter()
        .map(|(token, checksum)| TokenInfo {
            token: token.clone(),
            checksum,
            profile: None,
        })
        .collect()
}

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Local, TimeZone};

// 验证jwt token是否有效
pub fn validate_token(token: &str) -> bool {
    // 检查 token 格式
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    if parts[0] != "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9" {
        return false;
    }

    // 解码 payload
    let payload = match URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(decoded) => decoded,
        Err(_) => return false,
    };

    // 转换为字符串
    let payload_str = match String::from_utf8(payload) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // 解析 JSON
    let payload_json: serde_json::Value = match serde_json::from_str(&payload_str) {
        Ok(v) => v,
        Err(_) => return false,
    };

    // 验证必要字段是否存在且有效
    let required_fields = ["sub", "time", "randomness", "exp", "iss", "scope", "aud"];
    for field in required_fields {
        if !payload_json.get(field).is_some() {
            return false;
        }
    }

    // 验证 time 字段
    if let Some(time) = payload_json["time"].as_str() {
        // 验证 time 是否为有效的数字字符串
        if let Ok(time_value) = time.parse::<i64>() {
            let current_time = chrono::Utc::now().timestamp();
            if time_value > current_time {
                return false;
            }
        } else {
            return false;
        }
    } else {
        return false;
    }

    // 验证 randomness 长度
    if let Some(randomness) = payload_json["randomness"].as_str() {
        if randomness.len() != 18 {
            return false;
        }
    } else {
        return false;
    }

    // 验证过期时间
    if let Some(exp) = payload_json["exp"].as_i64() {
        let current_time = chrono::Utc::now().timestamp();
        if current_time > exp {
            return false;
        }
    } else {
        return false;
    }

    // 验证发行者
    if payload_json["iss"].as_str() != Some("https://authentication.cursor.sh") {
        return false;
    }

    // 验证授权范围
    if payload_json["scope"].as_str() != Some("openid profile email offline_access") {
        return false;
    }

    // 验证受众
    if payload_json["aud"].as_str() != Some("https://cursor.com") {
        return false;
    }

    true
}

// 从 JWT token 中提取用户 ID
pub fn extract_user_id(token: &str) -> Option<String> {
    // JWT token 由3部分组成，用 . 分隔
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // 解码 payload (第二部分)
    let payload = match URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(decoded) => decoded,
        Err(_) => return None,
    };

    // 将 payload 转换为字符串
    let payload_str = match String::from_utf8(payload) {
        Ok(s) => s,
        Err(_) => return None,
    };

    // 解析 JSON
    let payload_json: serde_json::Value = match serde_json::from_str(&payload_str) {
        Ok(v) => v,
        Err(_) => return None,
    };

    // 提取 sub 字段
    payload_json["sub"]
        .as_str()
        .map(|s| s.split('|').nth(1).unwrap_or(s).to_string())
}

// 从 JWT token 中提取 time 字段
pub fn extract_time(token: &str) -> Option<DateTime<Local>> {
    // JWT token 由3部分组成，用 . 分隔
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // 解码 payload (第二部分)
    let payload = match URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(decoded) => decoded,
        Err(_) => return None,
    };

    // 将 payload 转换为字符串
    let payload_str = match String::from_utf8(payload) {
        Ok(s) => s,
        Err(_) => return None,
    };

    // 解析 JSON
    let payload_json: serde_json::Value = match serde_json::from_str(&payload_str) {
        Ok(v) => v,
        Err(_) => return None,
    };

    // 提取时间戳并转换为本地时间
    payload_json["time"]
        .as_str()
        .and_then(|t| t.parse::<i64>().ok())
        .and_then(|timestamp| Local.timestamp_opt(timestamp, 0).single())
}
