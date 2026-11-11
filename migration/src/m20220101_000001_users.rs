use sea_orm_migration::prelude::*;

#[derive(Debug)]
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_users"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 依次创建 9 张核心及常用扩展表
        Box::pin(create_user_table(manager)).await?;
        Box::pin(create_account_table(manager)).await?;
        Box::pin(create_organization_table(manager)).await?;
        Box::pin(create_member_table(manager)).await?;
        Box::pin(create_invitation_table(manager)).await?;
        Box::pin(create_session_table(manager)).await?;
        Box::pin(create_verification_table(manager)).await?;
        Box::pin(create_two_factor_table(manager)).await?;
        Box::pin(create_notifications_table(manager)).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 按外键依赖逆序级联删除
        manager
            .drop_table(
                Table::drop()
                    .table(Notifications::Table)
                    .if_exists()
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(TwoFactor::Table).if_exists().cascade().to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Verification::Table)
                    .if_exists()
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Session::Table).if_exists().cascade().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Invitation::Table).if_exists().cascade().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Member::Table).if_exists().cascade().to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Organization::Table)
                    .if_exists()
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Account::Table).if_exists().cascade().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(User::Table).if_exists().cascade().to_owned())
            .await?;

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 表构建辅助函数
// ─────────────────────────────────────────────────────────────────────────────

/// 1. 用户核心档案表 ("user")
async fn create_user_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(User::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(User::Id)
                        .string()
                        .not_null()
                        .default(Expr::cust("gen_random_uuid()::text"))
                        .primary_key(),
                )
                .col(ColumnDef::new(User::Name).string().not_null())
                .col(ColumnDef::new(User::Email).string().not_null())
                .col(ColumnDef::new(User::EmailVerified).boolean().not_null().default(false))
                .col(ColumnDef::new(User::Image).string().null())
                .col(ColumnDef::new(User::Role).string().not_null().default("user"))
                .col(ColumnDef::new(User::Status).string().not_null().default("active"))
                .col(ColumnDef::new(User::Banned).boolean().not_null().default(false))
                .col(
                    ColumnDef::new(User::TwoFactorEnabled)
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .col(ColumnDef::new(User::Username).string().null())
                .col(ColumnDef::new(User::DisplayUsername).string().null())
                .col(ColumnDef::new(User::PhoneNumber).string().null())
                .col(
                    ColumnDef::new(User::PhoneNumberVerified)
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .col(ColumnDef::new(User::IsAnonymous).boolean().not_null().default(false))
                .col(ColumnDef::new(User::LastLoginMethod).string().null())
                .col(ColumnDef::new(User::BanReason).string().null())
                .col(ColumnDef::new(User::BanExpires).timestamp_with_time_zone().null())
                .col(ColumnDef::new(User::LastLoginAt).timestamp_with_time_zone().null())
                .col(
                    ColumnDef::new(User::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(User::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(ColumnDef::new(User::DeletedAt).timestamp_with_time_zone().null())
                .check(Expr::cust("status IN ('active', 'inactive', 'banned')"))
                .to_owned(),
        )
        .await?;

    // 索引声明
    manager
        .create_index(
            Index::create()
                .name("idx_user_email")
                .table(User::Table)
                .col(Expr::cust("LOWER(email)"))
                .and_where(Expr::col(User::DeletedAt).is_null())
                .unique()
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_user_username")
                .table(User::Table)
                .col(Expr::cust("LOWER(username)"))
                .and_where(Expr::col(User::DeletedAt).is_null())
                .and_where(Expr::col(User::Username).is_not_null())
                .unique()
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_user_phone_number")
                .table(User::Table)
                .col(User::PhoneNumber)
                .and_where(Expr::col(User::DeletedAt).is_null())
                .and_where(Expr::col(User::PhoneNumber).is_not_null())
                .unique()
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_user_role")
                .table(User::Table)
                .col(User::Role)
                .and_where(Expr::col(User::DeletedAt).is_null())
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_user_status")
                .table(User::Table)
                .col(User::Status)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_user_deleted_at")
                .table(User::Table)
                .col(User::DeletedAt)
                .to_owned(),
        )
        .await?;

    Ok(())
}

/// 2. 账号凭据与第三方 OAuth 关联表 ("account")
async fn create_account_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Account::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Account::Id)
                        .string()
                        .not_null()
                        .default(Expr::cust("gen_random_uuid()::text"))
                        .primary_key(),
                )
                .col(ColumnDef::new(Account::UserId).string().not_null())
                .col(ColumnDef::new(Account::AccountId).string().not_null())
                .col(ColumnDef::new(Account::ProviderId).string().not_null())
                .col(ColumnDef::new(Account::Password).string().null())
                .col(ColumnDef::new(Account::AccessToken).string().null())
                .col(ColumnDef::new(Account::RefreshToken).string().null())
                .col(ColumnDef::new(Account::IdToken).string().null())
                .col(
                    ColumnDef::new(Account::AccessTokenExpiresAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(Account::RefreshTokenExpiresAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(ColumnDef::new(Account::Scope).string().null())
                .col(
                    ColumnDef::new(Account::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Account::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_account_user_id")
                        .from(Account::Table, Account::UserId)
                        .to(User::Table, User::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_account_user_id")
                .table(Account::Table)
                .col(Account::UserId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_account_provider_account")
                .table(Account::Table)
                .col(Account::ProviderId)
                .col(Account::AccountId)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

/// 3. 多租户组织与工作空间表 ("organization")
async fn create_organization_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Organization::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Organization::Id)
                        .string()
                        .not_null()
                        .default(Expr::cust("gen_random_uuid()::text"))
                        .primary_key(),
                )
                .col(ColumnDef::new(Organization::Name).string().not_null())
                .col(ColumnDef::new(Organization::Slug).string().not_null())
                .col(ColumnDef::new(Organization::Logo).string().null())
                .col(ColumnDef::new(Organization::Metadata).string().null())
                .col(
                    ColumnDef::new(Organization::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Organization::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_organization_slug")
                .table(Organization::Table)
                .col(Organization::Slug)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

/// 4. 组织成员关联关系表 ("member")
async fn create_member_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Member::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Member::Id)
                        .string()
                        .not_null()
                        .default(Expr::cust("gen_random_uuid()::text"))
                        .primary_key(),
                )
                .col(ColumnDef::new(Member::OrganizationId).string().not_null())
                .col(ColumnDef::new(Member::UserId).string().not_null())
                .col(ColumnDef::new(Member::Role).string().not_null().default("member"))
                .col(
                    ColumnDef::new(Member::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Member::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_member_organization_id")
                        .from(Member::Table, Member::OrganizationId)
                        .to(Organization::Table, Organization::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_member_user_id")
                        .from(Member::Table, Member::UserId)
                        .to(User::Table, User::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_member_organization_id")
                .table(Member::Table)
                .col(Member::OrganizationId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_member_user_id")
                .table(Member::Table)
                .col(Member::UserId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_member_org_user")
                .table(Member::Table)
                .col(Member::OrganizationId)
                .col(Member::UserId)
                .unique()
                .to_owned(),
        )
        .await?;

    Ok(())
}

/// 5. 组织成员邀请表 ("invitation")
async fn create_invitation_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Invitation::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Invitation::Id)
                        .string()
                        .not_null()
                        .default(Expr::cust("gen_random_uuid()::text"))
                        .primary_key(),
                )
                .col(ColumnDef::new(Invitation::OrganizationId).string().not_null())
                .col(ColumnDef::new(Invitation::Email).string().not_null())
                .col(ColumnDef::new(Invitation::Role).string().not_null().default("member"))
                .col(
                    ColumnDef::new(Invitation::Status)
                        .string()
                        .not_null()
                        .default("pending"),
                )
                .col(
                    ColumnDef::new(Invitation::ExpiresAt)
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .col(ColumnDef::new(Invitation::InviterId).string().not_null())
                .col(
                    ColumnDef::new(Invitation::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Invitation::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .check(Expr::cust("status IN ('pending', 'accepted', 'rejected', 'canceled')"))
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_invitation_organization_id")
                        .from(Invitation::Table, Invitation::OrganizationId)
                        .to(Organization::Table, Organization::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_invitation_inviter_id")
                        .from(Invitation::Table, Invitation::InviterId)
                        .to(User::Table, User::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_invitation_organization_id")
                .table(Invitation::Table)
                .col(Invitation::OrganizationId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_invitation_email")
                .table(Invitation::Table)
                .col(Expr::cust("LOWER(email)"))
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_invitation_inviter_id")
                .table(Invitation::Table)
                .col(Invitation::InviterId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_invitation_expires_at")
                .table(Invitation::Table)
                .col(Invitation::ExpiresAt)
                .to_owned(),
        )
        .await?;

    Ok(())
}

/// 6. 持久化会话管理表 ("session")
async fn create_session_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Session::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Session::Id)
                        .string()
                        .not_null()
                        .default(Expr::cust("gen_random_uuid()::text"))
                        .primary_key(),
                )
                .col(ColumnDef::new(Session::UserId).string().not_null())
                .col(ColumnDef::new(Session::Token).string().not_null())
                .col(ColumnDef::new(Session::ExpiresAt).timestamp_with_time_zone().not_null())
                .col(ColumnDef::new(Session::IpAddress).string().null())
                .col(ColumnDef::new(Session::UserAgent).string().null())
                .col(ColumnDef::new(Session::ActiveOrganizationId).string().null())
                .col(ColumnDef::new(Session::ImpersonatedBy).string().null())
                .col(
                    ColumnDef::new(Session::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Session::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_session_user_id")
                        .from(Session::Table, Session::UserId)
                        .to(User::Table, User::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_session_active_organization_id")
                        .from(Session::Table, Session::ActiveOrganizationId)
                        .to(Organization::Table, Organization::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_session_impersonated_by")
                        .from(Session::Table, Session::ImpersonatedBy)
                        .to(User::Table, User::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_session_user_id")
                .table(Session::Table)
                .col(Session::UserId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_session_token")
                .table(Session::Table)
                .col(Session::Token)
                .unique()
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_session_expires_at")
                .table(Session::Table)
                .col(Session::ExpiresAt)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_session_active_organization_id")
                .table(Session::Table)
                .col(Session::ActiveOrganizationId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_session_impersonated_by")
                .table(Session::Table)
                .col(Session::ImpersonatedBy)
                .to_owned(),
        )
        .await?;

    Ok(())
}

/// 7. 验证凭据与一次性口令表 ("verification")
async fn create_verification_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Verification::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Verification::Id)
                        .string()
                        .not_null()
                        .default(Expr::cust("gen_random_uuid()::text"))
                        .primary_key(),
                )
                .col(ColumnDef::new(Verification::Identifier).string().not_null())
                .col(ColumnDef::new(Verification::Value).string().not_null())
                .col(
                    ColumnDef::new(Verification::ExpiresAt)
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Verification::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Verification::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_verification_identifier_value")
                .table(Verification::Table)
                .col(Verification::Identifier)
                .col(Verification::Value)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_verification_expires_at")
                .table(Verification::Table)
                .col(Verification::ExpiresAt)
                .to_owned(),
        )
        .await?;

    Ok(())
}

/// 8. TOTP 二步验证配置表 ("two_factor")
async fn create_two_factor_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(TwoFactor::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(TwoFactor::Id)
                        .string()
                        .not_null()
                        .default(Expr::cust("gen_random_uuid()::text"))
                        .primary_key(),
                )
                .col(ColumnDef::new(TwoFactor::UserId).string().not_null())
                .col(ColumnDef::new(TwoFactor::Secret).string().not_null())
                .col(ColumnDef::new(TwoFactor::BackupCodes).string().not_null())
                // 默认 false：pending 状态需 TOTP 首验通过后才由 confirm_enabled 翻转
                .col(ColumnDef::new(TwoFactor::Verified).boolean().not_null().default(false))
                .col(
                    ColumnDef::new(TwoFactor::FailedVerificationCount)
                        .integer()
                        .not_null()
                        .default(0),
                )
                .col(ColumnDef::new(TwoFactor::LockedUntil).timestamp_with_time_zone().null())
                .col(
                    ColumnDef::new(TwoFactor::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(TwoFactor::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_two_factor_user_id")
                        .from(TwoFactor::Table, TwoFactor::UserId)
                        .to(User::Table, User::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_two_factor_user_id")
                .table(TwoFactor::Table)
                .col(TwoFactor::UserId)
                .unique()
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_two_factor_secret")
                .table(TwoFactor::Table)
                .col(TwoFactor::Secret)
                .to_owned(),
        )
        .await?;

    Ok(())
}

/// 9. 站内系统通知表 ("notifications" - 兼容 Laravel Notification 规范)
async fn create_notifications_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Notifications::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Notifications::Id)
                        .uuid()
                        .not_null()
                        .default(Expr::cust("gen_random_uuid()"))
                        .primary_key(),
                )
                .col(ColumnDef::new(Notifications::Type).string().not_null())
                .col(ColumnDef::new(Notifications::NotifiableType).string().not_null())
                .col(ColumnDef::new(Notifications::NotifiableId).string().not_null())
                .col(ColumnDef::new(Notifications::Data).text().not_null())
                .col(ColumnDef::new(Notifications::ReadAt).timestamp_with_time_zone().null())
                .col(
                    ColumnDef::new(Notifications::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Notifications::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Notifications::DeletedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .to_owned(),
        )
        .await?;

    // 索引声明
    manager
        .create_index(
            Index::create()
                .name("idx_notifications_notifiable")
                .table(Notifications::Table)
                .col(Notifications::NotifiableType)
                .col(Notifications::NotifiableId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_notifications_unread")
                .table(Notifications::Table)
                .col(Notifications::NotifiableType)
                .col(Notifications::NotifiableId)
                .and_where(Expr::col(Notifications::ReadAt).is_null())
                .and_where(Expr::col(Notifications::DeletedAt).is_null())
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_notifications_read_at")
                .table(Notifications::Table)
                .col(Notifications::ReadAt)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_notifications_deleted_at")
                .table(Notifications::Table)
                .col(Notifications::DeletedAt)
                .to_owned(),
        )
        .await?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// 强类型 DeriveIden 表与字段枚举
// ─────────────────────────────────────────────────────────────────────────────

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Name,
    Email,
    EmailVerified,
    Image,
    Role,
    Status,
    Banned,
    TwoFactorEnabled,
    Username,
    DisplayUsername,
    PhoneNumber,
    PhoneNumberVerified,
    IsAnonymous,
    LastLoginMethod,
    BanReason,
    BanExpires,
    LastLoginAt,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
enum Account {
    Table,
    Id,
    UserId,
    AccountId,
    ProviderId,
    Password,
    AccessToken,
    RefreshToken,
    IdToken,
    AccessTokenExpiresAt,
    RefreshTokenExpiresAt,
    Scope,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Organization {
    Table,
    Id,
    Name,
    Slug,
    Logo,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Member {
    Table,
    Id,
    OrganizationId,
    UserId,
    Role,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Invitation {
    Table,
    Id,
    OrganizationId,
    Email,
    Role,
    Status,
    ExpiresAt,
    InviterId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Session {
    Table,
    Id,
    UserId,
    Token,
    ExpiresAt,
    IpAddress,
    UserAgent,
    ActiveOrganizationId,
    ImpersonatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Verification {
    Table,
    Id,
    Identifier,
    Value,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum TwoFactor {
    Table,
    Id,
    UserId,
    Secret,
    BackupCodes,
    Verified,
    FailedVerificationCount,
    LockedUntil,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Notifications {
    Table,
    Id,
    Type,
    NotifiableType,
    NotifiableId,
    Data,
    ReadAt,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
