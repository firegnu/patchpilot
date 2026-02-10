import { useEffect, useMemo, useRef, useState } from 'react';
import MonitorPanel from './components/MonitorPanel';
import SharedCommandsPanel from './components/SharedCommandsPanel';
import {
  checkAll,
  checkAutoAppItems,
  checkAutoCliItems,
  checkItem,
  loadConfig,
  loadHistory,
  runAdHocCommand,
  runItemUpdate,
  saveConfig,
} from './lib/ipc';
import { normalizeConfig } from './lib/config';
import { applyThemeMode } from './lib/theme';
import type { AppConfig, CheckResult, ExecutionHistoryEntry, SoftwareItem, ThemeMode } from './types/app';
const formatError = (error: unknown): string => (error instanceof Error ? error.message : String(error));
const themeModeLabel = (mode: ThemeMode): string => ({ system: '跟随系统', light: '浅色', dark: '深色' })[mode];
const isManualItem = (item: SoftwareItem): boolean => item.id === 'brew' || item.id === 'bun';
export default function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [message, setMessage] = useState('正在加载配置...');
  const [resultMap, setResultMap] = useState<Record<string, CheckResult>>({});
  const [checkingMap, setCheckingMap] = useState<Record<string, boolean>>({});
  const [autoCheckingMap, setAutoCheckingMap] = useState<Record<string, boolean>>({});
  const [updatingMap, setUpdatingMap] = useState<Record<string, boolean>>({});
  const [historyEntries, setHistoryEntries] = useState<ExecutionHistoryEntry[]>([]);
  const [checkAllRunning, setCheckAllRunning] = useState(false);
  const [autoCliCheckRunning, setAutoCliCheckRunning] = useState(false);
  const [autoAppCheckRunning, setAutoAppCheckRunning] = useState(false);
  const checkAllRunningRef = useRef(false);
  const autoCheckCycleRunningRef = useRef(false);
  const autoCliCheckRunningRef = useRef(false);
  const autoAppCheckRunningRef = useRef(false);
  const enabledItems = useMemo(() => config?.items.filter((item) => item.enabled) ?? [], [config]);
  const manualItems = useMemo(() => enabledItems.filter((item) => isManualItem(item)), [enabledItems]);
  const autoCliItems = useMemo(
    () => enabledItems.filter((item) => !isManualItem(item) && item.kind === 'cli'),
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
  const reloadConfig = async (): Promise<void> => {
    const nextConfig = normalizeConfig((await loadConfig()) as Partial<AppConfig>);
    setConfig(nextConfig);
    await refreshHistory();
    setMessage('配置已加载。');
  };
  useEffect(() => {
    void reloadConfig().catch((error) => setMessage(`加载配置失败：${formatError(error)}`));
  }, []);
  useEffect(() => (config ? applyThemeMode(config.theme_mode) : undefined), [config?.theme_mode]);
  useEffect(() => {
    if (!config) return;
    void handleAutoCheckCycle();
    const timer = setInterval(() => void handleAutoCheckCycle(), Math.max(config.check_interval_minutes, 1) * 60 * 1000);
    return () => clearInterval(timer);
  }, [config]);
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
  const handleAutoCliCheck = async (): Promise<void> => {
    if (!config || autoCliCheckRunningRef.current) {
      return;
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
    } catch (error) {
      console.error('CLI 自动检查失败', error);
    } finally {
      setAutoItemsChecking(itemIds, false);
      autoCliCheckRunningRef.current = false;
      setAutoCliCheckRunning(false);
    }
  };
  const handleAutoAppCheck = async (): Promise<void> => {
    if (!config || autoAppCheckRunningRef.current) {
      return;
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
    } catch (error) {
      console.error('App 自动检查失败', error);
    } finally {
      setAutoItemsChecking(itemIds, false);
      autoAppCheckRunningRef.current = false;
      setAutoAppCheckRunning(false);
    }
  };
  const handleAutoCheckCycle = async (): Promise<void> => {
    if (!config || autoCheckCycleRunningRef.current) {
      return;
    }
    autoCheckCycleRunningRef.current = true;
    try {
      await handleAutoCliCheck();
      await handleAutoAppCheck();
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
