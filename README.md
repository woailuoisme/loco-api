# loco-api

基于 Loco 框架 (Rust 2024 / Axum) 构建的生产级后端 API 服务，集成现代身份认证体系、多租户组织管理、多驱动云媒体处理、多态站内通知与流式数据引擎。

---

## 核心架构与特性

- **身份认证体系**：基于 Better Auth 规范设计，支持会话 (Session)、JWT 令牌、OAuth2 第三方关联、密码学凭据、Magic Link 免密登录、TOTP 二步验证 (2FA)、手机号验证码及 RBAC 角色权限控制。
- **多租户与组织管理**：多租户工作空间架构，内置组织 (Organization)、成员 (Member) 关联及成员邀请 (Invitation) 状态流转。
- **多驱动云媒体引擎**：媒体资源数据库仅存储标准相对路径，按需动态生成多规格变体 (`url`, `webp`, `thumb`, `cover`)。已接入 Direct、imgproxy (HMAC-SHA256 签名)、Cloudflare Images (`/cdn-cgi/image/`) 与 Aliyun OSS 驱动。
- **多态系统通知**：兼容 Laravel Notification 多态设计规范，支持邮件 (SMTP / Mailpit) 与 Telegram 等多通道分发及已读状态跟踪。
- **流式 Excel 引擎**：基于 `calamine` 与 `rust_xlsxwriter` 实现分块流式读取与大文件分块导出，保持低内存常驻。
- **全文检索集成**：集成 Meilisearch 搜索引擎，支持业务模型增量同步与全文检索。
- **API 契约与文档**：基于 Utoipa 自动生成 OpenAPI 3.0 规范，内置 Scalar 与 Redoc 交互式 UI，并通过 `ts-rs` 导出 TypeScript 类型定义。
- **双引擎 CI/CD 交付**：支持 GitHub Actions 与 CNB (Cloud Native Build) 自动化流水线，提供基于 `git-cliff` 的自动发版、多架构 Docker 镜像构建及 Helm Chart OCI 发布。

---

## 环境要求

- **Rust 工具链**：Rust 1.85+ (Edition 2024)
- **命令调度器**：`just` (推荐)
- **容器环境**：Docker & Docker Compose (用于编排 PostgreSQL、Valkey、Mailpit、imgproxy 等服务)
- **测试工具**：`cargo-nextest` (可选但推荐，已在 `justfile` 内置支持)

---

## 快速上手

### 1. 启动基础设施

通过 Docker Compose 启动本地依赖服务（PostgreSQL 18、Valkey 缓存/队列、Mailpit 邮件沙箱、imgproxy 图像服务）：

```sh
just up
# 或使用原生命令: docker compose up -d
```

### 2. 配置环境变量

从示例模板复制开发环境配置：

```sh
cp .env.example .env
```

> [!NOTE]
> 项目默认从当前目录的 `.env` 读取连接凭据。各环境专有应用配置位于 `config/development.yaml`、`config/test.yaml` 与 `config/production.yaml`。

### 3. 执行数据库迁移与种子数据

```sh
# 执行数据库迁移
just migrate

# (可选) 填充初始测试数据
just db-seed
```

### 4. 启动服务

```sh
# 普通启动
just dev

# 或监听文件变更热重载
just watch
```

默认服务监听地址：`http://127.0.0.1:5150`。健康检查接口返回：`{"status": "ok"}`。

---

## 常用命令速查

项目使用 `justfile` 管理所有工程化任务：

| 任务分类 | 命令 | 说明 |
| :--- | :--- | :--- |
| **服务启停** | `just dev` | 启动 Loco Web 服务 |
| | `just watch` | 启动开发服务并监听文件热重载 |
| | `just routes` | 打印当前注册的所有 HTTP 路由表 |
| | `just doctor` | 诊断数据库与缓存服务连通性 |
| **质量门禁** | `just fmt` | 执行 Rustfmt 格式化 |
| | `just lint` | 执行 Clippy 严格静态代码检查 (`-D warnings`) |
| | `just test` | 运行测试套件（优先调用 Nextest，保证 DB 测试串行） |
| | `just check` | 执行全量三阶门禁：`fmt-check` + `lint` + `test` |
| **数据库** | `just migrate` | 执行未应用的数据库迁移 |
| | `just db-status` | 查看当前数据库迁移版本状态 |
| | `just db-reset` | 重置数据库并重新执行全量迁移 |
| **维护瘦身** | `just sweep` | 智能精简 `target/` 目录，清理历史编译产物与增量缓存 |
| **容器编排** | `just up` | 后台启动 Docker Compose 服务 |
| | `just down` | 停止并移除 Docker Compose 容器 |
| | `just logs` | 跟踪 Docker Compose 运行日志 |

---

## API 文档与交互式界面

服务启动后，可通过以下内置端点查看 API 契约与调试接口：

- **OpenAPI JSON 规范**：`http://localhost:5150/api-docs/openapi.json`
- **Scalar 交互式界面**：`http://localhost:5150/scalar`
- **Redoc 文档界面**：`http://localhost:5150/redoc`

---

## 生产部署

### 容器镜像构建

```sh
# 构建 Debian Trixie 生产镜像
docker build -t loco-api:latest -f Dockerfile .
```

### Kubernetes (Helm) 部署

```sh
# Helm Chart 静态语法校验
helm lint ./helm/loco-api

# 安装或升级至 Kubernetes 集群
helm upgrade --install loco-api ./helm/loco-api \
  --namespace production \
  --create-namespace \
  --set image.tag="latest"
```
