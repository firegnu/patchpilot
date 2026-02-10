use std::collections::HashMap;

use crate::model::SoftwareItem;

const OLD_BREW_CHECK_CMD: &str = "brew outdated --quiet brew";
const OLD_BREW_UPDATE_CMD: &str = "brew update && brew upgrade brew";
const OLD_BUN_CHECK_CMD: &str = "brew outdated --quiet bun";
const OLD_BUN_UPDATE_CMD: &str = "brew upgrade bun";
const OLD_BUN_CURRENT_CMD: &str = "bun --version";
const OLD_CLAUDE_CHECK_CMD: &str = "npm outdated -g @anthropic-ai/claude-code --parseable";
const OLD_CLAUDE_UPDATE_CMD: &str = "npm install -g @anthropic-ai/claude-code@latest";
const OLD_CLAUDE_DESC: &str = "Example: update npm-managed claude-code";
const OLD_CODEX_LATEST_CMD: &str =
    "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask codex --json=v2 | sed -E 's/.*\"version\":\"([^\"]+)\".*/\\1/'";
const OLD_LMSTUDIO_CURRENT_CMD: &str = "if [ -d \"/Applications/LM Studio.app\" ]; then defaults read \"/Applications/LM Studio.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi";
const OLD_CHROME_LATEST_CMD: &str =
    "LATEST=\"$(curl -fsSL \"https://versionhistory.googleapis.com/v1/chrome/platforms/mac/channels/stable/versions?page_size=1\" | sed -nE 's/.*\"version\":\"([0-9.]+)\".*/\\1/p' | head -n 1)\"; if [ -n \"$LATEST\" ]; then echo \"$LATEST\"; else curl -fsSL \"https://versionhistory.googleapis.com/v1/chrome/platforms/mac_arm64/channels/stable/versions?page_size=1\" | sed -nE 's/.*\"version\":\"([0-9.]+)\".*/\\1/p' | head -n 1; fi";
const OLD_GHOSTTY_LATEST_CMD: &str =
    "curl -fsSLI -o /dev/null -w '%{url_effective}' https://github.com/ghostty-org/ghostty/releases/latest | sed -E 's#.*/tag/v?##'";
const PREV_GHOSTTY_LATEST_CMD: &str =
    "curl -fsSL https://api.github.com/repos/ghostty-org/ghostty/releases/latest | sed -nE 's/.*\"tag_name\"[[:space:]]*:[[:space:]]*\"v?([^\"]+)\".*/\\1/p' | head -n 1";
const PREV2_GHOSTTY_LATEST_CMD: &str =
    "set -o pipefail; LATEST=\"$(curl -fsSL -H 'Accept: application/vnd.github+json' -H 'User-Agent: PatchPilot' https://api.github.com/repos/ghostty-org/ghostty/releases/latest | tr '\\r\\n' '  ' | sed -nE 's/.*\"tag_name\"[[:space:]]*:[[:space:]]*\"v?([^\"]+)\".*/\\1/p' | head -n 1)\"; if [ -n \"$LATEST\" ]; then echo \"$LATEST\"; else exit 1; fi";
const OLD_RUST_LATEST_CMD: &str =
    "if command -v rustup >/dev/null 2>&1; then OUT=\"$(rustup check 2>/dev/null || true)\"; LATEST=\"$(echo \"$OUT\" | sed -nE 's/.*-> *([0-9][0-9A-Za-z.+-]*).*/\\1/p' | head -n 1)\"; if [ -z \"$LATEST\" ]; then LATEST=\"$(echo \"$OUT\" | sed -nE 's/.*: *([0-9][0-9A-Za-z.+-]*).*/\\1/p' | head -n 1)\"; fi; if [ -n \"$LATEST\" ]; then echo \"$LATEST\"; else rustc --version | sed -nE 's/^rustc ([^ ]+).*/\\1/p'; fi; else echo ''; fi";

fn set_option_if_missing(target: &mut Option<String>, default: &Option<String>) -> bool {
    if target
        .as_deref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
        && default.is_some()
    {
        *target = default.clone();
        return true;
    }
    false
}

fn set_string_if_empty(target: &mut String, default: &str) -> bool {
    if target.trim().is_empty() {
        *target = default.to_string();
        return true;
    }
    false
}

pub(super) fn patch_legacy_item_commands(
    item: &mut SoftwareItem,
    default_items: &HashMap<String, SoftwareItem>,
) -> bool {
    let mut changed = false;
    let Some(default_item) = default_items.get(&item.id) else {
        return false;
    };

    if item.id == "brew" {
        changed |= set_option_if_missing(
            &mut item.current_version_command,
            &default_item.current_version_command,
        );
        changed |= set_option_if_missing(
            &mut item.latest_version_command,
            &default_item.latest_version_command,
        );
        if item.update_check_command.as_deref() == Some(OLD_BREW_CHECK_CMD) {
            item.update_check_command = default_item.update_check_command.clone();
            changed = true;
        }
        if item.update_command == OLD_BREW_UPDATE_CMD {
            item.update_command = default_item.update_command.clone();
            changed = true;
        }
        if item.description == "Check and update Homebrew" {
            item.description = default_item.description.clone();
            changed = true;
        }
    }

    if item.id == "bun" {
        if item.current_version_command.as_deref() == Some(OLD_BUN_CURRENT_CMD) {
            item.current_version_command = default_item.current_version_command.clone();
            changed = true;
        } else {
            changed |= set_option_if_missing(
                &mut item.current_version_command,
                &default_item.current_version_command,
            );
        }
        changed |= set_option_if_missing(
            &mut item.latest_version_command,
            &default_item.latest_version_command,
        );
        if item.update_check_command.as_deref() == Some(OLD_BUN_CHECK_CMD) {
            item.update_check_command = default_item.update_check_command.clone();
            changed = true;
        }
        if item.update_command == OLD_BUN_UPDATE_CMD {
            item.update_command = default_item.update_command.clone();
            changed = true;
        }
    }

    if item.id == "claude-code" {
        let looks_legacy = item.description == OLD_CLAUDE_DESC
            || item.update_check_command.as_deref() == Some(OLD_CLAUDE_CHECK_CMD)
            || item.update_command == OLD_CLAUDE_UPDATE_CMD;
        if looks_legacy {
            item.enabled = default_item.enabled;
            item.description = default_item.description.clone();
            item.current_version_command = default_item.current_version_command.clone();
            item.latest_version_command = default_item.latest_version_command.clone();
            item.update_check_command = default_item.update_check_command.clone();
            item.update_check_regex = default_item.update_check_regex.clone();
            item.update_command = default_item.update_command.clone();
            changed = true;
        }
    }

    if item.id == "codex-cli"
        && item.latest_version_command.as_deref() == Some(OLD_CODEX_LATEST_CMD)
    {
        item.latest_version_command = default_item.latest_version_command.clone();
        changed = true;
    }

    if item.id == "lm-studio" {
        if item.current_version_command.as_deref() == Some(OLD_LMSTUDIO_CURRENT_CMD) {
            item.current_version_command = default_item.current_version_command.clone();
            changed = true;
        } else {
            changed |= set_option_if_missing(
                &mut item.current_version_command,
                &default_item.current_version_command,
            );
        }
        changed |= set_option_if_missing(
            &mut item.latest_version_command,
            &default_item.latest_version_command,
        );
        changed |= set_string_if_empty(&mut item.update_command, &default_item.update_command);
    }

    if item.id == "google-chrome" {
        changed |= set_option_if_missing(
            &mut item.current_version_command,
            &default_item.current_version_command,
        );
        let latest_cmd = item.latest_version_command.as_deref();
        if latest_cmd
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
            || latest_cmd == Some(OLD_CHROME_LATEST_CMD)
        {
            item.latest_version_command = default_item.latest_version_command.clone();
            changed = true;
        }
        changed |= set_string_if_empty(&mut item.update_command, &default_item.update_command);
    }

    if item.id == "ghostty" {
        changed |= set_option_if_missing(
            &mut item.current_version_command,
            &default_item.current_version_command,
        );
        let latest_cmd = item.latest_version_command.as_deref();
        if latest_cmd
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
            || latest_cmd == Some(OLD_GHOSTTY_LATEST_CMD)
            || latest_cmd == Some(PREV_GHOSTTY_LATEST_CMD)
            || latest_cmd == Some(PREV2_GHOSTTY_LATEST_CMD)
            || latest_cmd
                .map(|value| {
                    value.contains("ghostty-org/ghostty/releases")
                        || value.contains("api.github.com/repos/ghostty-org/ghostty/releases")
                })
                .unwrap_or(false)
        {
            item.latest_version_command = default_item.latest_version_command.clone();
            changed = true;
        }
        changed |= set_string_if_empty(&mut item.update_command, &default_item.update_command);
    }

    if item.id == "warp" {
        changed |= set_option_if_missing(
            &mut item.current_version_command,
            &default_item.current_version_command,
        );
        changed |= set_option_if_missing(
            &mut item.latest_version_command,
            &default_item.latest_version_command,
        );
        if item
            .update_check_command
            .as_deref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
        {
            item.update_check_command = default_item.update_check_command.clone();
            item.update_check_regex = default_item.update_check_regex.clone();
            changed = true;
        }
        changed |= set_string_if_empty(&mut item.update_command, &default_item.update_command);
    }

    if item.id == "codexbar" {
        changed |= set_option_if_missing(
            &mut item.current_version_command,
            &default_item.current_version_command,
        );
        let latest_cmd = item.latest_version_command.as_deref();
        if latest_cmd
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
            || latest_cmd
                .map(|value| {
                    !value.contains("git ls-remote --tags --refs https://github.com/steipete/CodexBar.git")
                })
                .unwrap_or(false)
        {
            item.latest_version_command = default_item.latest_version_command.clone();
            changed = true;
        }
        changed |= set_string_if_empty(&mut item.update_command, &default_item.update_command);
    }

    if item.id == "portkiller" {
        changed |= set_option_if_missing(
            &mut item.current_version_command,
            &default_item.current_version_command,
        );
        let latest_cmd = item.latest_version_command.as_deref();
        if latest_cmd
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
            || latest_cmd
                .map(|value| {
                    !value.contains(
                        "git ls-remote --tags --refs https://github.com/productdevbook/port-killer.git",
                    )
                })
                .unwrap_or(false)
        {
            item.latest_version_command = default_item.latest_version_command.clone();
            changed = true;
        }
        changed |= set_string_if_empty(&mut item.update_command, &default_item.update_command);
    }

    if item.id == "rust-toolchain" {
        changed |= set_option_if_missing(
            &mut item.current_version_command,
            &default_item.current_version_command,
        );
        let latest_cmd = item.latest_version_command.as_deref();
        if latest_cmd
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
            || latest_cmd == Some(OLD_RUST_LATEST_CMD)
            || latest_cmd
                .map(|value| !value.contains("channel-rust-stable.toml"))
                .unwrap_or(false)
            || latest_cmd
                .map(|value| {
                    value.contains("channel-rust-stable.toml")
                        && (!value.contains("[pkg\\.rust]") || value.contains("else echo ''; fi"))
                })
                .unwrap_or(false)
        {
            item.latest_version_command = default_item.latest_version_command.clone();
            changed = true;
        }
    }

    changed
}
