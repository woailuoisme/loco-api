//! # 阿里云 OSS 图片处理驱动 (Aliyun OSS Image Driver)

use super::{ImageDriver, ImageOptions, ResizeFit};

/// 阿里云 OSS 图片处理服务驱动
///
/// 生成规范格式：`https://{bucket}.{endpoint}/{relative_path}?x-oss-process=image/{actions}`
#[derive(Debug, Clone)]
pub struct AliyunOssDriver {
    bucket: String,
    endpoint: String,
}

impl AliyunOssDriver {
    pub fn new(bucket: impl Into<String>, endpoint: impl Into<String>) -> Self {
        let mut ep = endpoint.into();
        if let Some(stripped) = ep.strip_prefix("https://") {
            ep = stripped.to_string();
        } else if let Some(stripped) = ep.strip_prefix("http://") {
            ep = stripped.to_string();
        }
        if ep.ends_with('/') {
            ep.pop();
        }

        Self {
            bucket: bucket.into(),
            endpoint: ep,
        }
    }
}

impl ImageDriver for AliyunOssDriver {
    fn build_url(&self, relative_path: &str, options: &ImageOptions) -> String {
        let clean_path = relative_path.trim_start_matches('/');
        let base_url = format!("https://{}.{}/{}", self.bucket, self.endpoint, clean_path);

        let mut actions = Vec::new();

        // 1. Resize 处理
        if options.width.is_some() || options.height.is_some() {
            let mut resize_parts = Vec::new();
            resize_parts.push("resize".to_string());

            if let Some(fit) = options.fit {
                let m = match fit {
                    ResizeFit::Cover => "m_fill",
                    ResizeFit::Contain => "m_fit",
                    ResizeFit::Fill => "m_mfit",
                    ResizeFit::Inside => "m_lfit",
                    ResizeFit::Outside => "m_mfit",
                };
                resize_parts.push(m.to_string());
            }

            if let Some(w) = options.width {
                resize_parts.push(format!("w_{w}"));
            }
            if let Some(h) = options.height {
                resize_parts.push(format!("h_{h}"));
            }

            actions.push(resize_parts.join(","));
        }

        // 2. Format 转换
        if let Some(fmt) = &options.format {
            actions.push(format!("format,{fmt}"));
        }

        // 3. 质量压缩
        if let Some(q) = options.quality {
            actions.push(format!("quality,q_{q}"));
        }

        if actions.is_empty() {
            base_url
        } else {
            let action_str = actions.join("/");
            format!("{base_url}?x-oss-process=image/{action_str}")
        }
    }
}
