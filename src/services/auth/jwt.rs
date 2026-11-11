use chrono::Utc;
use loco_rs::prelude::*;

use crate::{
    dtos::auth::*,
    errors::{AppError, AppResult},
    models::user,
};

use super::SessionService;

impl SessionService {
    pub async fn get_jwt_token(ctx: &AppContext, user_id: &str) -> AppResult<JwtTokenResource> {
        let Some(user) = user::Entity::find_by_id(&ctx.db, user_id).await? else {
            return Err(AppError::not_found("User not found"));
        };

        let secret = ctx
            .config
            .auth
            .as_ref()
            .and_then(|a| a.jwt.as_ref())
            .map_or("loco-api-default-jwt-secret", |a| a.secret.as_str());

        #[derive(serde::Serialize)]
        struct Claims<'a> {
            sub: &'a str,
            email: &'a str,
            name: &'a str,
            role: &'a str,
            exp: usize,
            iat: usize,
            iss: &'a str,
            aud: &'a str,
        }

        let now = Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: &user.id,
            email: &user.email,
            name: &user.name,
            role: &user.role,
            exp: now + crate::common::consts::auth::JWT_EXPIRATION_SECS,
            iat: now,
            iss: "loco-auth",
            aud: "loco-client",
        };

        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256);
        let token = jsonwebtoken::encode(
            &header,
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| {
            tracing::error!(error = %e, "JWT encoding failed");
            AppError::internal("Internal token signing error")
        })?;

        Ok(JwtTokenResource { token })
    }

    /// 获取公开 JWKS 密钥集（适配 Better-Auth 契约元数据发现）
    ///
    /// 注意：系统使用 HS256 对称签名算法，根据 RFC 7517 安全规范，
    /// 公开 JWKS 中严禁泄露对称密钥明文（故意省略 `k` 属性以防密钥外泄）。
    /// 若需第三方公共服务离线验签，后续可演进为 RS256/EdDSA 非对称公钥。
    pub async fn get_jwks(_ctx: &AppContext) -> AppResult<JwksResource> {
        let key = serde_json::json!({
            "kty": "oct",
            "alg": "HS256",
            "use": "sig",
            "kid": "default",
        });
        Ok(JwksResource { keys: vec![key] })
    }
}
