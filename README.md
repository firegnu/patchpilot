# PatchPilot

PatchPilot 是一个 macOS menubar 桌面应用（Tauri + React），用于集中管理本机软件更新检查与手动执行更新命令。

设计目标：
- 把零散的软件更新命令收敛到一个入口。
- 通过配置驱动检查逻辑，减少手工维护成本。
- 保持“手动确认后执行”的安全策略，避免误操作。

## 功能概览

- 托盘应用常驻，主窗口关闭时隐藏到 menubar。
- menubar 菜单支持高频操作（分区检查、快捷更新、主题/频率切换）。
- 按配置检查软件是否有更新（支持“版本对比”或“命令输出匹配”两种模式）。
- 对单个软件执行更新命令（执行前确认）。
- 支持共享维护命令（例如 `brew update`、`brew upgrade`）。
- 主界面分区：
  - `Homebrew/Bun`：手动检查 + 手动更新
  - `CLI 工具`：自动检查 + 手动更新
  - `App`：自动检查 + 手动单项检查（不提供更新按钮）
- 启动时自动检测软件安装状态，未安装的项目自动隐藏。
- 支持主题切换（浅色 / 深色 / 跟随系统）。

## 技术栈

- 桌面壳：Tauri 2（Rust）
- 前端：React 18 + TypeScript + Vite
- 后端依赖：`tauri`, `serde`, `serde_json`, `regex`, `chrono`

## 项目结构（当前实现）

```text
PatchPilot/
├── client/                         # React UI
│   ├── src/
│   │   ├── App.tsx                # 页面编排、状态管理、定时检查
│   │   ├── components/
│   │   │   ├── MonitorPanel.tsx   # 软件监控表格（check/update）
│   │   │   ├── SharedCommandsPanel.tsx
│   │   │   └── HistoryPanel.tsx   # 历史数据（用于状态回显）
│   │   ├── lib/ipc.ts             # Tauri invoke 封装
│   │   └── types/app.ts           # 前端类型定义
│   └── package.json
└── src-tauri/                      # Rust 后端 + Tray + 命令执行
    ├── src/
    │   ├── main.rs                # Tauri 启动、Tray、窗口行为、命令注册
    │   ├── commands.rs            # 命令入口与流程编排
    │   ├── model.rs               # 数据模型与默认配置
    │   └── services/
    │       ├── check_all_guard.rs # check_all 防重入并发锁
    │       ├── check_service.rs   # 检查逻辑（版本对比/命令匹配）
    │       ├── config_store.rs    # 配置文件读写与路径解析
    │       ├── detect_service.rs  # 启动时并行检测软件安装状态
    │       ├── history_events.rs  # 历史事件构造与安全写入
    │       ├── history_store.rs   # 本地执行历史存储
    │       ├── result_store.rs    # 最近检查结果持久化
    │       └── shell_runner.rs    # 统一 shell 执行器（zsh -lc）
    ├── tauri.conf.json
    └── Cargo.toml
```

## 运行机制

### 1) 前后端交互

前端通过 `@tauri-apps/api/core` 的 `invoke` 调用 Rust 命令：
- `load_config`
- `save_config`
- `load_latest_results`
- `detect_installed_items`
- `check_item`
- `check_all`
- `check_auto_items`
- `check_auto_cli_items`
- `check_auto_app_items`
- `check_runtime_items`
- `run_item_update`
- `run_ad_hoc_command`
- `get_active_node_version`
- `load_history`

对应封装位于 `client/src/lib/ipc.ts`。

### 2) 检查逻辑

每个软件项（`SoftwareItem`）有两种检查路径：

- 版本对比模式（优先）：
  - 提供 `current_version_command` + `latest_version_command`
  - 取两者输出字符串并比较是否一致

- 输出匹配模式：
  - 执行 `update_check_command`
  - 若配置了 `update_check_regex`，使用正则判断输出
  - 若未配置 regex，则按布尔语义解析输出（`1/true/yes`）

`check_all` 只会处理 `enabled = true` 的项目。

### 3) 更新逻辑

- 点击 Update 时，前端先弹窗确认。
- 确认后调用 `run_item_update` 执行 `update_command`。
- 执行结束后，前端会触发该项重新检查。

### 4) 命令执行器

所有命令由 Rust 侧统一通过 `zsh -lc "<command>"` 执行，并返回：
- `exit_code`
- `stdout`
- `stderr`
- `duration_ms`
- `timed_out`

命令执行会使用配置中的 `command_timeout_seconds` 超时值，超时后会中止子进程并标记 `timed_out = true`。

### 5) 执行历史

- 后端将关键动作写入本地 `execution-history.json`（最多保留 200 条）。
- 前端通过 `load_history` 拉取并展示最近记录。
- 历史写入失败不会中断主流程（仅记录后端日志）。

## 配置文件

### 配置结构

```json
{
  "check_interval_minutes": 480,
  "command_timeout_seconds": 120,
  "theme_mode": "system",
  "auto_check_enabled": true,
  "shared_update_commands": ["brew update", "brew upgrade"],
  "items": [
    {
      "id": "brew",
      "name": "Homebrew",
      "kind": "cli",
      "enabled": true,
      "description": "Check and update Homebrew packages",
      "current_version_command": "brew --version | head -n 1 | awk '{print $2}'",
      "latest_version_command": null,
      "update_check_command": "brew outdated --quiet",
      "update_check_regex": ".+",
      "update_command": "brew update && brew upgrade"
    }
  ]
}
```

### 配置文件路径解析顺序

后端按以下顺序寻找 `software-items.json`：
1. 当前工作目录：`./software-items.json`
2. 当前工作目录：`./config/software-items.json`
3. 当前目录的父目录：`../config/software-items.json`
4. Tauri 应用配置目录：`app_config_dir/software-items.json`

如果不存在，会自动写入默认配置。

## 本地开发

### 环境要求

- Node.js 18+
- Rust stable（建议通过 `rustup`）
- Cargo
- Tauri CLI（可通过 `cargo` 方式运行，无需全局安装 npm 包）
- macOS 开发工具链（Xcode Command Line Tools）

### 安装依赖

```bash
cd client
npm install
```

### 启动开发模式

```bash
cd src-tauri
cargo tauri dev
```

说明：
- `tauri.conf.json` 已配置 `beforeDevCommand`，会自动在 `client` 启动 Vite。
- 前端开发地址为 `http://localhost:5173`。

## 构建

```bash
cd src-tauri
cargo tauri build
```

`beforeBuildCommand` 会先执行前端构建（`npm run build`）。

## 安全说明

- 每次执行更新命令都会在 UI 中手动确认。
- 所有 shell 命令都在本机执行，请仅保存你信任的命令。
- 建议优先使用只读检查命令，将写操作集中在 `update_command`。

## 适合贡献的方向

- 更细粒度的配置校验与错误提示。
- 命令执行超时/取消控制。
- 更新与检查历史记录。
- 通知中心集成与启动项管理。
