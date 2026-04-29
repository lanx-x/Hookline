# Hookline

一个 Rust HTTP 服务器，对外暴露 webhook endpoint，收到请求后向预配置的渠道发送通知。

## 背景

市面上已有的类似服务存在不足：

- **shoutrrr** — CLI 工具，只能本地调用，不适合作为服务部署
- **apprise** — 过于复杂，占用内存过大

Hookline 的定位是一个轻量、简单的 HTTP 通知网关。所有通知配置内置于服务端，调用方只需发送一个 HTTP 请求即可触发通知。这样在各种服务（CI/CD、监控、自动化脚本等）中只需填写一个 webhook 地址。

## 核心概念

```
调用方 --HTTP Request--> Hookline --通知--> 渠道 (Email / Discord / Telegram / ...)
```

### 三层配置模型

1. **全局配置** — 服务器端口、日志级别等
2. **Channel 配置** — 定义通知渠道的连接信息，可配置多个
3. **Endpoint 配置** — 定义对外暴露的 URL 路径，每个 endpoint 绑定一个或多个 channel

## 配置文件

使用 YAML 格式，示例结构：

```yaml
server:
  host: "0.0.0.0"
  port: 8080

channels:
  - name: "ops-email"
    type: email
    smtp_host: "smtp.example.com"
    smtp_port: 587
    format: html        # channel 级别的消息格式，由 channel 自身决定
    # ...

  - name: "dev-discord"
    type: discord
    webhook_url: "https://discord.com/api/webhooks/..."
    format: markdown
    # ...

endpoints:
  - path: "/alert/ci"
    token: "secret-token-xxx"    # 可选，用于鉴权
    channels:
      - "dev-discord"
      - "ops-email"

  - path: "/alert/monitor"
    token: "secret-token-yyy"
    channels:
      - "ops-email"

  # GitHub webhook 示例
  - path: "/webhook/github"
    token: "github-secret-xxx"
    channels:
      - "dev-discord"
```

## Endpoint

### HTTP Method

支持 `GET` 和 `POST`。

### 请求参数

| 参数 | 说明 |
|------|------|
| `title` | 通知标题 |
| `message` | 通知正文 |
| `to` | 指定接收方（如 email 地址，可选，覆盖 channel 默认值） |
| `from` | 指定发送方（可选） |
| `title_prefix` | 标题前缀（可选），拼接后为 `[prefix] title` |
| `title_path` | 从 JSON body 中提取 title，值为 JSON path，如 `.head_commit.message` |
| `message_path` | 从 JSON body 中提取 message，值为 JSON path |
| `level` | 通知级别（可选），`info`（绿色，默认）/ `warn`（黄色）/ `error`（红色） |

当请求携带 JSON body 但字段名不匹配时，调用方可通过 `title_path` / `message_path` 告诉 Hookline 从 body 的哪个字段取值。与直接传 `title` / `message` 互斥，`_path` 参数优先。

### 请求示例

```bash
# GET — 直接传 title 和 message
curl "http://localhost:8080/alert/ci?token=secret-token-xxx&title=Build+Failed&message=Pipeline+%23123+failed"

# POST — JSON body 直接匹配 title/message 字段
curl -X POST "http://localhost:8080/alert/ci?token=secret-token-xxx" \
  -H "Content-Type: application/json" \
  -d '{"title": "Build Failed", "message": "Pipeline #123 failed"}'

# POST — 第三方 payload，通过 _path 参数提取字段
curl -X POST "http://localhost:8080/webhook/github?token=github-secret-xxx&title_path=.head_commit.message&message_path=.body" \
  -H "Content-Type: application/json" \
  -d '{"ref": "refs/heads/main", "head_commit": {"message": "fix: update config"}, "body": "deployed"}'
#  → title = "fix: update config", message = "deployed"
```

### Token 鉴权

Endpoint 可配置 `token` 字段。启用后，请求必须携带正确的 token（通过 query param `?token=xxx` 或 `Authorization: Bearer xxx` header），否则返回 `401 Unauthorized`。

## 支持的渠道

| 渠道 | 类型标识 | 状态 |
|------|----------|------|
| Email | `email` | ✅ |
| Discord | `discord` | ✅ |
| Telegram | `telegram` | 计划中 |

## 响应格式

统一返回 JSON：

```json
{
  "status": "ok",
  "message": "notification sent"
}
```

错误情况：

| 状态码 | 说明 |
|--------|------|
| 400 | 参数缺失或无效 |
| 401 | Token 鉴权失败 |
| 404 | Endpoint 不存在 |
| 500 | 内部错误（如渠道发送失败） |

## 技术栈

- **语言**: Rust
- **配置**: YAML (serde_yaml)
- **异步运行时**: Tokio
