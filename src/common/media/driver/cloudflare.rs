//! # Cloudflare Images / Image Resizing 驱动 (Cloudflare Image Driver)

use super::{ImageDriver, ImageOptions, ResizeFit};

/// Cloudflare Images / Image Resizing 驱动
///
/// 生成规范格式：`https://{domain}/cdn-cgi/image/{params}/{source_path}`
#[derive(Debug, Clone)]
pub struct CloudflareDriver {
    domain: String,
}

impl CloudflareDriver {
    pub fn new(domain: impl Into<String>) -> Self {
        let mut d = domain.into();
        // 清理协议头与尾部斜杠以保持规范
        if let Some(stripped) = d.strip_prefix("https://") {
            d = stripped.to_string();
        } else if let Some(stripped) = d.strip_prefix("http://") {
            d = stripped.to_string();
        }
        if d.ends_with('/') {
            d.pop();
        }
        Self { domain: d }
    }
}

impl ImageDriver for CloudflareDriver {
    fn build_url(&self, relative_path: &str, options: &ImageOptions) -> String {
        let clean_path = relative_path.trim_start_matches('/');

        let mut params = Vec::new();

        if let Some(w) = options.width {
            params.push(format!("width={w}"));
        }
        if let Some(h) = options.height {
            params.push(format!("height={h}"));
        }
        if let Some(fit) = options.fit {
            let fit_str = match fit {
                ResizeFit::Cover => "cover",
                ResizeFit::Contain => "contain",
                ResizeFit::Fill => "scale-down",
                ResizeFit::Inside => "scale-down",
                ResizeFit::Outside => "crop",
            };
            params.push(format!("fit={fit_str}"));
        }
        if let Some(fmt) = &options.format {
            params.push(format!("format={fmt}"));
        }
        if let Some(q) = options.quality {
            params.push(format!("quality={q}"));
        }

        if params.is_empty() {
            // 原图直链
            format!("https://{}/{clean_path}", self.domain)
        } else {
            let param_str = params.join(",");
            format!("https://{}/cdn-cgi/image/{param_str}/{clean_path}", self.domain)
        }
    }
}
