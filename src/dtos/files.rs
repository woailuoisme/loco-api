use crate::common::media::ImageVariants;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 文件上传响应 Payload
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FileUploadResource {
    /// 存储系统中的唯一文件相对路径 Key
    pub key: String,
    /// 原始文件名
    pub filename: String,
    /// 文件字节大小
    pub size: usize,
    /// HTTP Content-Type 类型
    pub content_type: Option<String>,
    /// 若上传文件为图片，则附带生成的多端图片优化规格对象
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<ImageVariants>,
}
