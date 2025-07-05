# Design Document

## Overview

本设计文档描述了一个基于 Rust + Leptos + Tauri 技术栈的现代桌面应用程序的架构设计。该应用将提供类似 React 风格的组件化开发体验，集成第三方 API 服务，支持国际化，并具备完整的样式和资源管理系统。

### 技术栈选择

- **前端框架**: Leptos - 提供响应式 UI 和类似 React 的开发体验
- **桌面应用框架**: Tauri - 轻量级桌面应用包装器
- **HTTP 客户端**: reqwest - 用于第三方 API 集成
- **国际化**: leptos_i18n - Leptos 专用的多语言支持
- **构建工具**: Cargo + Tauri CLI - 项目管理和构建

## Architecture

### 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Desktop Application                       │
├─────────────────────────────────────────────────────────────┤
│                    Tauri Window                             │
├─────────────────────────────────────────────────────────────┤
│                    Leptos Frontend                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Pages     │  │ Components  │  │    I18n     │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
├─────────────────────────────────────────────────────────────┤
│                    Tauri Commands                           │
├─────────────────────────────────────────────────────────────┤
│                    Tauri Backend                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ API Services│  │   Commands  │  │   Storage   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
├─────────────────────────────────────────────────────────────┤
│                    Shared Crates                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Models    │  │    Utils    │  │ API Client  │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
├─────────────────────────────────────────────────────────────┤
│                  External APIs                              │
└─────────────────────────────────────────────────────────────┘
```

### Crates 架构设计

#### 1. shared-models crate
- 定义前后端共享的数据结构
- API 请求/响应模型
- 配置和设置模型
- 序列化/反序列化支持

#### 2. shared-utils crate  
- 通用工具函数
- 错误处理类型
- 数据验证逻辑
- 常量定义

#### 3. api-client crate
- HTTP 客户端封装
- API 端点定义
- 请求/响应处理
- 认证和授权逻辑

### 项目结构设计

```
rust-leptos-tauri-app/
├── crates/                         # 共享 crates
│   ├── shared-models/              # 共享数据模型
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── api.rs              # API 数据模型
│   │       └── config.rs           # 配置模型
│   ├── shared-utils/               # 共享工具函数
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs            # 错误处理
│   │       └── validation.rs       # 数据验证
│   └── api-client/                 # API 客户端库
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── client.rs           # HTTP 客户端
│           └── endpoints.rs        # API 端点定义
├── src-tauri/                      # Tauri 后端
│   ├── Cargo.toml                  # 后端依赖配置
│   ├── tauri.conf.json             # Tauri 应用配置
│   ├── build.rs                    # 构建脚本
│   ├── src/
│   │   ├── main.rs                 # 应用入口点
│   │   ├── commands/               # Tauri 命令模块
│   │   │   ├── mod.rs
│   │   │   ├── api.rs              # API 相关命令
│   │   │   └── system.rs           # 系统相关命令
│   │   ├── services/               # 业务服务层
│   │   │   ├── mod.rs
│   │   │   ├── external_api.rs     # 第三方 API 集成
│   │   │   └── storage.rs          # 本地存储服务
│   │   └── utils/                  # 后端特定工具
│   │       └── mod.rs
│   └── icons/                      # 应用图标资源
├── src/                            # Leptos 前端
│   ├── main.rs                     # 前端应用入口
│   ├── app.rs                      # 根应用组件
│   ├── components/                 # UI 组件库
│   │   ├── mod.rs
│   │   ├── common/                 # 通用组件
│   │   │   ├── mod.rs
│   │   │   ├── button.rs           # 按钮组件
│   │   │   ├── input.rs            # 输入框组件
│   │   │   └── modal.rs            # 模态框组件
│   │   └── layout/                 # 布局组件
│   │       ├── mod.rs
│   │       ├── header.rs           # 头部组件
│   │       └── sidebar.rs          # 侧边栏组件
│   ├── pages/                      # 页面组件
│   │   ├── mod.rs
│   │   ├── home.rs                 # 主页
│   │   └── settings.rs             # 设置页面
│   ├── hooks/                      # 自定义 Hooks
│   │   ├── mod.rs
│   │   ├── use_api.rs              # API 调用 Hook
│   │   └── use_i18n.rs             # 国际化 Hook
│   ├── services/                   # 前端服务层
│   │   ├── mod.rs
│   │   └── tauri_api.rs            # Tauri 命令调用
│   ├── styles/                     # 样式文件
│   │   ├── global.css              # 全局样式
│   │   ├── variables.css           # CSS 变量
│   │   └── components.css          # 组件样式
│   └── utils/                      # 前端工具函数
│       ├── mod.rs
│       └── constants.rs            # 常量定义
├── locales/                        # 国际化资源
│   ├── en.json                     # 英文翻译
│   ├── zh.json                     # 中文翻译
│   └── ja.json                     # 日文翻译
├── assets/                         # 静态资源
│   ├── images/                     # 图片资源
│   ├── fonts/                      # 字体文件
│   └── icons/                      # UI 图标
├── public/                         # 公共文件（可选）
├── Cargo.toml                      # 前端依赖配置
├── Cargo.lock                      # 依赖锁定文件
├── index.html                      # HTML 模板
├── README.md                       # 项目文档
└── .gitignore                      # Git 忽略文件
```

## Components and Interfaces

### 前端组件架构

#### 1. 应用根组件 (App)
```rust
// src/app.rs
#[component]
pub fn App() -> impl IntoView {
    provide_context(create_rw_signal(AppState::default()));
    
    view! {
        <Router>
            <Routes>
                <Route path="/" view=HomePage/>
                <Route path="/settings" view=SettingsPage/>
            </Routes>
        </Router>
    }
}
```

#### 2. 通用组件设计
```rust
// src/components/common/button.rs
#[component]
pub fn Button(
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] size: ButtonSize,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    children: Children,
) -> impl IntoView {
    // 组件实现
}
```

#### 3. 页面组件结构
```rust
// src/pages/home.rs
#[component]
pub fn HomePage() -> impl IntoView {
    let (data, set_data) = create_signal(None::<ApiData>);
    let api_service = use_api_service();
    
    // 页面逻辑和 UI
}
```

### 后端服务架构

#### 1. Tauri 命令接口
```rust
// src-tauri/src/commands/api.rs
#[tauri::command]
pub async fn fetch_external_data(
    endpoint: String,
    params: Option<serde_json::Value>
) -> Result<ApiResponse, String> {
    // API 调用实现
}

#[tauri::command]
pub async fn save_user_settings(
    settings: UserSettings
) -> Result<(), String> {
    // 设置保存实现
}
```

#### 2. API 服务层
```rust
// src-tauri/src/services/external_api.rs
pub struct ExternalApiService {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl ExternalApiService {
    pub async fn fetch_data(&self, endpoint: &str) -> Result<ApiData, ApiError> {
        // HTTP 请求实现
    }
}
```

#### 3. 数据模型
```rust
// src-tauri/src/models/api_models.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSettings {
    pub language: String,
    pub theme: String,
    pub api_endpoints: Vec<String>,
}
```

## Data Models

### 前端状态管理

#### 应用状态结构
```rust
#[derive(Debug, Clone)]
pub struct AppState {
    pub current_language: String,
    pub theme: Theme,
    pub user_settings: Option<UserSettings>,
    pub api_data: Option<ApiData>,
    pub loading_states: HashMap<String, bool>,
}
```

#### 响应式状态管理
- 使用 Leptos 的 `create_signal` 和 `create_rw_signal` 管理本地状态
- 使用 `provide_context` 和 `use_context` 进行状态共享
- 实现自定义 Hooks 封装复杂状态逻辑

### 后端数据流

#### API 数据流程
1. 前端组件触发数据请求
2. 调用 Tauri 命令
3. 后端服务处理 HTTP 请求
4. 数据序列化后返回前端
5. 前端更新响应式状态

## Error Handling

### 前端错误处理

#### 错误类型定义
```rust
#[derive(Debug, Clone)]
pub enum AppError {
    ApiError(String),
    ValidationError(String),
    NetworkError(String),
    UnknownError(String),
}
```

#### 错误处理策略
- 使用 `Result<T, E>` 类型进行错误传播
- 实现全局错误边界组件
- 提供用户友好的错误消息
- 支持错误重试机制

### 后端错误处理

#### 统一错误响应
```rust
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u32,
    pub details: Option<serde_json::Value>,
}
```

#### 错误处理中间件
- HTTP 请求错误处理
- 序列化/反序列化错误处理
- 业务逻辑错误处理
- 日志记录和错误追踪

## Testing Strategy

### 前端测试

#### 单元测试
- 组件渲染测试
- 状态管理测试
- 工具函数测试

#### 集成测试
- 页面交互测试
- API 调用测试
- 路由导航测试

### 后端测试

#### 单元测试
- Tauri 命令测试
- 服务层逻辑测试
- 数据模型测试

#### 集成测试
- API 集成测试
- 数据持久化测试
- 错误处理测试

### 测试工具和框架
- 前端: `wasm-bindgen-test`, `leptos-testing`
- 后端: `tokio-test`, `mockito`
- E2E 测试: Tauri 内置测试工具

## 国际化设计

### leptos_i18n 配置

#### Cargo.toml 配置
```toml
[dependencies]
leptos_i18n = "0.3"

[build-dependencies]
leptos_i18n = "0.3"
```

#### 翻译文件结构
```json
// locales/en.json
{
  "common": {
    "save": "Save",
    "cancel": "Cancel", 
    "loading": "Loading..."
  },
  "pages": {
    "home": {
      "title": "Welcome",
      "description": "This is the home page"
    }
  }
}

// locales/zh.json
{
  "common": {
    "save": "保存",
    "cancel": "取消",
    "loading": "加载中..."
  },
  "pages": {
    "home": {
      "title": "欢迎",
      "description": "这是主页"
    }
  }
}
```

### 国际化实现

#### 配置文件
```rust
// src/i18n.rs
use leptos_i18n::*;

leptos_i18n! {
    locales: "./locales",
    default: "en",
    sync_html_tag_lang: true,
    sync_html_tag_dir: true,
}
```

#### 组件中使用
```rust
// src/components/common/button.rs
use crate::i18n::*;

#[component]
pub fn SaveButton() -> impl IntoView {
    let i18n = use_i18n();
    
    view! {
        <button>{t!(i18n, common.save)}</button>
    }
}
```

#### 语言切换
```rust
// src/components/layout/language_selector.rs
#[component]
pub fn LanguageSelector() -> impl IntoView {
    let i18n = use_i18n();
    
    let change_language = move |lang: &str| {
        i18n.set_locale(lang.parse().unwrap());
    };
    
    view! {
        <select on:change=move |ev| {
            let value = event_target_value(&ev);
            change_language(&value);
        }>
            <option value="en">"English"</option>
            <option value="zh">"中文"</option>
        </select>
    }
}
```

## 样式和主题系统

### CSS 变量系统
```css
/* src/styles/variables.css */
:root {
  --primary-color: #007bff;
  --secondary-color: #6c757d;
  --background-color: #ffffff;
  --text-color: #333333;
  --border-radius: 4px;
  --spacing-unit: 8px;
}
```

### 组件样式策略
- 使用 CSS 模块或 styled-components 风格
- 支持主题切换
- 响应式设计
- 可访问性支持

## Workspace 配置

### 根目录 Cargo.toml
```toml
[workspace]
members = [
    "crates/shared-models",
    "crates/shared-utils", 
    "crates/api-client",
    "src-tauri"
]

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
```

### Crate 依赖关系
- `src-tauri` 依赖所有共享 crates
- `frontend` 通过 wasm-bindgen 使用共享模型
- 共享 crates 之间的依赖关系最小化

## 构建和部署

### 开发环境配置
```bash
# 安装依赖
cargo install tauri-cli
cargo install trunk

# 开发模式
tauri dev

# 构建生产版本
tauri build
```

### 构建优化
- 代码分割和懒加载
- 资源压缩和优化
- 死代码消除
- 缓存策略

### 部署策略
- 生成平台特定的安装包
- 自动更新机制
- 版本管理
- 分发渠道配置