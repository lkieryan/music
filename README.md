 src-tauri/
├── src/
│   ├── audio/              # 🎵 音频引擎
│   │   ├── mod.rs         
│   │   ├── player.rs       # 统一播放器接口
│   │   ├── decoder.rs      # 音频解码器 (支持 MP3, FLAC, OGG 等)
│   │   └── effects.rs      # 音效处理 (均衡器、混响)
│   │
│   ├── sources/           # 🎧 多音源支持
│   │   ├── mod.rs         # 音源管理器 + 统一搜索
│   │   ├── traits.rs      # MusicSource 通用接口
│   │   ├── local.rs       # 本地音乐扫描
│   │   ├── kugou.rs       # 酷狗音乐 API
│   │   ├── netease.rs     # 网易云音乐 API  
│   │   ├── qq_music.rs    # QQ音乐 API
│   │   └── bilibili.rs    # B站音频 API
│   │
│   ├── database/          # 💾 数据存储
│   │   ├── mod.rs
│   │   ├── models.rs      # 数据库模型定义
│   │   ├── migration.rs   # 数据库迁移
│   │   ├── local_library.rs # 本地音乐库操作
│   │   └── cache.rs       # 缓存管理
│   │
│   ├── commands/          # 🔌 前端接口
│   │   ├── mod.rs
│   │   └── player.rs      # 播放器控制命令
│   │
│   ├── state/             # 🔄 状态管理
│   │   ├── mod.rs
│   │   ├── app_state.rs   # 全局应用状态
│   │   ├── player_state.rs # 播放器状态
│   │   └── library_state.rs # 音乐库状态
│   │
│   ├── utils/             # 🛠️ 工具模块
│   │   ├── mod.rs
│   │   └── metadata.rs    # 元数据提取
│   │
│   └── main.rs            # 🚀 程序入口
│
├── Cargo.toml             # 📦 依赖管理
├── build.rs               # 🔨 构建脚本
└── tauri.conf.json        # ⚙️ Tauri 配置



计划

音频引擎实现 - 使用 rodio + symphonia 实现真正的播放功能
API 对接 - 实现各音乐平台的具体 API 调用
前端开发 - 使用 Next.js 构建现代化 UI
歌词显示 - LRC 歌词解析和同步显示
下载管理 - 支持高质量音乐下载