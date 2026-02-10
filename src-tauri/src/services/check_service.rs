use chrono::Utc;
use regex::Regex;

use crate::model::{CheckResult, CommandOutput, SoftwareItem};

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn command_error_text(stderr: &str, stdout: &str) -> String {
    if !stderr.trim().is_empty() {
        return stderr.trim().to_string();
    }
    if !stdout.trim().is_empty() {
        return format!("stdout: {}", stdout.trim());
    }
    "no output".to_string()
}

type CommandExecutor<'a> = dyn FnMut(&str) -> Result<CommandOutput, String> + 'a;

fn check_with_versions(
    item: &SoftwareItem,
    execute: &mut CommandExecutor<'_>,
) -> Result<CheckResult, String> {
    let current_cmd = item
        .current_version_command
        .as_deref()
        .ok_or_else(|| format!("{} has no current_version_command", item.id))?;
    let latest_cmd = item
        .latest_version_command
        .as_deref()
        .ok_or_else(|| format!("{} has no latest_version_command", item.id))?;

    let current_output = execute(current_cmd)?;
    let latest_output = execute(latest_cmd)?;
    if current_output.exit_code != 0 {
        return Err(format!(
            "current_version_command failed (exit {}): {}",
            current_output.exit_code,
            command_error_text(&current_output.stderr, &current_output.stdout)
        ));
    }
    if latest_output.exit_code != 0 {
        return Err(format!(
            "latest_version_command failed (exit {}): {}",
            latest_output.exit_code,
            command_error_text(&latest_output.stderr, &latest_output.stdout)
        ));
    }

    let current = current_output.stdout.trim().to_string();
    let latest = latest_output.stdout.trim().to_string();
    let has_update = !current.is_empty() && !latest.is_empty() && current != latest;

    Ok(CheckResult {
        item_id: item.id.clone(),
        checked_at: now_rfc3339(),
        has_update,
        current_version: Some(current),
        latest_version: Some(latest),
        details: "version comparison".to_string(),
        error: None,
    })
}

fn check_with_command(
    item: &SoftwareItem,
    execute: &mut CommandExecutor<'_>,
) -> Result<CheckResult, String> {
    let check_cmd = item
        .update_check_command
        .as_deref()
        .ok_or_else(|| format!("{} has no update_check_command", item.id))?;
    let output = execute(check_cmd)?;
    if output.exit_code != 0 {
        return Err(format!(
            "update_check_command failed (exit {}): {}",
            output.exit_code,
            command_error_text(&output.stderr, &output.stdout)
        ));
    }

    let has_update = match &item.update_check_regex {
        Some(pattern) => Regex::new(pattern)
            .map_err(|error| format!("invalid update_check_regex for {}: {error}", item.id))?
            .is_match(output.stdout.trim()),
        None => {
            let value = output.stdout.trim().to_ascii_lowercase();
            value == "1" || value == "true" || value == "yes"
        }
    };

    let current_version = if let Some(command) = &item.current_version_command {
        let version_output = execute(command)?;
        if version_output.exit_code == 0 {
            Some(version_output.stdout.trim().to_string())
        } else {
            None
        }
    } else {
        None
    };

    Ok(CheckResult {
        item_id: item.id.clone(),
        checked_at: now_rfc3339(),
        has_update,
        current_version,
        latest_version: None,
        details: format!("check command output: {}", output.stdout),
        error: None,
    })
}

pub fn check_single_item(item: &SoftwareItem, execute: &mut CommandExecutor<'_>) -> CheckResult {
    let result = if item.latest_version_command.is_some() {
        check_with_versions(item, execute)
    } else {
        check_with_command(item, execute)
    };

    match result {
        Ok(value) => value,
        Err(error) => CheckResult {
            item_id: item.id.clone(),
            checked_at: now_rfc3339(),
            has_update: false,
            current_version: None,
            latest_version: None,
            details: "check failed".to_string(),
            error: Some(error),
        },
    }
}
