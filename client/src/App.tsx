import { useEffect, useMemo, useRef, useState } from 'react';
import ConfigEditor from './components/ConfigEditor';
import HistoryPanel from './components/HistoryPanel';
import MonitorPanel from './components/MonitorPanel';
import SharedCommandsPanel from './components/SharedCommandsPanel';
import { checkAll, checkAutoItems, checkItem, loadConfig, loadHistory, runAdHocCommand, runItemUpdate, saveConfig } from './lib/ipc';
import { normalizeConfig, validateConfig } from './lib/config';
import { applyThemeMode } from './lib/theme';
import type { AppConfig, CheckResult, ExecutionHistoryEntry, SoftwareItem, ThemeMode } from './types/app';
const HISTORY_PAGE_SIZE = 20;
const HISTORY_MAX_LIMIT = 200;
const formatError = (error: unknown): string => (error instanceof Error ? error.message : String(error));
const themeModeLabel = (mode: ThemeMode): string => ({ system: '跟随系统', light: '浅色', dark: '深色' })[mode];
const isManualItem = (item: SoftwareItem): boolean => item.id === 'brew' || item.id === 'bun';
export default function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [editorText, setEditorText] = useState('');
  const [message, setMessage] = useState('正在加载配置...');
  const [resultMap, setResultMap] = useState<Record<string, CheckResult>>({});
  const [checkingMap, setCheckingMap] = useState<Record<string, boolean>>({});
  const [autoCheckingMap, setAutoCheckingMap] = useState<Record<string, boolean>>({});
  const [updatingMap, setUpdatingMap] = useState<Record<string, boolean>>({});
  const [historyEntries, setHistoryEntries] = useState<ExecutionHistoryEntry[]>([]);
  const [historyLimit, setHistoryLimit] = useState(HISTORY_PAGE_SIZE);
  const [checkAllRunning, setCheckAllRunning] = useState(false);
  const [autoCheckRunning, setAutoCheckRunning] = useState(false);
  const checkAllRunningRef = useRef(false);
  const autoCheckRunningRef = useRef(false);
  const enabledItems = useMemo(() => config?.items.filter((item) => item.enabled) ?? [], [config]);
  const manualItems = useMemo(() => enabledItems.filter((item) => isManualItem(item)), [enabledItems]);
  const autoItems = useMemo(() => enabledItems.filter((item) => !isManualItem(item)), [enabledItems]);
  const mergedCheckingMap = useMemo(
    () => ({ ...checkingMap, ...autoCheckingMap }),
    [checkingMap, autoCheckingMap]
  );
  const latestCheckAllEntry = useMemo(() => historyEntries.find((entry) => entry.action === 'check-all' || entry.action === 'check-all-skip') ?? null, [historyEntries]);
  const latestAutoCheckEntry = useMemo(() => historyEntries.find((entry) => entry.action === 'auto-check' || entry.action === 'auto-check-skip') ?? null, [historyEntries]);
  const refreshHistory = async (limit = historyLimit): Promise<void> => {
    try {
      const entries = await loadHistory(limit);
      setHistoryEntries(entries);
    } catch (error) {
      console.error('加载历史失败', error);
    }
  };
  const reloadConfig = async (): Promise<void> => {
    const nextConfig = normalizeConfig((await loadConfig()) as Partial<AppConfig>);
    setConfig(nextConfig);
    setEditorText(JSON.stringify(nextConfig, null, 2));
    await refreshHistory(historyLimit);
    setMessage('配置已加载。');
  };
  useEffect(() => {
    void reloadConfig().catch((error) => setMessage(`加载配置失败：${formatError(error)}`));
  }, []);
  useEffect(() => (config ? applyThemeMode(config.theme_mode) : undefined), [config?.theme_mode]);
  useEffect(() => {
    if (!config) return;
    void handleAutoCheck();
    const timer = setInterval(() => void handleAutoCheck(), Math.max(config.check_interval_minutes, 1) * 60 * 1000);
    return () => clearInterval(timer);
  }, [config]);
  const handleCheckItem = async (itemId: string): Promise<void> => {
    setCheckingMap((prev) => ({ ...prev, [itemId]: true }));
    try {
      const result = await checkItem(itemId);
      setResultMap((prev) => ({ ...prev, [itemId]: result }));
      await refreshHistory();
      setMessage(`已检查：${itemId}`);
    } catch (error) {
      setMessage(`检查失败：${formatError(error)}`);
    } finally {
      setCheckingMap((prev) => ({ ...prev, [itemId]: false }));
    }
  };
  const handleCheckAll = async (): Promise<void> => {
    if (!config || checkAllRunningRef.current) {
      return;
    }
    checkAllRunningRef.current = true;
    setCheckAllRunning(true);
    setMessage('正在检查全部启用项...');
    try {
      const results = await checkAll();
      const nextMap = results.reduce<Record<string, CheckResult>>((acc, item) => {
        acc[item.item_id] = item;
        return acc;
      }, {});
      setResultMap((prev) => ({ ...prev, ...nextMap }));
      await refreshHistory();
      setMessage('全量检查完成。');
    } catch (error) {
      const text = formatError(error);
      setMessage(text.includes('already running') ? '全量检查已跳过：上一轮仍在执行。' : `全量检查失败：${text}`);
    } finally {
      checkAllRunningRef.current = false;
      setCheckAllRunning(false);
    }
  };
  const handleAutoCheck = async (): Promise<void> => {
    if (!config || autoCheckRunningRef.current) {
      return;
    }
    const autoItemIds = config.items
      .filter((item) => item.enabled && !isManualItem(item))
      .map((item) => item.id);
    autoCheckRunningRef.current = true;
    setAutoCheckRunning(true);
    if (autoItemIds.length > 0) {
      setAutoCheckingMap((prev) => {
        const next = { ...prev };
        autoItemIds.forEach((id) => {
          next[id] = true;
        });
        return next;
      });
    }
    try {
      const results = await checkAutoItems();
      if (results.length > 0) {
        const nextMap = results.reduce<Record<string, CheckResult>>((acc, item) => {
          acc[item.item_id] = item;
          return acc;
        }, {});
        setResultMap((prev) => ({ ...prev, ...nextMap }));
        await refreshHistory();
      }
    } catch (error) {
      console.error('自动检查失败', error);
    } finally {
      if (autoItemIds.length > 0) {
        setAutoCheckingMap((prev) => {
          const next = { ...prev };
          autoItemIds.forEach((id) => {
            next[id] = false;
          });
          return next;
        });
      }
      autoCheckRunningRef.current = false;
      setAutoCheckRunning(false);
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
  const handleLoadMoreHistory = async (): Promise<void> => {
    const nextLimit = Math.min(historyLimit + HISTORY_PAGE_SIZE, HISTORY_MAX_LIMIT);
    if (nextLimit <= historyLimit) {
      setMessage('历史记录已达到展示上限（200 条）。');
      return;
    }
    setHistoryLimit(nextLimit);
    await refreshHistory(nextLimit);
  };
  const handleChangeThemeMode = async (mode: ThemeMode): Promise<void> => {
    if (!config || config.theme_mode === mode) {
      return;
    }
    const nextConfig = { ...config, theme_mode: mode };
    setConfig(nextConfig);
    setEditorText(JSON.stringify(nextConfig, null, 2));
    try {
      await saveConfig(nextConfig);
      setMessage(`主题已切换为：${themeModeLabel(mode)}。`);
    } catch (error) {
      setMessage(`主题保存失败：${formatError(error)}`);
    }
  };
  const handleSaveConfig = async (): Promise<void> => {
    try {
      const parsed = JSON.parse(editorText) as Partial<AppConfig>;
      const normalized = normalizeConfig(parsed);
      const error = validateConfig(normalized);
      if (error) {
        setMessage(`配置校验失败：${error}`);
        return;
      }
      await saveConfig(normalized);
      setConfig(normalized);
      setMessage('配置已保存。');
    } catch (error) {
      setMessage(`保存失败：${formatError(error)}`);
    }
  };
  return (
    <main>
      <header>
        <h1>PatchPilot</h1>
        <p>{message}</p>
      </header>
      {config && <SharedCommandsPanel checkIntervalMinutes={config.check_interval_minutes} timeoutSeconds={config.command_timeout_seconds} themeMode={config.theme_mode} commands={config.shared_update_commands} onRunSharedCommand={handleRunSharedCommand} onChangeThemeMode={handleChangeThemeMode} />}
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
        title="CLI 与 App（自动区）"
        batchLabel="立即检查自动区"
        items={autoItems}
        resultMap={resultMap}
        checkingMap={mergedCheckingMap}
        updatingMap={updatingMap}
        checkAllRunning={autoCheckRunning}
        latestCheckAllEntry={latestAutoCheckEntry}
        onCheckItem={handleCheckItem}
        onCheckAll={handleAutoCheck}
        onRunUpdate={handleRunUpdate}
      />
      <ConfigEditor value={editorText} onChange={setEditorText} onSave={handleSaveConfig} onReload={reloadConfig} />
      <HistoryPanel entries={historyEntries} canLoadMore={historyEntries.length >= historyLimit && historyLimit < HISTORY_MAX_LIMIT} onLoadMore={handleLoadMoreHistory} onReload={refreshHistory} />
    </main>
  );
}
