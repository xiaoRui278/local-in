# Local-In

<p align="center">
  <img src="https://img.shields.io/badge/React-18-blue?logo=react" alt="React">
  <img src="https://img.shields.io/badge/TypeScript-5.5-blue?logo=typescript" alt="TypeScript">
  <img src="https://img.shields.io/badge/Tauri-2.0-orange?logo=tauri" alt="Tauri">
  <img src="https://img.shields.io/badge/Rust-2021-brown?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/License-MIT-green" alt="License">
  <img src="https://img.shields.io/badge/Vibe%20Coding-🌊-ff69b4" alt="Vibe Coding">
</p>

<p align="center">
  <em>🌊 本项目采用 Vibe Coding 方式开发 — 人与 AI 结对协作完成。</em>
</p>

<p align="center">
  <strong>一个局域网 P2P 聊天与文件传输桌面应用，无需中心服务器即可在同一局域网内发现设备、发送消息和传输文件。</strong>
</p>

---

## ✨ 功能特点

- 🔍 **局域网自动发现** — 基于 mDNS 自动发现同网段在线设备
- 💬 **P2P 即时聊天** — 支持点对点消息、全局消息和本地消息持久化
- 👥 **群组会话** — 支持创建群组、加入群组、成员展示和群消息记录
- ⚡ **高速文件传输** — 基于 `libp2p-stream` 的二进制流传输，避免 Base64 膨胀
- 📈 **传输进度** — 文件接收显示进度、速度、失败原因，支持取消和重试
- 🔁 **断点续传** — 接收端按本地临时文件和数据库进度恢复传输
- 🔐 **加密传输** — libp2p Noise 握手加密，节点身份持久化
- 🖥️ **跨平台桌面端** — 基于 Tauri，可打包 macOS、Windows、Linux

## 🚀 快速开始

### 系统要求

| 要求 | 版本 |
|------|------|
| Node.js | 18 或更高 |
| Rust | 1.70 或更高 |
| pnpm | 推荐使用最新稳定版 |

### 从源码运行

```bash
git clone git@github.com:xiaoRui278/local-in.git
cd local-in
pnpm install
pnpm run tauri dev
```

### 构建应用

```bash
pnpm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 📖 使用方法

1. 启动应用后输入昵称进入局域网节点
2. 左侧设备列表会自动显示同一局域网内发现的在线用户
3. 选择用户后可发送点对点消息
4. 使用全局频道可向在线节点广播消息
5. 创建或加入群组后可进行群组会话
6. 点击文件按钮选择本地文件，向目标用户发送文件邀请
7. 接收方点击接受后开始二进制流传输，可查看进度、速度并在失败后重试

## ⚙️ 开发命令

| 命令 | 说明 |
|------|------|
| `pnpm install` | 安装前端和 Tauri CLI 依赖 |
| `pnpm run dev` | 启动 Vite 前端开发服务器 |
| `pnpm run tauri dev` | 启动 Tauri 桌面应用开发模式 |
| `pnpm run build` | TypeScript 检查并构建前端 |
| `pnpm run tauri build` | 构建桌面应用安装包 |
| `pnpm run preview` | 预览前端构建结果 |
| `cd src-tauri && cargo check` | 检查 Rust 后端 |
| `cd src-tauri && cargo test` | 运行 Rust 测试 |
| `cd src-tauri && cargo test <test_name>` | 运行单个 Rust 测试 |

> 当前 `package.json` 未配置前端测试或 lint 脚本。

## 🏗️ 技术架构

```
local-in/
├── src/                              # React 前端
│   ├── App.tsx                       # 应用布局、登录状态、主题和模态框组合
│   ├── hooks/useChat.ts              # Tauri IPC、频道事件、消息/文件/群组状态
│   ├── components/chat/              # 聊天区、输入区、消息气泡、文件卡片
│   ├── components/modals/            # 群组和设置相关弹窗
│   ├── types.ts                      # 前后端共享数据结构类型
│   └── styles.css                    # 全局样式
├── src-tauri/                        # Rust + Tauri 后端
│   ├── src/main.rs                   # Tauri 命令、应用状态、数据库和 P2P 事件桥接
│   ├── src/p2p/mod.rs                # libp2p 节点、mDNS、请求响应、stream 控制
│   ├── src/p2p/file_transfer.rs      # 二进制文件传输帧协议、校验、断点续传
│   ├── src/db/mod.rs                 # SQLite 持久化封装
│   ├── src/file/mod.rs               # 文件路径辅助函数
│   ├── Cargo.toml                    # Rust 依赖配置
│   └── tauri.conf.json               # Tauri 应用配置
├── package.json                      # 前端脚本和依赖
└── vite.config.ts                    # Vite 配置
```

### 技术栈

| 技术 | 用途 |
|------|------|
| React / TypeScript | 桌面应用前端 UI |
| Vite | 前端开发服务器和构建工具 |
| Tauri 2 | 桌面应用壳、系统能力和前后端 IPC |
| Rust / Tokio | 后端异步运行时和核心业务逻辑 |
| libp2p | 局域网 P2P 网络、节点发现和传输 |
| libp2p-stream | 高性能二进制文件流传输 |
| mDNS | 局域网节点自动发现 |
| Noise / Yamux | 加密握手和多路复用 |
| rusqlite | 本地 SQLite 数据持久化 |
| SHA-256 | 文件完整性校验 |

### 工作原理

1. **节点启动** — 后端读取或生成持久化 libp2p 身份，启动 P2P 节点并返回稳定 Peer ID
2. **设备发现** — mDNS 在局域网内发现其他 Local-In 节点，前端定时刷新在线列表
3. **消息传输** — 聊天消息通过 libp2p request-response 发送，SQLite 记录历史消息
4. **文件邀请** — 发送端先计算文件 SHA-256，再向接收端发送文件元数据
5. **二进制传输** — 接收端接受后建立 `/local-in-file/1` stream，按 frame 传输原始字节
6. **进度同步** — 后端通过 Tauri `Channel` 推送进度、完成、失败、取消事件到前端
7. **断点续传** — 重试时取数据库 `received_bytes` 和临时文件长度的较小值作为恢复偏移

## 📦 打包产物

| 平台 | 产物 |
|------|------|
| macOS | `.dmg` / `.app` |
| Windows | `.msi` / `.exe` |
| Linux | `.deb` / `.rpm` / `.AppImage` |

实际产物取决于当前平台和 Tauri bundler 配置。

## 📝 更新日志

### v1.0.0

- ✅ 支持局域网设备发现
- ✅ 支持 P2P 点对点聊天和全局消息
- ✅ 支持群组会话和成员展示
- ✅ 支持本地 SQLite 消息持久化
- ✅ 支持二进制流文件传输、进度显示、取消、重试和断点续传
- ✅ 支持稳定节点身份，重启后保持同一 Peer ID

## 📄 许可证

MIT License © [xiaoRui278](https://github.com/xiaoRui278)

---

<p align="center">
  <sub>Made with AI (Vibe Coding)</sub>
</p>
