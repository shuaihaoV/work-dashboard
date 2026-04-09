use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub generated_at: DateTime<Utc>,
    pub cache_ttl_sec: u64,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn new(cache_ttl_sec: u64, data: T) -> Self {
        Self {
            generated_at: Utc::now(),
            cache_ttl_sec,
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewStats {
    pub total_requests: i64,
    pub success_rate: f64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cached_tokens: i64,
    pub total_quota: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStatsItem {
    pub user_id: i64,
    pub user_name: String,
    pub total_requests: i64,
    pub success_rate: f64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_tokens: i64,
    pub quota_used: i64,
    pub avg_latency_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOptionItem {
    pub user_id: i64,
    pub user_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelOptionItem {
    pub model_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelOptionItem {
    pub channel_id: i64,
    pub channel_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelStatsItem {
    pub channel_id: i64,
    pub channel_name: String,
    pub channel_type: String,
    pub status: String,
    pub total_requests: i64,
    pub success_rate: f64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_tokens: i64,
    pub quota_used: i64,
    pub avg_latency_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatsItem {
    pub model_name: String,
    pub total_requests: i64,
    pub success_rate: f64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_tokens: i64,
    pub quota_used: i64,
    pub avg_latency_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawModelStatsItem {
    pub model_name: String,
    pub channel_id: i64,
    pub channel_name: String,
    pub channel_type: String,
    pub total_requests: i64,
    pub success_rate: f64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_tokens: i64,
    pub quota_used: i64,
    pub avg_latency_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopThroughputChannel {
    pub channel_id: i64,
    pub channel_name: String,
    pub tokens_per_second: f64,
    pub request_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopRequestedModel {
    pub model_name: String,
    pub total_requests: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraStats {
    pub top_throughput_channels: Vec<TopThroughputChannel>,
    pub top_requested_models: Vec<TopRequestedModel>,
}
