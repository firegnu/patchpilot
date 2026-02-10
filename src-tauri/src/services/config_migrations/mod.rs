use std::collections::HashMap;

use crate::model::{AppConfig, SoftwareItem};

mod item_patch;

const OLD_DEFAULT_CHECK_INTERVAL_MINUTES: u64 = 360;
const NEW_DEFAULT_CHECK_INTERVAL_MINUTES: u64 = 480;
const REMOVED_ITEM_IDS: [&str; 3] = ["chatgpt-atlas", "pencil", "codexskillmanager"];

fn append_default_items_if_missing(config: &mut AppConfig, default_items: &[SoftwareItem]) -> bool {
    let mut changed = false;
    for default_item in default_items {
        if config.items.iter().any(|item| item.id == default_item.id) {
            continue;
        }
        config.items.push(default_item.clone());
        changed = true;
    }
    changed
}

fn remove_disabled_items(config: &mut AppConfig) -> bool {
    let before = config.items.len();
    config
        .items
        .retain(|item| !REMOVED_ITEM_IDS.contains(&item.id.as_str()));
    config.items.len() != before
}

pub fn patch_legacy_config(config: &mut AppConfig) -> bool {
    let default_items = AppConfig::default().items;
    let default_item_map: HashMap<String, SoftwareItem> = default_items
        .iter()
        .cloned()
        .map(|item| (item.id.clone(), item))
        .collect();

    let mut changed = false;
    changed |= remove_disabled_items(config);

    if config.check_interval_minutes == OLD_DEFAULT_CHECK_INTERVAL_MINUTES {
        config.check_interval_minutes = NEW_DEFAULT_CHECK_INTERVAL_MINUTES;
        changed = true;
    }

    for item in &mut config.items {
        changed |= item_patch::patch_legacy_item_commands(item, &default_item_map);
    }

    changed |= append_default_items_if_missing(config, &default_items);
    changed
}
