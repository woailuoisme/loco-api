//! # Centrifugo 实时推送与 Token 签发服务 (Realtime Service)

use std::time::Duration;

use cent::token::{self, ConnectionClaims, Key, SubscriptionClaims};
use cent::{Client, Config, PublishRequest, PublishResult};

use super::config::CentrifugoConfig;
use super::envelope::RealtimeMessage;

/// 实时服务领域错误枚举
#[derive(Debug, thiserror::Error)]
pub enum RealtimeError {
    #[error("Centrifugo client error: {0}")]
    Client(#[from] cent::Error),
    #[error("Token error: {0}")]
    Token(#[from] cent::token::TokenError),
    #[error("Payload error: {0}")]
    Payload(#[from] cent::PayloadError),
    #[error("Centrifugo is not configured")]
    NotConfigured,
    #[error("Invalid configuration: {0}")]
    Config(String),
}

impl From<RealtimeError> for loco_rs::Error {
    fn from(err: RealtimeError) -> Self {
        loco_rs::Error::wrap(err)
    }
}

/// Centrifugo 核心实时服务
#[derive(Clone)]
pub struct RealtimeService {
    client: Client,
    key: Key,
    config: CentrifugoConfig,
}

impl std::fmt::Debug for RealtimeService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealtimeService").field("config", &self.config).finish()
    }
}

impl RealtimeService {
    /// 构造新的 RealtimeService 实例
    pub fn new(config: CentrifugoConfig) -> Result<Self, RealtimeError> {
        if !config.is_configured() {
            return Err(RealtimeError::NotConfigured);
        }

        let key = Key::hmac(&config.token_hmac_secret)?;

        let mut client_cfg = Config::new(&config.api_endpoint).request_timeout(Duration::from_secs(config.timeout_sec));

        if !config.api_key.is_empty() {
            client_cfg = client_cfg.api_key(&config.api_key);
        }

        let client = Client::new(client_cfg)?;

        Ok(Self { client, key, config })
    }

    /// 获取只读配置引用
    pub fn config(&self) -> &CentrifugoConfig {
        &self.config
    }

    /// 获取底层 cent::Client 引用
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// 签发客户端 WebSocket 连接 JWT 凭证
    pub fn generate_connection_token(
        &self,
        user_id: impl Into<String>,
        ttl_secs: Option<u64>,
    ) -> Result<String, RealtimeError> {
        let ttl = Duration::from_secs(ttl_secs.unwrap_or(self.config.token_ttl_secs));
        let mut claims = ConnectionClaims::new(user_id, cent::token::Expiry::after(ttl));
        if !self.config.token_audience.is_empty() {
            claims.audience = self.config.token_audience.clone();
        }
        let jwt = token::connection(&self.key, &claims)?;
        Ok(jwt)
    }

    /// 签发特定频道客户端订阅凭证 (Subscription JWT)
    pub fn generate_subscription_token(
        &self,
        user_id: impl Into<String>,
        channel: impl Into<String>,
        ttl_secs: Option<u64>,
    ) -> Result<String, RealtimeError> {
        let ttl = Duration::from_secs(ttl_secs.unwrap_or(self.config.token_ttl_secs));
        let mut claims = SubscriptionClaims::new(user_id, channel, cent::token::Expiry::after(ttl));
        if !self.config.token_audience.is_empty() {
            claims.audience = self.config.token_audience.clone();
        }
        let jwt = token::subscription(&self.key, &claims)?;
        Ok(jwt)
    }

    /// 发布标准实时事件信封到指定频道
    pub async fn publish_message(&self, message: &RealtimeMessage) -> Result<PublishResult, RealtimeError> {
        let payload = cent::json(message)?;
        let req = PublishRequest::new(&message.channel).data(payload);
        let res = self.client.publish(req).await?;
        Ok(res)
    }

    /// 便捷方法：向指定频道发布具名事件
    pub async fn publish(
        &self,
        channel: impl Into<String>,
        event: impl Into<String>,
        payload: serde_json::Value,
    ) -> Result<PublishResult, RealtimeError> {
        let message = RealtimeMessage::new(channel, event, payload);
        self.publish_message(&message).await
    }
}
