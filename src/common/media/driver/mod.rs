//! # 媒体图片多驱动抽象 (Media Image Driver)
//!
//! 支持 imgproxy、Cloudflare Images、Aliyun OSS 与 Direct 本地直连驱动。
//! 数据库落库保存相对路径，输出时由激活驱动生成对应的 WebP、缩略图、封面图等优化 URL。

pub mod aliyun;
pub mod cloudflare;
pub mod direct;
pub mod imgproxy;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use aliyun::AliyunOssDriver;
pub use cloudflare::CloudflareDriver;
pub use direct::DirectDriver;
pub use imgproxy::ImgproxyDriver;

/// 图像优化预设类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImagePreset {
    /// 原图
    Original,
    /// WebP 格式优化
    Webp,
    /// 缩略图（通常 150x150 cover webp）
    Thumb,
    /// 封面大图（通常 800x450 cover webp）
    Cover,
}

/// 裁剪缩放模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResizeFit {
    /// 居中填充并裁剪多余部分
    Cover,
    /// 完整保留在边界内（可能留白或保持原比例）
    Contain,
    /// 拉伸填充目标宽高
    Fill,
    /// 缩放至内接
    Inside,
    /// 缩放至外切
    Outside,
}

/// 图片处理参数选项
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fit: Option<ResizeFit>,
    pub format: Option<String>,
    pub quality: Option<u8>,
}

impl ImageOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_preset(preset: ImagePreset) -> Self {
        match preset {
            ImagePreset::Original => Self::default(),
            ImagePreset::Webp => Self {
                format: Some("webp".to_string()),
                quality: Some(85),
                ..Default::default()
            },
            ImagePreset::Thumb => Self {
                width: Some(150),
                height: Some(150),
                fit: Some(ResizeFit::Cover),
                format: Some("webp".to_string()),
                quality: Some(80),
            },
            ImagePreset::Cover => Self {
                width: Some(800),
                height: Some(450),
                fit: Some(ResizeFit::Cover),
                format: Some("webp".to_string()),
                quality: Some(85),
            },
        }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn with_fit(mut self, fit: ResizeFit) -> Self {
        self.fit = Some(fit);
        self
    }

    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = Some(quality);
        self
    }
}

/// 前端多规格图片资源对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImageVariants {
    /// 数据库存储的相对路径
    pub path: String,
    /// 原始文件完整直链
    pub url: String,
    /// WebP 格式优化链接
    pub webp: String,
    /// 缩略图（通常 150x150 cover webp）
    pub thumb: String,
    /// 封面大图（通常 800x450 cover webp）
    pub cover: String,
}

/// 图片驱动核心 Trait
pub trait ImageDriver: Send + Sync {
    /// 根据相对路径与自定义选项生成最终优化 URL
    fn build_url(&self, relative_path: &str, options: &ImageOptions) -> String;

    /// 根据相对路径与预设类型生成优化 URL
    fn url_with_preset(&self, relative_path: &str, preset: ImagePreset) -> String {
        let opts = ImageOptions::from_preset(preset);
        self.build_url(relative_path, &opts)
    }

    /// 一键生成全规格 ImageVariants 结构
    fn build_variants(&self, relative_path: &str) -> ImageVariants {
        let clean_path = relative_path.trim_start_matches('/');
        ImageVariants {
            path: clean_path.to_string(),
            url: self.url_with_preset(clean_path, ImagePreset::Original),
            webp: self.url_with_preset(clean_path, ImagePreset::Webp),
            thumb: self.url_with_preset(clean_path, ImagePreset::Thumb),
            cover: self.url_with_preset(clean_path, ImagePreset::Cover),
        }
    }
}
