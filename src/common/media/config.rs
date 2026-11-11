//! # 媒体服务配置对象 (Media Configuration)

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::driver::{AliyunOssDriver, CloudflareDriver, DirectDriver, ImageDriver, ImgproxyDriver};

/// 支持的媒体驱动类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MediaDriverKind {
    #[default]
    Direct,
    Imgproxy,
    Cloudflare,
    AliyunOss,
}

/// imgproxy 驱动专属配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImgproxyConfig {
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub salt: Option<String>,
    #[serde(default)]
    pub source_base_url: Option<String>,
}

/// Cloudflare Images 驱动专属配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CloudflareConfig {
    #[serde(default)]
    pub domain: String,
}

/// 阿里云 OSS 图片处理驱动专属配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AliyunOssConfig {
    #[serde(default)]
    pub bucket: String,
    #[serde(default)]
    pub endpoint: String,
}

/// 媒体模块顶级配置对象
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaConfig {
    /// 全局激活驱动
    #[serde(default)]
    pub driver: MediaDriverKind,
    /// 默认基础资源域名或直链前缀
    #[serde(default)]
    pub base_url: Option<String>,
    /// imgproxy 详细配置
    #[serde(default)]
    pub imgproxy: Option<ImgproxyConfig>,
    /// Cloudflare 详细配置
    #[serde(default)]
    pub cloudflare: Option<CloudflareConfig>,
    /// 阿里云 OSS 详细配置
    #[serde(default)]
    pub aliyun_oss: Option<AliyunOssConfig>,
}

impl MediaConfig {
    /// 从 AppContext 中的 settings: media 配置块解析加载
    pub fn from_ctx(ctx: &loco_rs::prelude::AppContext) -> Self {
        #[derive(Deserialize)]
        struct SettingsHolder {
            media: Option<MediaConfig>,
        }

        ctx.config
            .settings::<SettingsHolder>()
            .ok()
            .and_then(|h| h.media)
            .unwrap_or_default()
    }

    /// 根据配置实例化对应的全局 ImageDriver 实例
    pub fn build_driver(&self) -> Arc<dyn ImageDriver> {
        match self.driver {
            MediaDriverKind::Direct => {
                let base = self.base_url.as_deref().unwrap_or("");
                Arc::new(DirectDriver::new(base))
            }
            MediaDriverKind::Imgproxy => {
                let cfg = self.imgproxy.clone().unwrap_or_default();
                let source_base = cfg.source_base_url.or_else(|| self.base_url.clone());
                Arc::new(ImgproxyDriver::new(
                    cfg.endpoint,
                    cfg.key.as_deref(),
                    cfg.salt.as_deref(),
                    source_base,
                ))
            }
            MediaDriverKind::Cloudflare => {
                let cfg = self.cloudflare.clone().unwrap_or_default();
                Arc::new(CloudflareDriver::new(cfg.domain))
            }
            MediaDriverKind::AliyunOss => {
                let cfg = self.aliyun_oss.clone().unwrap_or_default();
                Arc::new(AliyunOssDriver::new(cfg.bucket, cfg.endpoint))
            }
        }
    }
}
