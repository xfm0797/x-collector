use serde::Serialize;
use tauri::Emitter;

/// 采集进度事件
#[derive(Serialize, Clone)]
pub struct CollectProgress {
    pub stage: String,
    pub current: usize,
    pub total: usize,
    pub message: String,
}

/// 采集日志事件
#[derive(Serialize, Clone)]
pub struct CollectLogEvent {
    pub status: String,
    pub url: Option<String>,
    pub message: String,
}

/// 采集汇总
#[derive(Serialize, Default)]
pub struct CollectSummary {
    pub found: usize,
    pub collected: usize,
    pub skipped: usize,
    pub failed: usize,
}

/// 发送采集进度事件
pub fn emit_progress(app: &tauri::AppHandle, stage: &str, current: usize, total: usize, message: &str) {
    let _ = app.emit(
        "collect:progress",
        CollectProgress {
            stage: stage.to_string(),
            current,
            total,
            message: message.to_string(),
        },
    );
}

/// 发送采集日志事件
pub fn emit_log(app: &tauri::AppHandle, status: &str, url: Option<&str>, message: &str) {
    let _ = app.emit(
        "collect:log",
        CollectLogEvent {
            status: status.to_string(),
            url: url.map(|s| s.to_string()),
            message: message.to_string(),
        },
    );
}
