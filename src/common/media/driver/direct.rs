//! # 本地直连 / 存储直链驱动 (Direct Storage Image Driver)

use super::{ImageDriver, ImageOptions};

/// 直连图片驱动：适用于本地环境或不需要云图片转换时的直链拼接
#[derive(Debug, Clone)]
pub struct DirectDriver {
    base_url: String,
}

impl DirectDriver {
    pub fn new(base_url: impl Into<String>) -> Self {
        let mut base = base_url.into();
        if base.ends_with('/') {
            base.pop();
        }
        Self { base_url: base }
    }
}

impl Default for DirectDriver {
    fn default() -> Self {
        Self::new("")
    }
}

impl ImageDriver for DirectDriver {
    fn build_url(&self, relative_path: &str, _options: &ImageOptions) -> String {
        let clean_path = relative_path.trim_start_matches('/');
        if self.base_url.is_empty() {
            format!("/{clean_path}")
        } else {
            format!("{}/{clean_path}", self.base_url)
        }
    }
}
