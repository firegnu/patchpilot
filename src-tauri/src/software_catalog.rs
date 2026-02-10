use crate::model::SoftwareItem;

pub fn default_shared_update_commands() -> Vec<String> {
    vec!["brew update".to_string(), "brew upgrade".to_string()]
}

pub fn default_software_items() -> Vec<SoftwareItem> {
    vec![
        SoftwareItem {
            id: "brew".to_string(),
            name: "Homebrew".to_string(),
            kind: "cli".to_string(),
            enabled: true,
            description: "Check and update Homebrew packages".to_string(),
            current_version_command: Some(
                "brew --version | head -n 1 | awk '{print $2}'".to_string(),
            ),
            latest_version_command: Some(
                "brew --version | head -n 1 | awk '{print $2}'".to_string(),
            ),
            update_check_command: Some("brew outdated --quiet".to_string()),
            update_check_regex: Some(".+".to_string()),
            update_command: "brew update && brew upgrade".to_string(),
        },
        SoftwareItem {
            id: "bun".to_string(),
            name: "Bun".to_string(),
            kind: "cli".to_string(),
            enabled: true,
            description: "Check and update Bun (if managed via brew)".to_string(),
            current_version_command: Some(
                "if command -v bun >/dev/null 2>&1; then bun --version; else echo ''; fi"
                    .to_string(),
            ),
            latest_version_command: Some(
                "if brew info bun --json=v2 >/dev/null 2>&1; then brew info bun --json=v2 | sed -nE 's/.*\"stable\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1; elif command -v bun >/dev/null 2>&1; then bun --version; else echo ''; fi"
                    .to_string(),
            ),
            update_check_command: Some(
                "if brew list bun >/dev/null 2>&1; then brew outdated --quiet bun; else echo ''; fi"
                    .to_string(),
            ),
            update_check_regex: Some(".+".to_string()),
            update_command: "if brew list bun >/dev/null 2>&1; then brew upgrade bun; else echo 'bun is not managed by brew'; fi".to_string(),
        },
        SoftwareItem {
            id: "claude-code".to_string(),
            name: "Claude Code".to_string(),
            kind: "cli".to_string(),
            enabled: true,
            description: "Auto-check via GitHub releases; update manually with claude update"
                .to_string(),
            current_version_command: Some(
                "claude --version | sed -E 's/[^0-9]*([0-9]+\\.[0-9]+\\.[0-9]+).*/\\1/'"
                    .to_string(),
            ),
            latest_version_command: Some(
                "curl -fsSLI -o /dev/null -w '%{url_effective}' https://github.com/anthropics/claude-code/releases/latest | sed -E 's#.*/tag/v?##'"
                    .to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "claude update".to_string(),
        },
        SoftwareItem {
            id: "gemini-cli".to_string(),
            name: "Google Gemini CLI".to_string(),
            kind: "cli".to_string(),
            enabled: true,
            description: "Auto-check via npm registry; update manually with npm upgrade".to_string(),
            current_version_command: Some(
                "if command -v gemini >/dev/null 2>&1; then gemini --version | sed -E 's/[^0-9]*([0-9]+\\.[0-9]+\\.[0-9]+).*/\\1/'; else echo ''; fi"
                    .to_string(),
            ),
            latest_version_command: Some("npm view @google/gemini-cli version".to_string()),
            update_check_command: None,
            update_check_regex: None,
            update_command: "npm upgrade -g @google/gemini-cli".to_string(),
        },
        SoftwareItem {
            id: "codex-cli".to_string(),
            name: "Codex CLI".to_string(),
            kind: "cli".to_string(),
            enabled: true,
            description: "Auto-check via Homebrew cask metadata; update manually with brew upgrade --cask"
                .to_string(),
            current_version_command: Some(
                "codex --version | sed -E 's/[^0-9]*([0-9]+\\.[0-9]+\\.[0-9]+).*/\\1/'"
                    .to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask codex --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1"
                    .to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "brew upgrade --cask codex".to_string(),
        },
        SoftwareItem {
            id: "oh-my-zsh".to_string(),
            name: "Oh My Zsh".to_string(),
            kind: "cli".to_string(),
            enabled: true,
            description: "Auto-check git HEAD; update manually via OMZ upgrade script".to_string(),
            current_version_command: Some(
                "if [ -d \"${ZSH:-$HOME/.oh-my-zsh}/.git\" ]; then git -C \"${ZSH:-$HOME/.oh-my-zsh}\" rev-parse --short=12 HEAD; else echo ''; fi"
                    .to_string(),
            ),
            latest_version_command: Some(
                "if [ -d \"${ZSH:-$HOME/.oh-my-zsh}/.git\" ]; then REMOTE=\"$(git -C \"${ZSH:-$HOME/.oh-my-zsh}\" config --get remote.origin.url 2>/dev/null || echo https://github.com/ohmyzsh/ohmyzsh.git)\"; git ls-remote \"$REMOTE\" HEAD 2>/dev/null | awk '{print substr($1,1,12)}' | head -n 1; else echo ''; fi"
                    .to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "if [ -x \"${ZSH:-$HOME/.oh-my-zsh}/tools/upgrade.sh\" ]; then \"${ZSH:-$HOME/.oh-my-zsh}/tools/upgrade.sh\" -v minimal; else echo 'oh-my-zsh not found'; exit 1; fi".to_string(),
        },
        SoftwareItem {
            id: "go-runtime".to_string(),
            name: "Go".to_string(),
            kind: "runtime".to_string(),
            enabled: true,
            description: "Manual check/update for Go runtime (brew-managed)".to_string(),
            current_version_command: Some(
                "if command -v go >/dev/null 2>&1; then go version | sed -nE 's/^go version go([^ ]+).*/\\1/p'; else echo ''; fi"
                    .to_string(),
            ),
            latest_version_command: Some(
                "if HOMEBREW_NO_AUTO_UPDATE=1 brew info go --json=v2 >/dev/null 2>&1; then HOMEBREW_NO_AUTO_UPDATE=1 brew info go --json=v2 | sed -nE 's/.*\"stable\"[[:space:]]*:[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1; else echo ''; fi"
                    .to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "if brew list go >/dev/null 2>&1; then brew upgrade go; else echo 'go is not managed by brew'; exit 1; fi".to_string(),
        },
        SoftwareItem {
            id: "rust-toolchain".to_string(),
            name: "Rust".to_string(),
            kind: "runtime".to_string(),
            enabled: true,
            description: "Manual check/update for Rust toolchain (rustup)".to_string(),
            current_version_command: Some(
                "if command -v rustc >/dev/null 2>&1; then rustc --version | sed -nE 's/^rustc ([^ ]+).*/\\1/p'; else echo ''; fi"
                    .to_string(),
            ),
            latest_version_command: Some(
                "LATEST=\"$(curl -fsSL https://static.rust-lang.org/dist/channel-rust-stable.toml 2>/dev/null | sed -nE '/^\\[pkg\\.rust\\]/,/^\\[/{s/^version = \"([0-9]+\\.[0-9]+\\.[0-9]+).*/\\1/p}' | head -n 1)\"; if [ -n \"$LATEST\" ]; then echo \"$LATEST\"; elif command -v rustup >/dev/null 2>&1; then OUT=\"$(rustup check 2>/dev/null || true)\"; LATEST=\"$(echo \"$OUT\" | sed -nE 's/.*-> *([0-9][0-9A-Za-z.+-]*).*/\\1/p' | head -n 1)\"; if [ -z \"$LATEST\" ]; then LATEST=\"$(echo \"$OUT\" | sed -nE 's/.*: *([0-9][0-9A-Za-z.+-]*).*/\\1/p' | head -n 1)\"; fi; if [ -n \"$LATEST\" ]; then echo \"$LATEST\"; else rustc --version | sed -nE 's/^rustc ([^ ]+).*/\\1/p'; fi; else rustc --version | sed -nE 's/^rustc ([^ ]+).*/\\1/p'; fi"
                    .to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "if command -v rustup >/dev/null 2>&1; then rustup update; else echo 'rustup not found'; exit 1; fi".to_string(),
        },
        SoftwareItem {
            id: "node-lts-nvm".to_string(),
            name: "Node.js LTS (nvm)".to_string(),
            kind: "runtime".to_string(),
            enabled: true,
            description: "Manual check/update for Node LTS via nvm".to_string(),
            current_version_command: Some(
                "TERM=\"${TERM:-xterm-256color}\"; NVM_DIR=\"${NVM_DIR:-$HOME/.nvm}\"; if [ -s \"$NVM_DIR/nvm.sh\" ]; then . \"$NVM_DIR/nvm.sh\" >/dev/null 2>&1; CUR=\"$(nvm current 2>/dev/null)\"; if [ -z \"$CUR\" ] || [ \"$CUR\" = \"none\" ]; then echo ''; else echo \"$CUR\" | sed -E 's/^v//'; fi; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "TERM=\"${TERM:-xterm-256color}\"; NVM_DIR=\"${NVM_DIR:-$HOME/.nvm}\"; if [ -s \"$NVM_DIR/nvm.sh\" ]; then . \"$NVM_DIR/nvm.sh\" >/dev/null 2>&1; nvm ls-remote --lts 2>/dev/null | sed -nE 's/^[[:space:]]*v([0-9]+\\.[0-9]+\\.[0-9]+).*/\\1/p' | tail -n 1; else echo ''; fi".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "NVM_DIR=\"${NVM_DIR:-$HOME/.nvm}\"; if [ -s \"$NVM_DIR/nvm.sh\" ]; then . \"$NVM_DIR/nvm.sh\"; nvm install --lts && nvm alias default 'lts/*'; else echo 'nvm not found'; exit 1; fi".to_string(),
        },
        SoftwareItem {
            id: "visual-studio-code".to_string(),
            name: "Visual Studio Code".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Visual Studio Code.app\" ]; then defaults read \"/Applications/Visual Studio Code.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask visual-studio-code --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'Visual Studio Code update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "antigravity".to_string(),
            name: "Antigravity".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Antigravity.app\" ]; then defaults read \"/Applications/Antigravity.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask antigravity --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'Antigravity update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "lm-studio".to_string(),
            name: "LM Studio".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and official changelog"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/LM Studio.app\" ]; then defaults read \"/Applications/LM Studio.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null | sed -E 's/^([0-9]+\\.[0-9]+\\.[0-9]+).*/\\1/' || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "curl -fsSL https://lmstudio.ai/changelog | tr '\\r' '\\n' | sed -nE 's/.*>v?([0-9]+\\.[0-9]+\\.[0-9]+([+.-][0-9A-Za-z]+)*)<.*/\\1/p' | head -n 1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'LM Studio update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "google-chrome".to_string(),
            name: "Google Chrome".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description:
                "Auto-check app version via local Info.plist and Chrome VersionHistory API"
                    .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Google Chrome.app\" ]; then defaults read \"/Applications/Google Chrome.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "LATEST=\"$(curl -fsSL \"https://versionhistory.googleapis.com/v1/chrome/platforms/mac/channels/stable/versions?page_size=1\" | sed -nE 's/.*\"version\"[[:space:]]*:[[:space:]]*\"([0-9.]+)\".*/\\1/p' | head -n 1)\"; if [ -n \"$LATEST\" ]; then echo \"$LATEST\"; else curl -fsSL \"https://versionhistory.googleapis.com/v1/chrome/platforms/mac_arm64/channels/stable/versions?page_size=1\" | sed -nE 's/.*\"version\"[[:space:]]*:[[:space:]]*\"([0-9.]+)\".*/\\1/p' | head -n 1; fi".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'Google Chrome update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "claude-desktop".to_string(),
            name: "Claude Desktop".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Claude.app\" ]; then defaults read \"/Applications/Claude.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask claude --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command:
                "echo 'Claude Desktop update is managed manually outside PatchPilot'"
                    .to_string(),
        },
        SoftwareItem {
            id: "chatgpt-desktop".to_string(),
            name: "ChatGPT".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/ChatGPT.app\" ]; then defaults read \"/Applications/ChatGPT.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask chatgpt --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'ChatGPT update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "codex-app".to_string(),
            name: "Codex App".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Codex appcast feed"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Codex.app\" ]; then defaults read \"/Applications/Codex.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "set -o pipefail; LATEST=\"$(curl -fsSL https://persistent.oaistatic.com/codex-app-prod/appcast.xml 2>/dev/null | tr '\\r' '\\n' | sed -nE 's/.*sparkle:shortVersionString=\"([^\"]+)\".*/\\1/p' | head -n 1)\"; if [ -n \"$LATEST\" ]; then echo \"$LATEST\"; else defaults read '/Applications/Codex.app/Contents/Info.plist' CFBundleShortVersionString 2>/dev/null || echo ''; fi".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'Codex App update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "codexbar".to_string(),
            name: "CodexBar".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Sparkle appcast"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/CodexBar.app\" ]; then defaults read \"/Applications/CodexBar.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "set -o pipefail; FEED=\"$(defaults read '/Applications/CodexBar.app/Contents/Info.plist' SUFeedURL 2>/dev/null || echo '')\"; LATEST=\"\"; if [ -n \"$FEED\" ]; then LATEST=\"$(curl -fsSL \"$FEED\" 2>/dev/null | tr '\\r\\n' '  ' | sed -nE \"s/.*sparkle:shortVersionString=[\\\"']([^\\\"']+)[\\\"'].*/\\1/p; t; s/.*<sparkle:shortVersionString>([^<]+)<.*/\\1/p\" | head -n 1)\"; fi; if [ -z \"$LATEST\" ]; then LATEST=\"$(git ls-remote --tags --refs https://github.com/steipete/CodexBar.git 2>/dev/null | awk '{print $2}' | sed -E 's#refs/tags/v?##' | grep -E '^[0-9]+\\.[0-9]+\\.[0-9]+([.-][0-9A-Za-z]+)?$' | sort -V | tail -n 1)\"; fi; if [ -n \"$LATEST\" ]; then echo \"$LATEST\"; else echo ''; fi".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'CodexBar update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "portkiller".to_string(),
            name: "PortKiller".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Sparkle appcast"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/PortKiller.app\" ]; then defaults read \"/Applications/PortKiller.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "set -o pipefail; FEED=\"$(defaults read '/Applications/PortKiller.app/Contents/Info.plist' SUFeedURL 2>/dev/null || echo '')\"; LATEST=\"\"; if [ -n \"$FEED\" ]; then LATEST=\"$(curl -fsSL \"$FEED\" 2>/dev/null | tr '\\r\\n' '  ' | sed -nE \"s/.*sparkle:shortVersionString=[\\\"']([^\\\"']+)[\\\"'].*/\\1/p; t; s/.*<sparkle:shortVersionString>([^<]+)<.*/\\1/p\" | head -n 1)\"; fi; if [ -z \"$LATEST\" ]; then LATEST=\"$(git ls-remote --tags --refs https://github.com/productdevbook/port-killer.git 2>/dev/null | awk '{print $2}' | sed -E 's#refs/tags/v?##' | grep -E '^[0-9]+\\.[0-9]+\\.[0-9]+([.-][0-9A-Za-z]+)?$' | sort -V | tail -n 1)\"; fi; if [ -n \"$LATEST\" ]; then echo \"$LATEST\"; else echo ''; fi".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'PortKiller update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "docker-desktop".to_string(),
            name: "Docker".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Docker.app\" ]; then defaults read \"/Applications/Docker.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask docker-desktop --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'Docker update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "openclaw".to_string(),
            name: "OpenClaw".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/OpenClaw.app\" ]; then defaults read \"/Applications/OpenClaw.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask openclaw --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'OpenClaw update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "raycast".to_string(),
            name: "Raycast".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Raycast.app\" ]; then defaults read \"/Applications/Raycast.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask raycast --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'Raycast update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "notion".to_string(),
            name: "Notion".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Notion.app\" ]; then defaults read \"/Applications/Notion.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask notion --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'Notion update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "bruno".to_string(),
            name: "Bruno".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Bruno.app\" ]; then defaults read \"/Applications/Bruno.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask bruno --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'Bruno update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "fork".to_string(),
            name: "Fork".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Fork.app\" ]; then defaults read \"/Applications/Fork.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask fork --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'Fork update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "zed".to_string(),
            name: "Zed".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Zed.app\" ]; then defaults read \"/Applications/Zed.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask zed --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'Zed update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "typora".to_string(),
            name: "Typora".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Typora.app\" ]; then defaults read \"/Applications/Typora.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask typora --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'Typora update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "datagrip".to_string(),
            name: "DataGrip".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/DataGrip.app\" ]; then defaults read \"/Applications/DataGrip.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask datagrip --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'DataGrip update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "telegram".to_string(),
            name: "Telegram".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Telegram.app\" ]; then defaults read \"/Applications/Telegram.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask telegram --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'Telegram update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "ollama".to_string(),
            name: "Ollama".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Ollama.app\" ]; then defaults read \"/Applications/Ollama.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask ollama --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'Ollama update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "ghostty".to_string(),
            name: "Ghostty".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and GitHub releases"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Ghostty.app\" ]; then defaults read \"/Applications/Ghostty.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "set -o pipefail; LATEST=\"\"; if brew info --cask ghostty --json=v2 >/dev/null 2>&1; then LATEST=\"$(brew info --cask ghostty --json=v2 | tr '\\r\\n' ' ' | sed -nE 's/.*\"version\"[[:space:]]*:[[:space:]]*\"([^\",]+).*/\\1/p' | head -n 1)\"; fi; if [ -z \"$LATEST\" ]; then LATEST=\"$(git ls-remote --tags --refs https://github.com/ghostty-org/ghostty.git 2>/dev/null | awk '{print $2}' | sed -E 's#refs/tags/v?##' | grep -E '^[0-9]+\\.[0-9]+\\.[0-9]+([.-][0-9A-Za-z]+)?$' | sort -V | tail -n 1)\"; fi; if [ -n \"$LATEST\" ]; then echo \"$LATEST\"; else exit 1; fi"
                    .to_string(),
            ),
            update_check_command: None,
            update_check_regex: None,
            update_command: "echo 'Ghostty update is managed manually outside PatchPilot'"
                .to_string(),
        },
        SoftwareItem {
            id: "warp".to_string(),
            name: "Warp".to_string(),
            kind: "gui".to_string(),
            enabled: true,
            description: "Auto-check app version via local Info.plist and Homebrew cask metadata"
                .to_string(),
            current_version_command: Some(
                "if [ -d \"/Applications/Warp.app\" ]; then defaults read \"/Applications/Warp.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
            ),
            latest_version_command: Some(
                "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask warp --json=v2 | tr '\\r\\n' ' ' | sed -nE 's/.*\"version\"[[:space:]]*:[[:space:]]*\"([^\",]+).*/\\1/p' | head -n 1".to_string(),
            ),
            update_check_command: Some(
                "set -o pipefail; CURRENT=\"$(if [ -d '/Applications/Warp.app' ]; then defaults read '/Applications/Warp.app/Contents/Info.plist' CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi)\"; LATEST=\"$(HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask warp --json=v2 | tr '\\r\\n' ' ' | sed -nE 's/.*\"version\"[[:space:]]*:[[:space:]]*\"([^\",]+).*/\\1/p' | head -n 1)\"; CN=\"$(echo \"$CURRENT\" | sed -E 's/^([0-9]+(\\.[0-9]+){2,}).*/\\1/')\"; LN=\"$(echo \"$LATEST\" | sed -E 's/^([0-9]+(\\.[0-9]+){2,}).*/\\1/')\"; if [ -z \"$CN\" ] || [ -z \"$LN\" ]; then echo 'false'; exit 0; fi; MAX=\"$(printf '%s\\n%s\\n' \"$CN\" \"$LN\" | sort -V | tail -n 1)\"; if [ \"$MAX\" = \"$LN\" ] && [ \"$CN\" != \"$LN\" ]; then echo 'true'; else echo 'false'; fi".to_string(),
            ),
            update_check_regex: None,
            update_command: "echo 'Warp update is managed manually outside PatchPilot'".to_string(),
        },
    ]
}
