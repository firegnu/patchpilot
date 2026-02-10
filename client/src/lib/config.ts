import type { AppConfig } from '../types/app';

export const validateConfig = (config: AppConfig): string | null => {
  if (!Number.isFinite(config.check_interval_minutes) || config.check_interval_minutes < 1) {
    return 'check_interval_minutes 必须大于等于 1';
  }
  if (!Number.isFinite(config.command_timeout_seconds) || config.command_timeout_seconds < 1) {
    return 'command_timeout_seconds 必须大于等于 1';
  }
  if (!Array.isArray(config.items) || config.items.length === 0) {
    return 'items 必须是非空数组';
  }
  if (!['light', 'dark', 'system'].includes(config.theme_mode)) {
    return 'theme_mode 仅支持 light / dark / system';
  }
  if (typeof config.auto_check_enabled !== 'boolean') {
    return 'auto_check_enabled 必须是 boolean';
  }
  if (typeof config.auto_check_manual_enabled !== 'boolean') {
    return 'auto_check_manual_enabled 必须是 boolean';
  }
  return null;
};

export const normalizeConfig = (payload: Partial<AppConfig>): AppConfig => ({
  ...payload,
  check_interval_minutes: Number(payload.check_interval_minutes),
  command_timeout_seconds: Number(payload.command_timeout_seconds ?? 120),
  theme_mode:
    payload.theme_mode === 'light' || payload.theme_mode === 'dark' || payload.theme_mode === 'system'
      ? payload.theme_mode
      : 'system',
  auto_check_enabled: typeof payload.auto_check_enabled === 'boolean' ? payload.auto_check_enabled : true,
  auto_check_manual_enabled:
    typeof payload.auto_check_manual_enabled === 'boolean'
      ? payload.auto_check_manual_enabled
      : true,
  shared_update_commands: payload.shared_update_commands ?? [],
  items: payload.items ?? [],
});
