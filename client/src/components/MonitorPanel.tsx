import type { CheckResult, ExecutionHistoryEntry, SoftwareItem } from '../types/app';

interface MonitorPanelProps {
  items: SoftwareItem[];
  resultMap: Record<string, CheckResult>;
  checkingMap: Record<string, boolean>;
  updatingMap: Record<string, boolean>;
  checkAllRunning: boolean;
  latestCheckAllEntry: ExecutionHistoryEntry | null;
  onCheckItem: (itemId: string) => Promise<void>;
  onCheckAll: () => Promise<void>;
  onRunUpdate: (item: SoftwareItem) => Promise<void>;
}

type CheckAllState = 'running' | 'success' | 'failed' | 'skipped' | 'idle';

const statusText = (item: SoftwareItem, result?: CheckResult): string => {
  if (!item.enabled) {
    return '已禁用';
  }
  if (!result) {
    return '未检查';
  }
  if (result.error) {
    return `错误：${result.error}`;
  }
  return result.has_update ? '有可用更新' : '已是最新';
};

const resolveCheckAllState = (
  checkAllRunning: boolean,
  latestCheckAllEntry: ExecutionHistoryEntry | null
): CheckAllState => {
  if (checkAllRunning) {
    return 'running';
  }
  if (!latestCheckAllEntry) {
    return 'idle';
  }
  if (latestCheckAllEntry.action === 'check-all-skip') {
    return 'skipped';
  }
  return latestCheckAllEntry.success ? 'success' : 'failed';
};

export default function MonitorPanel({
  items,
  resultMap,
  checkingMap,
  updatingMap,
  checkAllRunning,
  latestCheckAllEntry,
  onCheckItem,
  onCheckAll,
  onRunUpdate,
}: MonitorPanelProps) {
  const checkAllState = resolveCheckAllState(checkAllRunning, latestCheckAllEntry);
  const badgeText = {
    running: '运行中',
    success: '成功',
    failed: '失败',
    skipped: '已跳过',
    idle: '空闲',
  }[checkAllState];
  const checkAllStatus = checkAllRunning
    ? '全量检查正在运行...'
    : latestCheckAllEntry
      ? `最近一次全量检查：${new Date(latestCheckAllEntry.recorded_at).toLocaleString('zh-CN')} | ${latestCheckAllEntry.summary}`
      : '全量检查尚未执行。';

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>软件监控</h2>
        <div className="inline-actions">
          <span className={`status-badge status-${checkAllState}`}>{badgeText}</span>
          <button
            type="button"
            className="btn"
            disabled={checkAllRunning}
            onClick={() => void onCheckAll()}
          >
            {checkAllRunning ? '检查中...' : '全量检查'}
          </button>
        </div>
      </div>
      <p className="muted">{checkAllStatus}</p>
      <table>
        <thead>
          <tr>
            <th>名称</th>
            <th>当前版本</th>
            <th>最新版本</th>
            <th>状态</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          {items.map((item) => {
            const result = resultMap[item.id];
            const checking = Boolean(checkingMap[item.id]);
            const updating = Boolean(updatingMap[item.id]);
            const hasUpdate = result?.has_update && !result?.error;
            return (
              <tr key={item.id}>
                <td>
                  <div>{item.name}</div>
                  <small>{item.description}</small>
                </td>
                <td>{result?.current_version ?? '-'}</td>
                <td>{result?.latest_version ?? '-'}</td>
                <td>{checking ? '检查中...' : statusText(item, result)}</td>
                <td>
                  <div className="inline-actions">
                    <button
                      type="button"
                      className="btn"
                      disabled={!item.enabled || checking}
                      onClick={() => void onCheckItem(item.id)}
                    >
                      检查
                    </button>
                    <button
                      type="button"
                      className="btn btn-primary"
                      title={hasUpdate ? '检测到更新，执行更新命令' : '请先执行检查，确认有更新后再执行'}
                      disabled={!item.enabled || updating || !hasUpdate}
                      onClick={() => void onRunUpdate(item)}
                    >
                      {updating ? '执行中...' : '更新'}
                    </button>
                  </div>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </section>
  );
}
