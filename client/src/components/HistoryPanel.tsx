import { Fragment, useMemo, useState } from 'react';
import type { ExecutionHistoryEntry } from '../types/app';

interface HistoryPanelProps {
  entries: ExecutionHistoryEntry[];
  canLoadMore: boolean;
  onLoadMore: () => Promise<void>;
  onReload: () => Promise<void>;
}

const formatStatus = (entry: ExecutionHistoryEntry): string => {
  if (entry.timed_out) {
    return '超时';
  }
  if (!entry.success) {
    return '失败';
  }
  return '成功';
};

const formatAction = (action: string): string => {
  const map: Record<string, string> = {
    'check-item': '单项检查',
    'check-all': '全量检查',
    'check-all-skip': '全量检查（跳过）',
    'auto-check': '自动检查',
    'auto-check-skip': '自动检查（跳过）',
    'run-item-update': '执行更新',
    'run-shared-command': '执行共享命令',
  };
  return map[action] ?? action;
};

type HistoryFilter = 'all' | 'check' | 'update' | 'shared';

const matchFilter = (entry: ExecutionHistoryEntry, filter: HistoryFilter): boolean => {
  if (filter === 'all') {
    return true;
  }
  if (filter === 'check') {
    return (
      entry.action.startsWith('check-') ||
      entry.action === 'auto-check' ||
      entry.action === 'auto-check-skip'
    );
  }
  if (filter === 'update') {
    return entry.action === 'run-item-update';
  }
  return entry.action === 'run-shared-command';
};

const hasDetails = (entry: ExecutionHistoryEntry): boolean =>
  Boolean(
    entry.command ||
      (entry.stdout && entry.stdout.trim()) ||
      (entry.stderr && entry.stderr.trim())
  );

export default function HistoryPanel({
  entries,
  canLoadMore,
  onLoadMore,
  onReload,
}: HistoryPanelProps) {
  const [filter, setFilter] = useState<HistoryFilter>('all');
  const [selectedEntry, setSelectedEntry] = useState<ExecutionHistoryEntry | null>(null);
  const filteredEntries = useMemo(
    () => entries.filter((entry) => matchFilter(entry, filter)),
    [entries, filter]
  );

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>执行历史</h2>
        <div className="inline-actions">
          <button type="button" className="btn" onClick={() => setFilter('all')}>
            全部
          </button>
          <button type="button" className="btn" onClick={() => setFilter('check')}>
            检查
          </button>
          <button type="button" className="btn" onClick={() => setFilter('update')}>
            更新
          </button>
          <button type="button" className="btn" onClick={() => setFilter('shared')}>
            共享命令
          </button>
          <button type="button" className="btn" onClick={() => void onReload()}>
            刷新
          </button>
          <button type="button" className="btn" disabled={!canLoadMore} onClick={() => void onLoadMore()}>
            加载更多
          </button>
        </div>
      </div>
      {filteredEntries.length === 0 ? (
        <p>暂无历史记录。</p>
      ) : (
        <div className="history-scroll">
          <table className="history-table">
            <thead>
              <tr>
                <th>时间</th>
                <th>动作</th>
                <th>目标</th>
                <th>结果</th>
                <th>耗时</th>
                <th>摘要</th>
                <th>详情</th>
              </tr>
            </thead>
            <tbody>
              {filteredEntries.map((entry) => (
                <Fragment key={entry.id}>
                  <tr>
                    <td>{new Date(entry.recorded_at).toLocaleString('zh-CN')}</td>
                    <td>{formatAction(entry.action)}</td>
                    <td>{entry.target}</td>
                    <td>{formatStatus(entry)}</td>
                    <td>{entry.duration_ms == null ? '-' : `${entry.duration_ms}ms`}</td>
                    <td>{entry.summary}</td>
                    <td>
                      <button
                        type="button"
                        className="btn"
                        disabled={!hasDetails(entry)}
                        onClick={() => setSelectedEntry(entry)}
                      >
                        查看
                      </button>
                    </td>
                  </tr>
                </Fragment>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {selectedEntry && (
        <div className="history-modal-backdrop" onClick={() => setSelectedEntry(null)}>
          <div className="history-modal" onClick={(event) => event.stopPropagation()}>
            <div className="panel-header">
              <h2>历史详情</h2>
              <button type="button" className="btn" onClick={() => setSelectedEntry(null)}>
                关闭
              </button>
            </div>
            <p className="muted">
              {new Date(selectedEntry.recorded_at).toLocaleString('zh-CN')} |{' '}
              {formatAction(selectedEntry.action)} | {selectedEntry.target}
            </p>
            {selectedEntry.command && (
              <>
                <strong>命令</strong>
                <pre>{selectedEntry.command}</pre>
              </>
            )}
            {selectedEntry.stdout && selectedEntry.stdout.trim() && (
              <>
                <strong>标准输出（stdout）</strong>
                <pre>{selectedEntry.stdout}</pre>
              </>
            )}
            {selectedEntry.stderr && selectedEntry.stderr.trim() && (
              <>
                <strong>错误输出（stderr）</strong>
                <pre>{selectedEntry.stderr}</pre>
              </>
            )}
          </div>
        </div>
      )}
    </section>
  );
}
