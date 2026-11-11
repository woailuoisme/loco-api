//! # 实时通讯频道 (Channel) 领域模型
//!
//! 定义实时频道强类型枚举，统一管理频道的命名空间规约、别名映射与订阅权限判定。

use loco_rs::prelude::ModelError;
use sea_orm::DatabaseConnection;

use crate::models::{_entities::user, member::Member};

/// 实时通讯频道领域对象
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RealtimeChannel {
    /// 用户私人频道 (`user:{user_id}`)
    User(String),
    /// 组织专属频道 (`private:org_{org_id}`)
    Organization(String),
    /// 公共广播或系统通知频道 (`public:{name}`)
    Public(String),
}

impl RealtimeChannel {
    /// 解析原始频道字符串，并自动规约为标准化频道类型
    pub fn parse(raw: &str) -> Option<Self> {
        if let Some(uid) = raw.strip_prefix("user:").or_else(|| raw.strip_prefix("personal:")) {
            Some(Self::User(uid.to_string()))
        } else if let Some(org_id) = raw.strip_prefix("private:org_").or_else(|| raw.strip_prefix("org:")) {
            Some(Self::Organization(org_id.to_string()))
        } else {
            raw.strip_prefix("public:")
                .or_else(|| raw.strip_prefix("broadcast:"))
                .or_else(|| raw.strip_prefix("notification:"))
                .map(|name| Self::Public(name.to_string()))
        }
    }

    /// 校验指定用户是否有权订阅此频道
    pub async fn can_subscribe(&self, user: &user::Model, db: &DatabaseConnection) -> Result<bool, ModelError> {
        if user.role == "admin" {
            return Ok(true);
        }

        match self {
            Self::User(target_user_id) => Ok(target_user_id == &user.id),
            Self::Organization(org_id) => {
                let member = Member::find_by_org_and_user(db, org_id, &user.id).await?;
                Ok(member.is_some())
            }
            Self::Public(_) => Ok(true),
        }
    }
}
