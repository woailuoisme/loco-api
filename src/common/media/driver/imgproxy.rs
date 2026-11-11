//! # imgproxy 图片处理服务驱动 (imgproxy Image Driver)

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use super::{ImageDriver, ImageOptions, ResizeFit};

type HmacSha256 = Hmac<Sha256>;

/// imgproxy 高性能图片转换服务驱动
///
/// 支持安全 HMAC-SHA256 签名模式及 /unsafe/ 降级模式。
/// 生成规范格式：`{endpoint}/{signature}/{options}/plain/{source_url}`
#[derive(Debug, Clone)]
pub struct ImgproxyDriver {
    endpoint: String,
    key_bytes: Option<Vec<u8>>,
    salt_bytes: Option<Vec<u8>>,
    source_base_url: Option<String>,
}

impl ImgproxyDriver {
    pub fn new(
        endpoint: impl Into<String>,
        key_hex: Option<&str>,
        salt_hex: Option<&str>,
        source_base_url: Option<String>,
    ) -> Self {
        let mut ep = endpoint.into();
        if ep.ends_with('/') {
            ep.pop();
        }

        let key_bytes = key_hex
            .filter(|s| !s.trim().is_empty())
            .and_then(|k| hex::decode(k.trim()).ok());

        let salt_bytes = salt_hex
            .filter(|s| !s.trim().is_empty())
            .and_then(|s| hex::decode(s.trim()).ok());

        let mut base_url = source_base_url.filter(|s| !s.trim().is_empty());
        if let Some(ref mut b) = base_url
            && b.ends_with('/')
        {
            b.pop();
        }

        Self {
            endpoint: ep,
            key_bytes,
            salt_bytes,
            source_base_url: base_url,
        }
    }

    /// 对处理路径计算 imgproxy HMAC-SHA256 签名（Base64 URL 无填充格式）
    fn sign_path(&self, path_to_sign: &str) -> String {
        match (&self.key_bytes, &self.salt_bytes) {
            (Some(key), Some(salt)) => {
                if let Ok(mut mac) = HmacSha256::new_from_slice(key) {
                    mac.update(salt);
                    mac.update(path_to_sign.as_bytes());
                    let signature = mac.finalize().into_bytes();
                    URL_SAFE_NO_PAD.encode(signature)
                } else {
                    "unsafe".to_string()
                }
            }
            _ => "unsafe".to_string(),
        }
    }
}

impl ImageDriver for ImgproxyDriver {
    fn build_url(&self, relative_path: &str, options: &ImageOptions) -> String {
        let clean_path = relative_path.trim_start_matches('/');

        // 1. 构建源图 URL (若是完整 URL 则直传，否则拼接 source_base_url)
        let source_url = if clean_path.starts_with("http://")
            || clean_path.starts_with("https://")
            || clean_path.starts_with("s3://")
            || clean_path.starts_with("local://")
        {
            clean_path.to_string()
        } else if let Some(base) = &self.source_base_url {
            format!("{base}/{clean_path}")
        } else {
            format!("local:///{clean_path}")
        };

        // 2. 构建 imgproxy 处理指令
        let mut parts = Vec::new();

        // 尺寸与缩放模式 rs:{resizing_type}:{width}:{height}:{enlarge}:{extend}
        if options.width.is_some() || options.height.is_some() {
            let fit_str = match options.fit.unwrap_or(ResizeFit::Cover) {
                ResizeFit::Cover => "fill",
                ResizeFit::Contain => "fit",
                ResizeFit::Fill => "force",
                ResizeFit::Inside => "fit",
                ResizeFit::Outside => "fill",
            };
            let w = options.width.unwrap_or(0);
            let h = options.height.unwrap_or(0);
            parts.push(format!("rs:{fit_str}:{w}:{h}"));
        }

        // 质量与压缩
        if let Some(q) = options.quality {
            parts.push(format!("q:{q}"));
        }

        // 格式转换
        if let Some(fmt) = &options.format {
            parts.push(format!("ext:{fmt}"));
        }

        // 3. 构建待签名路径 /options/plain/source_url
        let options_segment = if parts.is_empty() {
            String::new()
        } else {
            format!("/{}", parts.join("/"))
        };

        let path_to_sign = format!("{options_segment}/plain/{source_url}");

        // 4. 计算签名并组合为最终 URL
        let signature = self.sign_path(&path_to_sign);
        format!("{}/{signature}{path_to_sign}", self.endpoint)
    }
}
