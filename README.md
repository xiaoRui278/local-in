# Local-In

局域网P2P聊天工具，基于Tauri + rust-libp2p开发。

## 功能特性

- 局域网自动发现设备
- P2P点对点聊天
- 无中心服务器
- 端到端加密
- 跨平台支持

## 技术栈

- **前端**: React + TypeScript + Vite
- **后端**: Rust + Tauri
- **P2P**: rust-libp2p
- **发现**: mDNS
- **加密**: Noise协议

## 开发环境

- Rust 1.70+
- Node.js 18+
- pnpm (推荐)

## 快速开始

```bash
# 安装依赖
pnpm install

# 开发模式
pnpm tauri dev

# 构建
pnpm tauri build
```

## 项目结构

```
local-in/
├── src/                # React前端
│   ├── App.tsx
│   ├── main.tsx
│   └── styles.css
├── src-tauri/          # Rust后端
│   ├── src/
│   │   ├── main.rs
│   │   └── p2p/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── vite.config.ts
```

## 打包产物

| 平台 | 产物 |
|------|------|
| macOS | .dmg |
| Windows | .msi |
| Linux | .deb |

## 许可证

MIT License
