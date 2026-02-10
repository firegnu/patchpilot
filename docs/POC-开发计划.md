# PatchPilot PoC 开发计划（可演进）

## 1. 目标

本 PoC 目标是做出一个“可演示闭环”的 macOS menubar 更新管理工具：
- 自动检查更新（定时触发 `check_all`）。
- 手动确认后执行更新（不做自动更新）。
- 失败与超时信息可见、可追踪。
- 重启后保留历史记录。

## 2. PoC 完成标准

满足以下条件即视为 PoC 完成：
1. 启动后可以正常加载配置并展示软件列表。
2. 定时或手动触发 `check_all`，且不会重叠执行。
3. 用户可手动执行单项更新，并在执行前确认。
4. 命令超时可被识别（`timed_out`）并展示错误信息。
5. 历史记录面板可查看最近检查/更新/共享命令执行结果。
6. 构建通过：`npm run build`、`cargo check`。

## 3. 范围定义

### In Scope（PoC 内）
- 自动检查 + 手动更新确认。
- 超时控制、真实错误透传、执行历史。
- `check_all` 运行状态可视化（运行中/最近一次完成状态）。
- PoC 验收脚本与演示配置。

### Out of Scope（PoC 后）
- 自动更新执行。
- 完整权限系统或命令沙箱。
- 完整可视化配置编辑器（替代 JSON 编辑器）。

## 4. 当前基线

已具备能力：
1. `command_timeout_seconds` 配置字段与命令超时处理。
2. 执行历史持久化（`execution-history.json`）与前端历史展示。
3. `check_all` 前端防重入与运行状态显示。
4. README 已同步当前架构和运行机制。

## 5. 实施计划（执行顺序）

### 阶段 A：文档与验收基线
1. 创建本文件 `docs/POC-开发计划.md`。
2. 新增 `docs/POC-验收脚本.md`，固化演示步骤和验收项。
3. 新增 `docs/poc-software-items.sample.json`，提供可演示配置模板。

### 阶段 B：稳定性收尾
1. 后端 `check_all` 增加防重入兜底（避免多入口并发触发）。
2. 统一 `summary` 文案，减少历史噪音并提升可读性。
3. 归一化错误文案（包含命令类型、exit code、stderr）。

### 阶段 C：体验收尾
1. 监控面板补充最近一次 `check_all` 状态标识（Success/Failed/Running）。
2. 历史面板增加最小筛选能力（All / Check / Update / Shared）。

### 阶段 D：联调与验收
1. 执行完整链路：启动 -> 自动检查 -> 手动更新 -> 查看历史 -> 重启验证。
2. 执行构建验证：`npm run build`、`cargo check`。
3. 在验收文档记录结果与残留问题。

## 6. 接口与数据约束（PoC 冻结）

### 后端命令
- `load_config`
- `save_config`
- `check_item`
- `check_all`
- `run_item_update`
- `run_ad_hoc_command`
- `load_history`

### 关键类型
- `AppConfig.command_timeout_seconds: u64`
- `CommandOutput.timed_out: bool`
- `ExecutionHistoryEntry`（历史记录主结构）

### 默认值
- `check_interval_minutes = 480`
- `command_timeout_seconds = 120`
- history 保留条数上限 = 200

## 7. 测试与验收场景

### 功能场景
1. `check_item` 成功/失败路径可见。
2. `check_all` 运行中状态可见，且无重叠执行。
3. `run_item_update` 执行前确认，执行后状态刷新。
4. `run_ad_hoc_command` 执行结果进入历史。

### 异常场景
1. 命令不存在（exit 非 0）错误可读。
2. 命令超时（`timed_out = true`）可识别。
3. history 读取失败时不影响主流程。

### 构建场景
1. 前端构建：`client` 下 `npm run build` 通过。
2. 后端检查：`src-tauri` 下 `cargo check` 通过。

## 8. 风险与应对

1. 外部命令耗时不稳定：
- 应对：超时控制 + Running 状态 + 历史记录。

2. 历史噪音过多：
- 应对：统一 summary、前端筛选、保留上限 200 条。

3. 配置兼容问题（旧配置无 timeout 字段）：
- 应对：前端 normalize + 后端 `serde default` 双保险。

## 9. 交付物清单

PoC 收口时应存在以下文档与结果：
1. `docs/POC-开发计划.md`
2. `docs/POC-验收脚本.md`
3. `docs/poc-software-items.sample.json`
4. 构建验证通过记录（写入验收脚本文档）
