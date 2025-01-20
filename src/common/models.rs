pub mod error;
pub mod health;
pub mod config;
pub mod userinfo;

use config::ConfigData;

use serde::Serialize;

#[derive(Serialize)]
pub enum ApiStatus {
    #[serde(rename = "healthy")]
    Healthy,
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "failed")]
    Failed,
}

// #[derive(Serialize)]
// #[serde(untagged)]
// pub enum ApiResponse {
//     HealthCheck(HealthCheckResponse),
//     ConfigData(NormalResponse<ConfigData>),
//     Error(ErrorResponse),
// }

// impl ApiResponse {
//     pub fn to_string(&self) -> String {
//         serde_json::to_string(self).unwrap()
//     }
// }

#[derive(Serialize)]
pub struct NormalResponse<T> {
    pub status: ApiStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl std::fmt::Display for NormalResponse<ConfigData> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Serialize)]
pub struct NormalResponseNoData {
    pub status: ApiStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    // status -> 成功 / 失败
    pub status: ApiStatus,
    // HTTP 请求的状态码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u16>,
    // HTTP 请求的错误码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    // 错误详情
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
