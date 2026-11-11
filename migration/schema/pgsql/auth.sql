-- =============================================================================
-- Loco API 统一认证与多租户架构 Schema (PostgreSQL)
-- 对应迁移文件: migration/src/m20220101_000001_users.rs
-- 架构特性: Better Auth 体系、多租户 RBAC、软删除支持、2FA/TOTP、OAuth 账号关联
-- =============================================================================

-- 启用 UUID 生成扩展 (PostgreSQL 13+ 已内置 gen_random_uuid)
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- -----------------------------------------------------------------------------
-- 1. 用户核心档案表 ("user")
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS "user" (
    id VARCHAR(255) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    image TEXT,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    banned BOOLEAN NOT NULL DEFAULT false,
    two_factor_enabled BOOLEAN NOT NULL DEFAULT false,
    username VARCHAR(100),
    display_username VARCHAR(100),
    phone_number VARCHAR(50),
    phone_number_verified BOOLEAN NOT NULL DEFAULT false,
    is_anonymous BOOLEAN NOT NULL DEFAULT false,
    last_login_method VARCHAR(50),
    ban_reason TEXT,
    ban_expires TIMESTAMPTZ,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT check_user_status CHECK (status IN ('active', 'inactive', 'banned'))
);

-- "user" 表索引
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_email ON "user" (LOWER(email)) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_username ON "user" (LOWER(username)) WHERE deleted_at IS NULL AND username IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_phone_number ON "user" (phone_number) WHERE deleted_at IS NULL AND phone_number IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_user_role ON "user" (role) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_user_status ON "user" (status);
CREATE INDEX IF NOT EXISTS idx_user_deleted_at ON "user" (deleted_at);

COMMENT ON TABLE "user" IS '系统用户基础档案与认证状态表';
COMMENT ON COLUMN "user".id IS '用户全局唯一标识 (UUID)';
COMMENT ON COLUMN "user".role IS '系统全局角色 (如 user, admin, superadmin)';
COMMENT ON COLUMN "user".status IS '用户状态 (active, inactive, banned)';
COMMENT ON COLUMN "user".deleted_at IS '软删除标记时间戳';

-- -----------------------------------------------------------------------------
-- 2. 账号凭据与第三方 OAuth 关联表 ("account")
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS account (
    id VARCHAR(255) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id VARCHAR(255) NOT NULL,
    account_id VARCHAR(255) NOT NULL,
    provider_id VARCHAR(50) NOT NULL,
    password VARCHAR(255),
    access_token TEXT,
    refresh_token TEXT,
    id_token TEXT,
    access_token_expires_at TIMESTAMPTZ,
    refresh_token_expires_at TIMESTAMPTZ,
    scope TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_account_user_id FOREIGN KEY (user_id) REFERENCES "user"(id) ON DELETE CASCADE
);

-- account 表索引
CREATE INDEX IF NOT EXISTS idx_account_user_id ON account (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_account_provider_account ON account (provider_id, account_id);

COMMENT ON TABLE account IS '用户密码与第三方 OAuth2 提供方关联凭据表';
COMMENT ON COLUMN account.provider_id IS '提供商标识 (如 credential, github, google)';
COMMENT ON COLUMN account.account_id IS '提供商侧的用户唯一标识/主键';

-- -----------------------------------------------------------------------------
-- 3. 多租户组织与工作空间表 ("organization")
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS organization (
    id VARCHAR(255) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    logo TEXT,
    metadata TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- organization 表索引
CREATE UNIQUE INDEX IF NOT EXISTS idx_organization_slug ON organization (slug);

COMMENT ON TABLE organization IS '多租户组织/团队与工作空间表';
COMMENT ON COLUMN organization.slug IS '组织短标识 (唯一路径别名)';

-- -----------------------------------------------------------------------------
-- 4. 组织成员关联关系表 ("member")
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS member (
    id VARCHAR(255) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    organization_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_member_organization_id FOREIGN KEY (organization_id) REFERENCES organization(id) ON DELETE CASCADE,
    CONSTRAINT fk_member_user_id FOREIGN KEY (user_id) REFERENCES "user"(id) ON DELETE CASCADE
);

-- member 表索引
CREATE INDEX IF NOT EXISTS idx_member_organization_id ON member (organization_id);
CREATE INDEX IF NOT EXISTS idx_member_user_id ON member (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_member_org_user ON member (organization_id, user_id);

COMMENT ON TABLE member IS '组织与用户多对多归属关联表 (含租户内角色)';
COMMENT ON COLUMN member.role IS '租户内角色 (如 owner, admin, member)';

-- -----------------------------------------------------------------------------
-- 5. 组织成员邀请表 ("invitation")
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS invitation (
    id VARCHAR(255) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    organization_id VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    expires_at TIMESTAMPTZ NOT NULL,
    inviter_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT check_invitation_status CHECK (status IN ('pending', 'accepted', 'rejected', 'canceled')),
    CONSTRAINT fk_invitation_organization_id FOREIGN KEY (organization_id) REFERENCES organization(id) ON DELETE CASCADE,
    CONSTRAINT fk_invitation_inviter_id FOREIGN KEY (inviter_id) REFERENCES "user"(id) ON DELETE CASCADE
);

-- invitation 表索引
CREATE INDEX IF NOT EXISTS idx_invitation_organization_id ON invitation (organization_id);
CREATE INDEX IF NOT EXISTS idx_invitation_email ON invitation (LOWER(email));
CREATE INDEX IF NOT EXISTS idx_invitation_inviter_id ON invitation (inviter_id);
CREATE INDEX IF NOT EXISTS idx_invitation_expires_at ON invitation (expires_at);

COMMENT ON TABLE invitation IS '团队成员加入邀请与审核记录表';

-- -----------------------------------------------------------------------------
-- 6. 持久化会话管理表 ("session")
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS session (
    id VARCHAR(255) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id VARCHAR(255) NOT NULL,
    token VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    active_organization_id VARCHAR(255),
    impersonated_by VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_session_user_id FOREIGN KEY (user_id) REFERENCES "user"(id) ON DELETE CASCADE,
    CONSTRAINT fk_session_active_organization_id FOREIGN KEY (active_organization_id) REFERENCES organization(id) ON DELETE SET NULL,
    CONSTRAINT fk_session_impersonated_by FOREIGN KEY (impersonated_by) REFERENCES "user"(id) ON DELETE SET NULL
);

-- session 表索引
CREATE INDEX IF NOT EXISTS idx_session_user_id ON session (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_session_token ON session (token);
CREATE INDEX IF NOT EXISTS idx_session_expires_at ON session (expires_at);
CREATE INDEX IF NOT EXISTS idx_session_active_organization_id ON session (active_organization_id);
CREATE INDEX IF NOT EXISTS idx_session_impersonated_by ON session (impersonated_by);

COMMENT ON TABLE session IS '用户在线登录会话、模拟身份与当前活跃租户记录表';

-- -----------------------------------------------------------------------------
-- 7. 验证凭据与一次性口令表 ("verification")
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS verification (
    id VARCHAR(255) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    identifier VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- verification 表索引
CREATE INDEX IF NOT EXISTS idx_verification_identifier_value ON verification (identifier, value);
CREATE INDEX IF NOT EXISTS idx_verification_expires_at ON verification (expires_at);

COMMENT ON TABLE verification IS '一次性验证码凭据表 (如 MagicLink、邮箱验证、短信 OTP)';

-- -----------------------------------------------------------------------------
-- 8. TOTP 二步验证配置表 ("two_factor")
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS two_factor (
    id VARCHAR(255) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    user_id VARCHAR(255) NOT NULL,
    secret VARCHAR(255) NOT NULL,
    backup_codes TEXT NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT false,
    failed_verification_count INTEGER NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_two_factor_user_id FOREIGN KEY (user_id) REFERENCES "user"(id) ON DELETE CASCADE
);

-- two_factor 表索引
CREATE UNIQUE INDEX IF NOT EXISTS idx_two_factor_user_id ON two_factor (user_id);
CREATE INDEX IF NOT EXISTS idx_two_factor_secret ON two_factor (secret);

COMMENT ON TABLE two_factor IS '用户 TOTP 二步验证密钥、状态与恢复备用码表';

-- -----------------------------------------------------------------------------
-- 9. 站内系统通知表 ("notifications" - 兼容 Laravel Notification 规范)
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    type VARCHAR(255) NOT NULL,
    notifiable_type VARCHAR(255) NOT NULL,
    notifiable_id VARCHAR(255) NOT NULL,
    data TEXT NOT NULL,
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ
);

-- notifications 表索引
CREATE INDEX IF NOT EXISTS idx_notifications_notifiable ON notifications (notifiable_type, notifiable_id);
CREATE INDEX IF NOT EXISTS idx_notifications_unread ON notifications (notifiable_type, notifiable_id) WHERE read_at IS NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_notifications_read_at ON notifications (read_at);
CREATE INDEX IF NOT EXISTS idx_notifications_deleted_at ON notifications (deleted_at);

COMMENT ON TABLE notifications IS '系统业务与站内通知表 (Laravel 多态通知架构标准)';
COMMENT ON COLUMN notifications.id IS '通知唯一标识 (UUID)';
COMMENT ON COLUMN notifications.type IS '通知业务类型类名 (如 NotificationType 或 App\Notifications\...)';
COMMENT ON COLUMN notifications.notifiable_type IS '接收主体多态类型 (如 user, admin)';
COMMENT ON COLUMN notifications.notifiable_id IS '接收主体 ID';
COMMENT ON COLUMN notifications.data IS '通知载荷内容 (JSON 结构序列化字符串)';
COMMENT ON COLUMN notifications.read_at IS '已读时间戳 (为 NULL 表示未读)';
COMMENT ON COLUMN notifications.deleted_at IS '软删除标记时间戳';

