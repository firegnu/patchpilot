use std::collections::HashMap;
use std::thread;

use crate::model::SoftwareItem;
use crate::services::shell_runner;

const DETECT_TIMEOUT_SECONDS: u64 = 10;

fn is_installed(item: &SoftwareItem) -> bool {
    let cmd = match &item.current_version_command {
        Some(cmd) => cmd,
        None => return true,
    };
    match shell_runner::run_shell_command(cmd, DETECT_TIMEOUT_SECONDS) {
        Ok(output) => output.exit_code == 0 && !output.stdout.trim().is_empty(),
        Err(_) => false,
    }
}

pub fn detect_all(items: &[SoftwareItem]) -> HashMap<String, bool> {
    thread::scope(|scope| {
        let handles: Vec<_> = items
            .iter()
            .map(|item| {
                let id = item.id.clone();
                scope.spawn(move || (id, is_installed(item)))
            })
            .collect();

        handles
            .into_iter()
            .filter_map(|handle| handle.join().ok())
            .collect()
    })
}
