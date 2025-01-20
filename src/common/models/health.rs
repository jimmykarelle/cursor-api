use serde::Serialize;

use super::ApiStatus;

#[derive(Serialize)]
pub struct HealthCheckResponse {
    pub status: ApiStatus,
    pub version: &'static str,
    pub uptime: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<SystemStats>,
    pub models: Vec<&'static str>,
    pub endpoints: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct SystemStats {
    pub started: String,
    pub total_requests: u64,
    pub active_requests: u64,
    pub system: SystemInfo,
}

#[derive(Serialize)]
pub struct SystemInfo {
    pub memory: MemoryInfo,
    pub cpu: CpuInfo,
}

#[derive(Serialize)]
pub struct MemoryInfo {
    pub rss: u64, // 物理内存使用量(字节)
}

#[derive(Serialize)]
pub struct CpuInfo {
    pub usage: f32, // CPU 使用率(百分比)
}
