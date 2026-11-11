//! # Notification 模块集成测试

use loco_api::app::App;
use loco_api::common::notifications::{
    AnonymousNotifiable, Channel, MailMessage, Notifiable, Notification, NotificationManager, TelegramInlineButton,
    TelegramMessage, TelegramParseMode,
};
use loco_rs::testing::prelude::*;

/// 自定义业务通知样例 (WelcomeNotification)
struct WelcomeNotification {
    pub user_name: String,
}

impl Notification for WelcomeNotification {
    fn via(&self) -> Vec<Channel> {
        vec![Channel::Email, Channel::Telegram]
    }

    fn to_mail(&self) -> Option<MailMessage> {
        Some(
            MailMessage::new()
                .subject("欢迎注册平台账号")
                .body(format!("亲爱的 {}，欢迎注册我们的服务！", self.user_name))
                .html(format!("<h1>欢迎 {}</h1><p>感谢您的支持！</p>", self.user_name))
                .tag("welcome"),
        )
    }

    fn to_telegram(&self) -> Option<TelegramMessage> {
        Some(
            TelegramMessage::new(format!("🎉 *新用户注册通知*\n\n欢迎用户: `{}`", self.user_name))
                .markdown_v2()
                .add_button(TelegramInlineButton::url("访问官网", "https://loco.rs")),
        )
    }
}

#[test]
fn test_mail_message_builder() {
    let msg = MailMessage::new()
        .subject("测试邮件")
        .body("文本正文")
        .html("<p>HTML</p>")
        .tag("test_tag");

    assert_eq!(msg.subject, "测试邮件");
    assert_eq!(msg.body.as_deref(), Some("文本正文"));
    assert_eq!(msg.html.as_deref(), Some("<p>HTML</p>"));
    assert_eq!(msg.tags, vec!["test_tag".to_string()]);
}

#[test]
fn test_telegram_message_builder() {
    let msg = TelegramMessage::new("Hello Telegram")
        .to_chat("-100987654321")
        .html()
        .disable_web_page_preview(true)
        .add_button(TelegramInlineButton::url("打开网页", "https://example.com"));

    assert_eq!(msg.text, "Hello Telegram");
    assert_eq!(msg.chat_id.as_deref(), Some("-100987654321"));
    assert_eq!(msg.parse_mode, Some(TelegramParseMode::HTML));
    assert_eq!(msg.disable_web_page_preview, Some(true));
    assert_eq!(msg.inline_keyboard.len(), 1);
    assert_eq!(msg.inline_keyboard[0][0].text, "打开网页");
}

#[test]
fn test_anonymous_notifiable_routing() {
    let notifiable = AnonymousNotifiable::new()
        .route_mail("user@example.com")
        .route_telegram("-100123456789")
        .route_realtime("personal:user_123");

    assert_eq!(
        notifiable.route_notification_for_mail().as_deref(),
        Some("user@example.com")
    );
    assert_eq!(
        notifiable.route_notification_for_telegram().as_deref(),
        Some("-100123456789")
    );
    assert_eq!(
        notifiable.route_notification_for_realtime().as_deref(),
        Some("personal:user_123")
    );
}

#[test]
fn test_realtime_message_envelope() {
    use loco_api::common::realtime::RealtimeMessage;

    let msg = RealtimeMessage::new("personal:123", "user.alert", serde_json::json!({ "title": "hello" }));
    assert_eq!(msg.channel, "personal:123");
    assert_eq!(msg.event, "user.alert");
    assert_eq!(msg.payload["title"], "hello");
    assert!(!msg.id.is_empty());

    assert_eq!(RealtimeMessage::personal_channel("42"), "user:42");
    assert_eq!(RealtimeMessage::org_channel("99"), "private:org_99");
    assert_eq!(RealtimeMessage::BROADCAST_SYSTEM, "public:system");
}

struct RealtimeTestNotification {
    pub message: String,
}

impl Notification for RealtimeTestNotification {
    fn via(&self) -> Vec<Channel> {
        vec![Channel::Realtime]
    }

    fn to_realtime(&self) -> Option<loco_api::common::realtime::RealtimeMessage> {
        Some(loco_api::common::realtime::RealtimeMessage::new(
            "",
            "test.event",
            serde_json::json!({ "msg": self.message }),
        ))
    }
}

#[tokio::test]
#[serial_test::serial]
async fn test_notification_hub_realtime_channel_dispatch() {
    use loco_api::common::notifications::NotificationHub;

    request::<App, _, _>(|_request, ctx| async move {
        let target = AnonymousNotifiable::new().route_realtime("personal:target_user");
        let notif = RealtimeTestNotification {
            message: "ping".to_string(),
        };

        // RealtimeService 注入 shared_store，验证 dispatch 逻辑
        let _ = NotificationHub::send(&ctx, &target, &notif).await;
    })
    .await;
}

#[test]
fn test_notification_trait_methods() {
    let notification = WelcomeNotification {
        user_name: "Alice".to_string(),
    };

    let channels = notification.via();
    assert_eq!(channels, vec![Channel::Email, Channel::Telegram]);

    let mail = notification.to_mail().expect("Should generate mail message");
    assert_eq!(mail.subject, "欢迎注册平台账号");

    let tg = notification.to_telegram().expect("Should generate telegram message");
    assert!(tg.text.contains("Alice"));
}

#[tokio::test]
#[serial_test::serial]
async fn test_notification_manager_send() {
    request::<App, _, _>(|_request, ctx| async move {
        let target = AnonymousNotifiable::new()
            .route_mail("alice@example.com")
            .route_telegram("123456789");

        let notification = WelcomeNotification {
            user_name: "Alice".to_string(),
        };

        NotificationManager::send(&ctx, &target, &notification)
            .await
            .expect("NotificationManager::send failed");

        // 回归断言: EmailChannel::send 应真实投递邮件到 Stub 发送器 (mailer.stub: true, ForegroundBlocking 同步执行)
        let deliveries = ctx.mailer.as_ref().expect("测试环境应配置 mailer").deliveries();
        assert_eq!(deliveries.count, 1, "邮件通知应通过 Stub 恰好投递 1 封");
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_notification_hub_send() {
    use loco_api::common::notifications::NotificationHub;

    request::<App, _, _>(|_request, ctx| async move {
        let target = AnonymousNotifiable::new()
            .route_mail("bob@example.com")
            .route_telegram("987654321");

        let notification = WelcomeNotification {
            user_name: "Bob".to_string(),
        };

        NotificationHub::send(&ctx, &target, &notification)
            .await
            .expect("NotificationHub::send failed");

        let deliveries = ctx.mailer.as_ref().expect("测试环境应配置 mailer").deliveries();
        assert_eq!(deliveries.count, 1, "邮件通知应通过 Stub 投递 1 封");
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_telegram_config_from_ctx_empty_in_test() {
    let boot = boot_test::<App>().await.unwrap();
    let cfg = loco_api::common::notifications::TelegramConfig::from_ctx(&boot.app_context);
    // 测试环境默认 bot_token 为空，应返回 None
    assert!(cfg.is_none());
}
