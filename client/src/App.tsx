import { useEffect, useMemo, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import MonitorPanel from './components/MonitorPanel';
import SharedCommandsPanel from './components/SharedCommandsPanel';
import {
  checkAll,
  checkAutoAppItems,
  checkAutoCliItems,
  checkRuntimeItems,
  checkItem,
  loadConfig,
  getActiveNodeVersion,
  loadHistory,
  loadLatestResults,
  detectInstalledItems,
  runAdHocCommand,
  runItemUpdate,
  saveConfig,
} from './lib/ipc';
import { normalizeConfig } from './lib/config';
import { applyThemeMode } from './lib/theme';
import type { AppConfig, CheckResult, ExecutionHistoryEntry, SoftwareItem, ThemeMode } from './types/app';
const formatError = (error: unknown): string => (error instanceof Error ? error.message : String(error));
const themeModeLabel = (mode: ThemeMode): string => ({ system: '跟随系统', light: '浅色', dark: '深色' })[mode];
const isThemeMode = (value: unknown): value is ThemeMode =>
  value === 'system' || value === 'light' || value === 'dark';
const isManualItem = (item: SoftwareItem): boolean => item.id === 'brew' || item.id === 'bun';
const mapLatestResultsToResultMap = (items: Record<string, { item_id: string; checked_at: string; has_update: boolean; current_version: string | null; latest_version: string | null; error: string | null }>): Record<string, CheckResult> => {
  const next: Record<string, CheckResult> = {};
  Object.values(items).forEach((value) => {
    next[value.item_id] = {
      item_id: value.item_id,
      checked_at: value.checked_at,
      has_update: value.has_update,
      current_version: value.current_version,
      latest_version: value.latest_version,
      details: 'latest snapshot',
      error: value.error,
    };
  });
  return next;
};

interface CheckSectionReport {
  checkedCount: number;
  errorCount: number;
  firstError: string | null;
  skipped: boolean;
}

interface AutoCycleInsight {
  running: boolean;
  lastRunAt: string | null;
  nextRunAt: string | null;
  lastDurationMs: number | null;
  lastCheckedCount: number;
  lastFailureReason: string | null;
}

const emptySectionReport = (): CheckSectionReport => ({
  checkedCount: 0,
  errorCount: 0,
  firstError: null,
  skipped: false,
});

const sectionReportFromResults = (results: CheckResult[]): CheckSectionReport => {
  const firstError = results.find((item) => item.error)?.error ?? null;
  const errorCount = results.filter((item) => item.error).length;
  return {
    checkedCount: results.length,
    errorCount,
    firstError,
    skipped: false,
  };
};

const formatDateTimeLabel = (raw: string | null): string =>
  raw ? new Date(raw).toLocaleString('zh-CN', { hour12: false }) : '-';

const formatDurationLabel = (value: number | null): string => {
  if (value == null) {
    return '-';
  }
  if (value < 1000) {
    return `${value} ms`;
  }
  return `${(value / 1000).toFixed(1)} s`;
};

export default function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [message, setMessage] = useState('正在加载配置...');
  const [resultMap, setResultMap] = useState<Record<string, CheckResult>>({});
  const [checkingMap, setCheckingMap] = useState<Record<string, boolean>>({});
  const [autoCheckingMap, setAutoCheckingMap] = useState<Record<string, boolean>>({});
  const [updatingMap, setUpdatingMap] = useState<Record<string, boolean>>({});
  const [historyEntries, setHistoryEntries] = useState<ExecutionHistoryEntry[]>([]);
  const [activeNodeVersion, setActiveNodeVersion] = useState('');
  const [installedMap, setInstalledMap] = useState<Record<string, boolean>>({});
  const [checkAllRunning, setCheckAllRunning] = useState(false);
  const [runtimeCheckRunning, setRuntimeCheckRunning] = useState(false);
  const [autoCliCheckRunning, setAutoCliCheckRunning] = useState(false);
  const [autoAppCheckRunning, setAutoAppCheckRunning] = useState(false);
  const [autoCycleInsight, setAutoCycleInsight] = useState<AutoCycleInsight>({
    running: false,
    lastRunAt: null,
    nextRunAt: null,
    lastDurationMs: null,
    lastCheckedCount: 0,
    lastFailureReason: null,
  });
  const checkAllRunningRef = useRef(false);
  const runtimeCheckRunningRef = useRef(false);
  const autoCheckCycleRunningRef = useRef(false);
  const autoCliCheckRunningRef = useRef(false);
  const autoAppCheckRunningRef = useRef(false);
  const enabledItems = useMemo(
    () => config?.items.filter((item) => item.enabled && installedMap[item.id] !== false) ?? [],
    [config, installedMap]
  );
  const manualItems = useMemo(() => enabledItems.filter((item) => isManualItem(item)), [enabledItems]);
  const autoCliItems = useMemo(
    () => enabledItems.filter((item) => !isManualItem(item) && item.kind === 'cli'),
    [enabledItems]
  );
  const runtimeItems = useMemo(
    () => enabledItems.filter((item) => item.kind === 'runtime'),
    [enabledItems]
  );
  const autoAppItems = useMemo(
    () => enabledItems.filter((item) => !isManualItem(item) && (item.kind === 'gui' || item.kind === 'app')),
    [enabledItems]
  );
  const mergedCheckingMap = useMemo(
    () => ({ ...checkingMap, ...autoCheckingMap }),
    [checkingMap, autoCheckingMap]
  );
  const latestCheckAllEntry = useMemo(() => historyEntries.find((entry) => entry.action === 'check-all' || entry.action === 'check-all-skip') ?? null, [historyEntries]);
  const latestRuntimeCheckEntry = useMemo(() => historyEntries.find((entry) => entry.action === 'check-runtime' || entry.action === 'check-runtime-skip') ?? null, [historyEntries]);
  const latestAutoCliCheckEntry = useMemo(() => historyEntries.find((entry) => entry.action === 'auto-check-cli' || entry.action === 'auto-check-cli-skip') ?? null, [historyEntries]);
  const latestAutoAppCheckEntry = useMemo(() => historyEntries.find((entry) => entry.action === 'auto-check-app' || entry.action === 'auto-check-app-skip') ?? null, [historyEntries]);
  const refreshHistory = async (): Promise<void> => {
    try {
      const entries = await loadHistory(50);
      setHistoryEntries(entries);
    } catch (error) {
      console.error('加载历史失败', error);
    }
  };
  const refreshLatestResults = async (): Promise<void> => {
    try {
      const latest = await loadLatestResults();
      setResultMap(mapLatestResultsToResultMap(latest.items));
    } catch (error) {
      console.error('加载最近检查结果失败', error);
    }
  };
  const refreshActiveNodeVersion = async (): Promise<void> => {
    try {
      const version = (await getActiveNodeVersion()).trim();
      setActiveNodeVersion(version);
    } catch (error) {
      console.error('加载当前 Node 版本失败', error);
      setActiveNodeVersion('');
    }
  };
  const reloadConfig = async (): Promise<void> => {
    const nextConfig = normalizeConfig((await loadConfig()) as Partial<AppConfig>);
    setConfig(nextConfig);
    detectInstalledItems()
      .then(setInstalledMap)
      .catch((error) => console.error('检测安装状态失败', error));
    await refreshLatestResults();
    await refreshHistory();
    await refreshActiveNodeVersion();
    setMessage('配置已加载。');
  };
  useEffect(() => {
    void reloadConfig().catch((error) => setMessage(`加载配置失败：${formatError(error)}`));
  }, []);
  useEffect(() => (config ? applyThemeMode(config.theme_mode) : undefined), [config?.theme_mode]);
  useEffect(() => {
    if (!config || !config.auto_check_enabled) {
      setAutoCycleInsight((prev) => ({ ...prev, running: false, nextRunAt: null }));
      return;
    }
    const intervalMs = Math.max(config.check_interval_minutes, 1) * 60 * 1000;
    setAutoCycleInsight((prev) => ({
      ...prev,
      nextRunAt: new Date(Date.now() + intervalMs).toISOString(),
    }));
    void handleAutoCheckCycle();
    const timer = setInterval(() => void handleAutoCheckCycle(), intervalMs);
    return () => clearInterval(timer);
  }, [config]);
  useEffect(() => {
    let unlistenConfig: (() => void) | undefined;
    let unlistenLatest: (() => void) | undefined;
    let unlistenHistory: (() => void) | undefined;
    let unlistenThemeMode: (() => void) | undefined;

    void (async () => {
      unlistenConfig = await listen('patchpilot://config-updated', () => {
        void reloadConfig();
      });
      unlistenLatest = await listen('patchpilot://latest-results-updated', () => {
        void refreshLatestResults();
      });
      unlistenHistory = await listen('patchpilot://history-updated', () => {
        void refreshHistory();
      });
      unlistenThemeMode = await listen<ThemeMode>('patchpilot://theme-mode-updated', (event) => {
        const mode = event.payload;
        if (!isThemeMode(mode)) {
          return;
        }
        setConfig((prev) => (prev ? { ...prev, theme_mode: mode } : prev));
      });
    })();

    return () => {
      unlistenConfig?.();
      unlistenLatest?.();
      unlistenHistory?.();
      unlistenThemeMode?.();
    };
  }, []);
  const setAutoItemsChecking = (itemIds: string[], checking: boolean): void => {
    if (itemIds.length === 0) {
      return;
    }
    setAutoCheckingMap((prev) => {
      const next = { ...prev };
      itemIds.forEach((id) => {
        next[id] = checking;
      });
      return next;
    });
  };
  const handleCheckItem = async (itemId: string): Promise<void> => {
    setCheckingMap((prev) => ({ ...prev, [itemId]: true }));
    try {
      const result = await checkItem(itemId);
      setResultMap((prev) => ({ ...prev, [itemId]: result }));
      await refreshHistory();
      if (itemId === 'node-lts-nvm') {
        await refreshActiveNodeVersion();
      }
      setMessage(`已检查：${itemId}`);
    } catch (error) {
      setMessage(`检查失败：${formatError(error)}`);
    } finally {
      setCheckingMap((prev) => ({ ...prev, [itemId]: false }));
    }
  };
  const runManualCheck = async (silent = false): Promise<CheckSectionReport> => {
    if (!config || checkAllRunningRef.current) {
      return { ...emptySectionReport(), skipped: true };
    }
    checkAllRunningRef.current = true;
    setCheckAllRunning(true);
    if (!silent) {
      setMessage('正在检查全部启用项...');
    }
    try {
      const results = await checkAll();
      const nextMap = results.reduce<Record<string, CheckResult>>((acc, item) => {
        acc[item.item_id] = item;
        return acc;
      }, {});
      setResultMap((prev) => ({ ...prev, ...nextMap }));
      await refreshHistory();
      if (!silent) {
        setMessage('全量检查完成。');
      }
      return sectionReportFromResults(results);
    } catch (error) {
      const text = formatError(error);
      const skipped = text.includes('already running');
      if (!silent) {
        setMessage(skipped ? '全量检查已跳过：上一轮仍在执行。' : `全量检查失败：${text}`);
      }
      return {
        ...emptySectionReport(),
        errorCount: skipped ? 0 : 1,
        firstError: skipped ? null : text,
        skipped,
      };
    } finally {
      checkAllRunningRef.current = false;
      setCheckAllRunning(false);
    }
  };
  const handleCheckAll = async (): Promise<void> => {
    await runManualCheck(false);
  };
  const runAutoCliCheck = async (): Promise<CheckSectionReport> => {
    if (!config || autoCliCheckRunningRef.current) {
      return { ...emptySectionReport(), skipped: true };
    }
    const itemIds = autoCliItems.map((item) => item.id);
    autoCliCheckRunningRef.current = true;
    setAutoCliCheckRunning(true);
    setAutoItemsChecking(itemIds, true);
    try {
      const results = await checkAutoCliItems();
      if (results.length > 0) {
        const nextMap = results.reduce<Record<string, CheckResult>>((acc, item) => {
          acc[item.item_id] = item;
          return acc;
        }, {});
        setResultMap((prev) => ({ ...prev, ...nextMap }));
      }
      await refreshHistory();
      return sectionReportFromResults(results);
    } catch (error) {
      const text = formatError(error);
      console.error('CLI 自动检查失败', error);
      return { ...emptySectionReport(), errorCount: 1, firstError: text };
    } finally {
      setAutoItemsChecking(itemIds, false);
      autoCliCheckRunningRef.current = false;
      setAutoCliCheckRunning(false);
    }
  };
  const handleAutoCliCheck = async (): Promise<void> => {
    await runAutoCliCheck();
  };
  const handleRuntimeCheck = async (): Promise<void> => {
    if (!config || runtimeCheckRunningRef.current) {
      return;
    }
    const itemIds = runtimeItems.map((item) => item.id);
    runtimeCheckRunningRef.current = true;
    setRuntimeCheckRunning(true);
    setAutoItemsChecking(itemIds, true);
    try {
      const results = await checkRuntimeItems();
      if (results.length > 0) {
        const nextMap = results.reduce<Record<string, CheckResult>>((acc, item) => {
          acc[item.item_id] = item;
          return acc;
        }, {});
        setResultMap((prev) => ({ ...prev, ...nextMap }));
      }
      await refreshHistory();
      await refreshActiveNodeVersion();
    } catch (error) {
      console.error('运行时手动检查失败', error);
    } finally {
      setAutoItemsChecking(itemIds, false);
      runtimeCheckRunningRef.current = false;
      setRuntimeCheckRunning(false);
    }
  };
  const runAutoAppCheck = async (): Promise<CheckSectionReport> => {
    if (!config || autoAppCheckRunningRef.current) {
      return { ...emptySectionReport(), skipped: true };
    }
    const itemIds = autoAppItems.map((item) => item.id);
    autoAppCheckRunningRef.current = true;
    setAutoAppCheckRunning(true);
    setAutoItemsChecking(itemIds, true);
    try {
      const results = await checkAutoAppItems();
      if (results.length > 0) {
        const nextMap = results.reduce<Record<string, CheckResult>>((acc, item) => {
          acc[item.item_id] = item;
          return acc;
        }, {});
        setResultMap((prev) => ({ ...prev, ...nextMap }));
      }
      await refreshHistory();
      return sectionReportFromResults(results);
    } catch (error) {
      const text = formatError(error);
      console.error('App 自动检查失败', error);
      return { ...emptySectionReport(), errorCount: 1, firstError: text };
    } finally {
      setAutoItemsChecking(itemIds, false);
      autoAppCheckRunningRef.current = false;
      setAutoAppCheckRunning(false);
    }
  };
  const handleAutoAppCheck = async (): Promise<void> => {
    await runAutoAppCheck();
  };
  const handleAutoCheckCycle = async (): Promise<void> => {
    if (!config || autoCheckCycleRunningRef.current) {
      return;
    }
    const cycleStartedAt = Date.now();
    const intervalMs = Math.max(config.check_interval_minutes, 1) * 60 * 1000;
    autoCheckCycleRunningRef.current = true;
    setAutoCycleInsight((prev) => ({ ...prev, running: true }));
    try {
      const reports: CheckSectionReport[] = [];
      if (config.auto_check_manual_enabled) {
        reports.push(await runManualCheck(true));
      }
      reports.push(await runAutoCliCheck());
      reports.push(await runAutoAppCheck());
      const lastFailureReason =
        reports.find((item) => item.firstError)?.firstError ??
        (reports.some((item) => item.errorCount > 0) ? '检查出现异常，请查看分区状态。' : null);
      const checkedCount = reports.reduce((total, item) => total + item.checkedCount, 0);
      setAutoCycleInsight({
        running: false,
        lastRunAt: new Date().toISOString(),
        nextRunAt: new Date(Date.now() + intervalMs).toISOString(),
        lastDurationMs: Date.now() - cycleStartedAt,
        lastCheckedCount: checkedCount,
        lastFailureReason,
      });
    } catch (error) {
      setAutoCycleInsight((prev) => ({
        ...prev,
        running: false,
        lastRunAt: new Date().toISOString(),
        nextRunAt: new Date(Date.now() + intervalMs).toISOString(),
        lastDurationMs: Date.now() - cycleStartedAt,
        lastFailureReason: formatError(error),
      }));
    } finally {
      autoCheckCycleRunningRef.current = false;
    }
  };
  const handleRunUpdate = async (item: SoftwareItem): Promise<void> => {
    setUpdatingMap((prev) => ({ ...prev, [item.id]: true }));
    setMessage(`正在更新 ${item.name}...`);
    try {
      const result = await runItemUpdate(item.id);
      await refreshHistory();
      setMessage(`${item.name} 更新完成（退出码 ${result.output.exit_code}）。`);
      await handleCheckItem(item.id);
    } catch (error) {
      setMessage(`更新失败：${formatError(error)}`);
    } finally {
      setUpdatingMap((prev) => ({ ...prev, [item.id]: false }));
    }
  };
  const handleRunSharedCommand = async (command: string): Promise<void> => {
    setMessage('正在执行共享命令...');
    try {
      const output = await runAdHocCommand(command);
      await refreshHistory();
      setMessage(`共享命令执行完成（退出码 ${output.exit_code}）。`);
    } catch (error) {
      setMessage(`共享命令执行失败：${formatError(error)}`);
    }
  };
  const handleChangeThemeMode = async (mode: ThemeMode): Promise<void> => {
    if (!config || config.theme_mode === mode) {
      return;
    }
    const nextConfig = { ...config, theme_mode: mode };
    setConfig(nextConfig);
    try {
      await saveConfig(nextConfig);
      setMessage(`主题已切换为：${themeModeLabel(mode)}。`);
    } catch (error) {
      setMessage(`主题保存失败：${formatError(error)}`);
    }
  };
  const handleToggleManualAutoCheck = async (enabled: boolean): Promise<void> => {
    if (!config || config.auto_check_manual_enabled === enabled) {
      return;
    }
    const nextConfig = { ...config, auto_check_manual_enabled: enabled };
    setConfig(nextConfig);
    try {
      await saveConfig(nextConfig);
      setMessage(enabled ? '已开启手动区自动检查。' : '已关闭手动区自动检查。');
    } catch (error) {
      setMessage(`手动区自动检查设置保存失败：${formatError(error)}`);
    }
  };
  const autoCycleStateLabel = !config?.auto_check_enabled
    ? '已暂停'
    : autoCycleInsight.running
      ? '运行中'
      : '空闲';
  const autoCycleStateClass = !config?.auto_check_enabled
    ? 'status-skipped'
    : autoCycleInsight.running
      ? 'status-running'
      : autoCycleInsight.lastFailureReason
        ? 'status-failed'
        : 'status-success';
  return (
    <main>
      <header className="app-header">
        <div className="app-title-group">
          <h1>PatchPilot</h1>
          <p className="app-subtitle">本地软件版本巡检与手动更新</p>
        </div>
        <p className="app-status">{message}</p>
      </header>
      <section className="panel scheduler-panel">
        <div className="panel-header">
          <h2>自动调度状态</h2>
          <span className={`status-badge ${autoCycleStateClass}`}>{autoCycleStateLabel}</span>
        </div>
        <div className="scheduler-grid">
          <div className="scheduler-item">
            <span className="scheduler-label">下次自动检查</span>
            <strong className="scheduler-value">{formatDateTimeLabel(autoCycleInsight.nextRunAt)}</strong>
          </div>
          <div className="scheduler-item">
            <span className="scheduler-label">上次自动检查</span>
            <strong className="scheduler-value">{formatDateTimeLabel(autoCycleInsight.lastRunAt)}</strong>
          </div>
          <div className="scheduler-item">
            <span className="scheduler-label">上次耗时</span>
            <strong className="scheduler-value">{formatDurationLabel(autoCycleInsight.lastDurationMs)}</strong>
          </div>
          <div className="scheduler-item">
            <span className="scheduler-label">上次检查项数</span>
            <strong className="scheduler-value">{autoCycleInsight.lastCheckedCount || '-'}</strong>
          </div>
        </div>
        <p className={`scheduler-message ${autoCycleInsight.lastFailureReason ? 'scheduler-message-error' : ''}`}>
          最近失败原因：{autoCycleInsight.lastFailureReason ?? '无'}
        </p>
      </section>
      {config && (
        <SharedCommandsPanel
          checkIntervalMinutes={config.check_interval_minutes}
          timeoutSeconds={config.command_timeout_seconds}
          themeMode={config.theme_mode}
          autoCheckManualEnabled={config.auto_check_manual_enabled}
          commands={config.shared_update_commands}
          onRunSharedCommand={handleRunSharedCommand}
          onChangeThemeMode={handleChangeThemeMode}
          onToggleManualAutoCheck={handleToggleManualAutoCheck}
        />
      )}
      <MonitorPanel
        title="Homebrew 与 Bun（手动区）"
        batchLabel="手动全量检查"
        items={manualItems}
        resultMap={resultMap}
        checkingMap={mergedCheckingMap}
        updatingMap={updatingMap}
        checkAllRunning={checkAllRunning}
        latestCheckAllEntry={latestCheckAllEntry}
        onCheckItem={handleCheckItem}
        onCheckAll={handleCheckAll}
        onRunUpdate={handleRunUpdate}
      />
      <MonitorPanel
        title="CLI 工具（自动检查 + 手动更新）"
        batchLabel="立即检查 CLI 工具"
        items={autoCliItems}
        resultMap={resultMap}
        checkingMap={mergedCheckingMap}
        updatingMap={updatingMap}
        checkAllRunning={autoCliCheckRunning}
        latestCheckAllEntry={latestAutoCliCheckEntry}
        onCheckItem={handleCheckItem}
        onCheckAll={handleAutoCliCheck}
        onRunUpdate={handleRunUpdate}
      />
      <p className="runtime-hint">当前系统 Node 版本：{activeNodeVersion || '-'}</p>
      <MonitorPanel
        title="开发运行时（手动检查 + 手动更新）"
        batchLabel="立即检查运行时"
        items={runtimeItems}
        resultMap={resultMap}
        checkingMap={mergedCheckingMap}
        updatingMap={updatingMap}
        checkAllRunning={runtimeCheckRunning}
        latestCheckAllEntry={latestRuntimeCheckEntry}
        onCheckItem={handleCheckItem}
        onCheckAll={handleRuntimeCheck}
        onRunUpdate={handleRunUpdate}
      />
      <MonitorPanel
        title="App（自动检查）"
        batchLabel="立即检查 App"
        showUpdateButton={false}
        items={autoAppItems}
        resultMap={resultMap}
        checkingMap={mergedCheckingMap}
        updatingMap={updatingMap}
        checkAllRunning={autoAppCheckRunning}
        latestCheckAllEntry={latestAutoAppCheckEntry}
        onCheckItem={handleCheckItem}
        onCheckAll={handleAutoAppCheck}
        onRunUpdate={handleRunUpdate}
      />
    </main>
  );
}
