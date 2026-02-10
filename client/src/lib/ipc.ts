import { invoke } from '@tauri-apps/api/core';
import type {
  AppConfig,
  CheckResult,
  CommandOutput,
  ExecutionHistoryEntry,
  LatestResultState,
  UpdateResult,
} from '../types/app';

export const loadConfig = async (): Promise<AppConfig> => invoke('load_config');

export const saveConfig = async (config: AppConfig): Promise<void> => {
  await invoke('save_config', { config });
};

export const loadLatestResults = async (): Promise<LatestResultState> =>
  invoke('load_latest_results');

export const checkItem = async (itemId: string): Promise<CheckResult> =>
  invoke('check_item', { itemId });

export const checkAll = async (): Promise<CheckResult[]> => invoke('check_all');
export const checkAutoItems = async (): Promise<CheckResult[]> => invoke('check_auto_items');
export const checkAutoCliItems = async (): Promise<CheckResult[]> => invoke('check_auto_cli_items');
export const checkAutoAppItems = async (): Promise<CheckResult[]> => invoke('check_auto_app_items');

export const runItemUpdate = async (itemId: string): Promise<UpdateResult> =>
  invoke('run_item_update', { itemId });

export const runAdHocCommand = async (command: string): Promise<CommandOutput> =>
  invoke('run_ad_hoc_command', { command });

export const loadHistory = async (limit = 50): Promise<ExecutionHistoryEntry[]> =>
  invoke('load_history', { limit });
