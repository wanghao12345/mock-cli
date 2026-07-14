<div align="center">

# mock-cli

**极速 OpenAPI Mock 服务器（基于 Rust）**
毫秒级启动，零配置，动态生成假数据。

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](#license)
[![CI](https://github.com/wanghao12345/mock-cli/actions/workflows/release.yml/badge.svg)](https://github.com/wanghao12345/mock-cli/actions)
[![npm version](https://img.shields.io/npm/v/@mock-cli/server.svg)](https://www.npmjs.com/package/@mock-cli/server)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

[English](README.md) | **简体中文**

[安装](#安装) • [快速开始](#快速开始) • [功能特性](#功能特性) • [CLI 参考](#cli-参考) • [路线图](#路线图)

</div>

---

## 为什么选择 mock-cli？

大多数 OpenAPI Mock 服务器启动慢、依赖 Docker、或只能返回静态假数据。`mock-cli` 不一样：

- **极速启动** — 原生 Rust 二进制，无 JVM/Node/Docker 开销
- **动态假数据** — 根据 Schema 类型（字符串格式、枚举、数组、对象）生成真实的随机数据
- **路径感知** — 路径参数会被提取并回显到响应中（当字段名匹配时）
- **Schema 感知** — 遵守 `format`（email/uuid/date-time/uri/...）、`enum`、`required`、`minItems`/`maxItems`
- **正确的状态码** — 未知路径返回 404，不支持的方法返回 405，成功返回 200
- **循环安全** — 递归 `$ref` 循环有深度限制，服务器永不崩溃
- **单二进制** — 一个静态二进制，离线可用，无需运行时
- **跨平台** — macOS / Linux / Windows，原生安装包


## 安装

### npm（推荐）

```bash
npm install -g @mock-cli/server
```

npm 包会自动安装匹配你平台的预编译二进制（macOS / Linux / Windows，x64 / arm64）。无需 Rust 工具链。

### 预编译二进制

从 [GitHub Releases](https://github.com/wanghao12345/mock-cli/releases) 下载对应平台的最新版本。

### 从源码构建

```bash
git clone https://github.com/wanghao12345/mock-cli.git
cd mock-cli
cargo install --path crates/cli
```

### 用 cargo 直接运行

```bash
cargo run --release -- examples/swagger.yaml
```


## 快速开始

**1. 使用内置示例 spec：**

```bash
mock-cli examples/swagger.yaml
```

或者创建你自己的 `swagger.yaml`：

```yaml
openapi: 3.0.3
info:
  title: Petstore
  version: 1.0.0
paths:
  /pets/{petId}:
    get:
      parameters:
        - name: petId
          in: path
          required: true
          schema:
            type: integer
      responses:
        '200':
          description: OK
          content:
            application/json:
              schema:
                type: object
                properties:
                  petId:
                    type: integer
                  name:
                    type: string
                  vaccinated:
                    type: boolean
```

**2. 启动 Mock 服务器：**

```bash
mock-cli swagger.yaml
```

```text
✓ Loaded "swagger.yaml" (1 paths)
✓ Listening on 127.0.0.1:3333
```

**3. 获取动态假数据：**

```bash
curl http://localhost:3333/pets/123
# {"petId":123,"name":"Dr. Margret Kihn","vaccinated":true}

curl http://localhost:3333/pets/456
# {"petId":456,"name":"Sarah Connor","vaccinated":false}
```

注意：响应中的 `petId` 回显了路径参数值 `123` / `456`（已强制转换为 Schema 类型 `integer`）。详见[路径参数注入](#路径参数注入)。每次请求都返回**不同的、符合 Schema 的**假数据，不是静态假数据。


## 功能特性

### Schema 驱动的假数据

| OpenAPI 构造 | 行为 |
|-------------------|----------|
| `type: string` + `format: email` | 生成真实的邮箱地址 |
| `type: string` + `format: uuid` | 生成 v4 UUID |
| `type: string` + `format: date` / `date-time` | 生成 ISO 8601 日期 / 日期时间字符串 |
| `type: string` + `format: uri` | 生成示例 URI |
| `type: string` + `enum` | 随机选取一个允许的值 |
| `type: integer` / `number` | 随机数值 |
| `type: boolean` | 随机 `true` / `false` |
| `type: object` + `required` | 必填字段始终存在；可选字段可能为 `null`（保留在响应中以保持结构稳定） |
| `type: array` + `minItems` / `maxItems` | 遵守边界，默认 1–5 个元素 |
| `$ref`（schema + response） | 解析 `#/components/schemas/...` 和 `#/components/responses/...` |
| 递归 `$ref`（循环） | 限制递归深度，防止栈溢出 |

### 路径参数注入

当响应 Schema 中存在与路径参数同名的字段时（例如 `/pets/{petId}` 中的 `petId`），请求值会被注入到该字段中，并强制转换为 Schema 类型。这使 Mock 响应与请求上下文保持一致。

### HTTP 方法覆盖

所有 OpenAPI HTTP 方法均已支持：`GET`、`POST`、`PUT`、`DELETE`、`PATCH`、`OPTIONS`、`HEAD`、`TRACE`。

### 正确的状态码

| 场景 | 状态码 |
|-----------|--------|
| 成功 | `200` |
| 路径不在 spec 中 | `404` |
| 路径上未定义该方法 | `405` |
| 未定义 `200` 响应 | `501` |


## CLI 参考

```text
Usage: mock-cli [OPTIONS] <SPEC>

Arguments:
  <SPEC>  OpenAPI spec 文件路径（YAML 或 JSON）

Options:
  -p, --port <PORT>   监听端口 [默认: 3333]
  -H, --host <HOST>   绑定的主机/IP 地址 [默认: 127.0.0.1]
  -h, --help          打印帮助信息
  -V, --version       打印版本号
```

绑定到所有网络接口（在容器中很有用）：

```bash
mock-cli --host 0.0.0.0 --port 8080 spec.yaml
```

Spec 格式（JSON 还是 YAML）会自动检测：先看文件扩展名，再嗅探内容。像 `spec.txt` 这样的文件也能用。


## 路线图

| 功能 | 状态 |
|---------|--------|
| 基础 Mock 与动态数据 | 已完成 |
| 路径参数注入 | 已完成 |
| Schema `format` / `enum` / `required` / 数组边界 | 已完成 |
| `$ref` 解析与循环安全 | 已完成 |
| 正确的 HTTP 状态码 | 已完成 |
| 可配置绑定主机 | 已完成 |
| 优雅关闭（Ctrl+C / SIGTERM） | 已完成 |
| Spec 变更热重载 | 计划中 |
| 请求校验（按 spec） | 计划中 |
| 通过配置自定义 faker 规则 | 计划中 |
| 多 spec 组合 | 计划中 |


## 项目结构

```text
crates/
├── cli/      # 二进制入口与参数解析
├── core/     # Spec 解析与数据生成（无 IO）
└── server/   # HTTP 路由与请求处理
```

### 开发

```bash
git clone https://github.com/wanghao12345/mock-cli.git
cd mock-cli
cargo run -- examples/swagger.yaml
```


## License

基于 MIT License 授权。
