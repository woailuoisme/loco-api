use async_trait::async_trait;
use axum::{Json, Router, routing::get};
use loco_rs::{
    Result,
    app::{AppContext, Initializer},
};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_redoc::{Redoc, Servable as RedocServable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use crate::{
    common::{
        self,
        consts::openapi::{
            TAG_ADMIN, TAG_CREDENTIALS, TAG_FILES, TAG_HOME, TAG_ORG, TAG_REALTIME, TAG_SESSION, TAG_SIGNIN,
        },
    },
    controllers, dtos,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        controllers::auth::sign_up_email,
        controllers::auth::sign_in_email,
        controllers::auth::sign_in_username,
        controllers::auth::sign_in_anonymous,
        controllers::auth::get_session,
        controllers::auth::sign_out,
        controllers::auth::list_sessions,
        controllers::auth::revoke_session,
        controllers::auth::revoke_other_sessions,
        controllers::auth::set_active_session,
        controllers::auth::send_verification_otp,
        controllers::auth::check_verification_otp,
        controllers::auth::sign_in_email_otp,
        controllers::auth::verify_email,
        controllers::auth::reset_password,
        controllers::auth::sign_in_magic_link,
        controllers::auth::verify_magic_link,
        controllers::auth::send_phone_otp,
        controllers::auth::verify_phone_number,
        controllers::auth::sign_in_phone_number,
        controllers::auth::two_factor_enable,
        controllers::auth::two_factor_verify_totp,
        controllers::auth::two_factor_verify_backup_code,
        controllers::auth::two_factor_disable,
        controllers::auth::two_factor_generate_backup_codes,
        controllers::auth::sign_in_oauth2,
        controllers::auth::oauth2_callback,
        controllers::auth::one_tap_callback,
        controllers::auth::get_jwt_token,
        controllers::auth::get_jwks,
        controllers::auth::create_organization,
        controllers::auth::update_organization,
        controllers::auth::delete_organization,
        controllers::auth::get_full_organization,
        controllers::auth::list_organizations,
        controllers::auth::set_active_organization,
        controllers::auth::get_active_member,
        controllers::auth::invite_member,
        controllers::auth::accept_invitation,
        controllers::auth::reject_invitation,
        controllers::auth::cancel_invitation,
        controllers::auth::remove_member,
        controllers::auth::update_member_role,
        controllers::auth::leave_organization,
        controllers::auth::admin_set_role,
        controllers::auth::admin_ban_user,
        controllers::auth::admin_unban_user,
        controllers::auth::admin_impersonate_user,
        controllers::auth::admin_stop_impersonating,
        controllers::auth::admin_list_users,
        controllers::auth::admin_remove_user,
        controllers::files::upload,
        controllers::files::get_file,
        controllers::home::index,
        controllers::admin::auth::login,
        controllers::admin::auth::logout,
        controllers::admin::auth::me,
        controllers::admin::user::list,
        controllers::admin::user::get_one,
        controllers::admin::user::create,
        controllers::admin::user::update,
        controllers::admin::user::soft_delete,
        controllers::admin::user::ban,
        controllers::admin::user::unban,
        controllers::admin::user::export,
        controllers::admin::user::import,
        controllers::realtime::get_connection_token,
        controllers::realtime::get_subscription_token,
    ),
    components(
        schemas(
            dtos::files::FileUploadResource,
            common::media::ImageVariants,
            dtos::common::ApiError,
            controllers::home::HomeResource,
            dtos::admin::AdminListUsersQuery,
            dtos::admin::AdminCreateUserRequest,
            dtos::admin::AdminUpdateUserRequest,
            dtos::admin::AdminBanUserRequest,
            dtos::admin::NullableString,
            dtos::admin::AdminMeResource,
            dtos::admin::AdminUserResource,
            dtos::admin::UserImportRow,
            dtos::admin::UserImportSummary,
            dtos::admin::UserImportError,
            dtos::auth::SignUpEmailRequest,
            dtos::auth::SignInEmailRequest,
            dtos::auth::SignInUsernameRequest,
            dtos::auth::SignInAnonymousRequest,
            dtos::auth::RevokeSessionRequest,
            dtos::auth::SetActiveSessionRequest,
            dtos::auth::SendEmailOtpRequest,
            dtos::auth::CheckEmailOtpRequest,
            dtos::auth::SignInEmailOtpRequest,
            dtos::auth::VerifyEmailOtpRequest,
            dtos::auth::ResetPasswordEmailOtpRequest,
            dtos::auth::SignInMagicLinkRequest,
            dtos::auth::SendPhoneOtpRequest,
            dtos::auth::VerifyPhoneNumberRequest,
            dtos::auth::SignInPhoneNumberRequest,
            dtos::auth::TwoFactorPasswordRequest,
            dtos::auth::VerifyTotpRequest,
            dtos::auth::VerifyBackupCodeRequest,
            dtos::auth::UserResource,
            dtos::auth::SessionResource,
            dtos::auth::AuthResource,
            dtos::auth::SessionContextResource,
            dtos::auth::StatusResource,
            dtos::auth::VerifyEmailOtpResource,
            dtos::auth::TwoFactorEnableResource,
            dtos::auth::TwoFactorBackupCodesResource,
            dtos::auth::TwoFactorRedirectResource,
            dtos::auth::SignInResponse,
            dtos::auth::SignInOAuth2Request,
            dtos::auth::OAuth2UrlResource,
            dtos::auth::OAuth2CallbackRequest,
            dtos::auth::OneTapCallbackRequest,
            dtos::auth::JwtTokenResource,
            dtos::auth::JwksResource,
            dtos::auth::CreateOrganizationRequest,
            dtos::auth::UpdateOrganizationRequest,
            dtos::auth::UpdateOrganizationDataRequest,
            dtos::auth::SetActiveOrganizationRequest,
            dtos::auth::InviteMemberRequest,
            dtos::auth::AcceptInvitationRequest,
            dtos::auth::RejectInvitationRequest,
            dtos::auth::CancelInvitationRequest,
            dtos::auth::RemoveMemberRequest,
            dtos::auth::UpdateMemberRoleRequest,
            dtos::auth::LeaveOrganizationRequest,
            dtos::auth::OrganizationResource,
            dtos::auth::MemberResource,
            dtos::auth::ActiveMemberResource,
            dtos::auth::InvitationResource,
            dtos::auth::FullOrganizationResource,
            dtos::auth::AdminSetRoleRequest,
            dtos::auth::AdminBanUserRequest,
            dtos::auth::AdminUnbanUserRequest,
            dtos::auth::AdminImpersonateUserRequest,
            dtos::auth::AdminRemoveUserRequest,
            dtos::realtime::RealtimeTokenResource,
            dtos::realtime::ChannelSubscriptionRequest,
            dtos::realtime::ChannelSubscriptionResource,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = TAG_SESSION, description = "注册、登录与多会话管理"),
        (name = TAG_SIGNIN, description = "验证码、魔法链接与第三方登录"),
        (name = TAG_CREDENTIALS, description = "两步验证、通行密钥与 API 密钥"),
        (name = TAG_ORG, description = "多租户组织与成员邀请"),
        (name = TAG_ADMIN, description = "管理员用户治理操作"),
        (name = TAG_FILES, description = "文件上传与下载"),
        (name = TAG_REALTIME, description = "WebSocket 客户端凭据与实时通讯"),
        (name = TAG_HOME, description = "API 导航与版本信息"),
    ),
    info(
        title = "Loco RESTful API 文档",
        version = "1.0.0",
        description = "基于 utoipa 与 Scalar 驱动的 RESTful API 接口规范说明"
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

pub struct OpenApiInitializer;

#[async_trait]
impl Initializer for OpenApiInitializer {
    fn name(&self) -> String {
        "openapi".to_string()
    }

    async fn after_routes(&self, router: Router, _ctx: &AppContext) -> Result<Router> {
        let openapi = ApiDoc::openapi();

        let router = router
            .route(
                "/api-docs/openapi.json",
                get({
                    let openapi_json = openapi.clone();
                    move || async move { Json(openapi_json) }
                }),
            )
            .merge(Scalar::with_url("/docs", openapi.clone()).custom_html(
                r#"<!doctype html>
<html>
<head>
    <title>Loco RESTful API 文档</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1"/>
</head>
<body>

<script
        id="api-reference"
        data-configuration='{"hideModels": true}'
        type="application/json">
    $spec
</script>
<script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
</body>
</html>"#,
            ))
            .merge(Redoc::with_url("/redoc", openapi));

        Ok(router)
    }
}
