use std::path::Path;
use crate::debug::debug_log;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpusAttachment {
    pub id: String,
    pub task_id: String,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub url: Option<String>,
    pub created_at: String,
}

impl super::OpusClient {
    pub async fn get_task_attachments(
        &self,
        task_id: &str,
    ) -> Result<Vec<crate::opus::models::Attachment>, Box<dyn std::error::Error + Send + Sync>> {
        debug_log(&format!("Fetching attachments for task {}", task_id));

        let _task = self.get_task(task_id).await?;

        Ok(Vec::new())
    }
}

pub fn format_file_size(size_bytes: i64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let size = size_bytes as f64;

    if size >= GB {
        format!("{:.1} GB", size / GB)
    } else if size >= MB {
        format!("{:.1} MB", size / MB)
    } else if size >= KB {
        format!("{:.1} KB", size / KB)
    } else {
        format!("{} B", size_bytes)
    }
}

pub fn get_file_extension(filename: &str) -> Option<&str> {
    Path::new(filename).extension()?.to_str()
}

pub fn is_image_file(filename: &str) -> bool {
    if let Some(ext) = get_file_extension(filename) {
        let ext_lower = ext.to_lowercase();
        matches!(
            ext_lower.as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" | "tif"
        )
    } else {
        false
    }
}

pub fn get_file_icon(filename: &str) -> &'static str {
    if is_image_file(filename) {
        "\u{1f5bc}\u{fe0f}"
    } else if let Some(ext) = get_file_extension(filename) {
        let ext_lower = ext.to_lowercase();
        match ext_lower.as_str() {
            "pdf" => "\u{1f4c4}",
            "txt" => "\u{1f4c4}",
            "md" => "\u{1f4dd}",
            "zip" | "rar" | "7z" | "tar" | "gz" => "\u{1f4e6}",
            "mp3" | "wav" | "flac" | "ogg" => "\u{1f3b5}",
            "mp4" | "avi" | "mov" | "mkv" => "\u{1f3ac}",
            "py" | "js" | "rs" | "go" | "java" | "cpp" | "c" => "\u{1f4bb}",
            "html" | "css" | "xml" | "json" => "\u{1f310}",
            _ => "\u{1f4ce}",
        }
    } else {
        "\u{1f4ce}"
    }
}
