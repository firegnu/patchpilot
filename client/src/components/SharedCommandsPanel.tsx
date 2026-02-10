import type { ThemeMode } from '../types/app';

interface SharedCommandsPanelProps {
  checkIntervalMinutes: number;
  timeoutSeconds: number;
  themeMode: ThemeMode;
  autoCheckManualEnabled: boolean;
  commands: string[];
  onRunSharedCommand: (command: string) => Promise<void>;
  onChangeThemeMode: (mode: ThemeMode) => Promise<void>;
  onToggleManualAutoCheck: (enabled: boolean) => Promise<void>;
}

export default function SharedCommandsPanel({
  checkIntervalMinutes,
  timeoutSeconds,
  themeMode,
  autoCheckManualEnabled,
  commands,
  onRunSharedCommand,
  onChangeThemeMode,
  onToggleManualAutoCheck,
}: SharedCommandsPanelProps) {
  return (
    <section className="panel">
      <div className="panel-header">
        <h2>共享更新命令</h2>
        <div className="inline-actions">
          <span>
            检查间隔：{checkIntervalMinutes} 分钟 | 超时：{timeoutSeconds} 秒
          </span>
          <label className="theme-mode">
            主题
            <select
              value={themeMode}
              onChange={(event) => void onChangeThemeMode(event.target.value as ThemeMode)}
            >
              <option value="system">跟随系统</option>
              <option value="light">浅色</option>
              <option value="dark">深色</option>
            </select>
          </label>
          <label className="theme-mode">
            <input
              type="checkbox"
              checked={autoCheckManualEnabled}
              onChange={(event) => void onToggleManualAutoCheck(event.target.checked)}
            />
            手动区自动检查
          </label>
        </div>
      </div>
      <div className="inline-actions">
        {commands.map((command) => (
          <button
            key={command}
            type="button"
            className="btn"
            onClick={() => void onRunSharedCommand(command)}
          >
            {command}
          </button>
        ))}
      </div>
    </section>
  );
}
