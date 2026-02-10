# PatchPilot PoC 验收脚本

## 1. 目标

用同一套步骤验证 PoC 是否达到“可演示闭环”：
- 自动检查可用且无重叠执行
- 手动更新可用且有确认
- 超时/失败可见
- 历史可追踪且可持久化

## 2. 前置准备

1. 使用样例配置：`docs/poc-software-items.sample.json`
2. 安装依赖：
```bash
cd client
npm install
```
3. 启动开发：
```bash
cd src-tauri
cargo tauri dev
```

## 3. 验收步骤

### A. 启动与配置加载
1. 启动应用后可看到主界面与软件列表。
2. 顶部显示 Interval 与 Timeout。
3. History 面板可正常显示（首次允许为空）。

### B. Check All 与防重入
1. 点击 `Check All`，状态变为 `Checking...`。
2. 运行期间按钮禁用，状态文案显示 `Check-all is running...`。
3. 结束后状态文案显示最近一次 `check-all` 的时间与摘要。
4. 并发触发时，后端应拒绝重入并记录 `check-all-skip` 历史。

### C. 单项检查与更新
1. 点击某项 `Check`，该项状态变为 `Checking...` 后恢复。
2. 若有更新，`Update` 按钮可用。
3. 点击 `Update` 必须先弹确认框；取消后不执行命令。
4. 确认执行后可看到 exit code，并触发该项重新检查。

### D. 错误与超时
1. 将某项命令改为无效命令，执行检查，错误信息可读。
2. 使用超时命令验证 `timed_out` 路径（例如长时间 sleep）。
3. 失败和超时都应出现在历史记录。

### E. 持久化
1. 执行若干 check/update/shared command 后退出应用。
2. 重新启动应用，History 仍能看到最近记录。

## 4. 构建验证

1. 前端构建：
```bash
cd client
npm run build
```
2. 后端检查：
```bash
cd src-tauri
cargo check
```

## 5. 验收结果记录（执行时填写）

- 日期：
- 执行人：
- A 启动与配置加载：`通过 / 不通过`
- B Check All 与防重入：`通过 / 不通过`
- C 单项检查与更新：`通过 / 不通过`
- D 错误与超时：`通过 / 不通过`
- E 持久化：`通过 / 不通过`
- 构建验证：`通过 / 不通过`
- 备注与遗留问题：

最近一次自动化执行记录见：
- `docs/POC-验收记录.md`
