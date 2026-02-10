export type SoftwareKind = 'cli' | 'gui' | 'app' | 'runtime';
export type ThemeMode = 'light' | 'dark' | 'system';

export interface SoftwareItem {
  id: string;
  name: string;
  kind: SoftwareKind;
  enabled: boolean;
  description: string;
  current_version_command: string | null;
  latest_version_command: string | null;
  update_check_command: string | null;
  update_check_regex: string | null;
  update_command: string;
}

export interface AppConfig {
  check_interval_minutes: number;
  command_timeout_seconds: number;
  theme_mode: ThemeMode;
  auto_check_enabled: boolean;
  auto_check_manual_enabled: boolean;
  shared_update_commands: string[];
  items: SoftwareItem[];
}

export interface CheckResult {
  item_id: string;
  checked_at: string;
  has_update: boolean;
  current_version: string | null;
  latest_version: string | null;
  details: string;
  error: string | null;
}

export interface LatestResultSnapshot {
  item_id: string;
  checked_at: string;
  has_update: boolean;
  current_version: string | null;
  latest_version: string | null;
  error: string | null;
}

export interface LatestResultState {
  updated_at: string;
  items: Record<string, LatestResultSnapshot>;
}

export interface CommandOutput {
  command: string;
  exit_code: number;
  stdout: string;
  stderr: string;
  duration_ms: number;
  timed_out: boolean;
}

export interface UpdateResult {
  item_id: string;
  updated_at: string;
  output: CommandOutput;
}

export interface ExecutionHistoryEntry {
  id: string;
  action: string;
  target: string;
  command: string | null;
  stdout: string | null;
  stderr: string | null;
  recorded_at: string;
  success: boolean;
  exit_code: number | null;
  timed_out: boolean;
  duration_ms: number | null;
  summary: string;
}
