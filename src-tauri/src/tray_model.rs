//! What the tray menu should contain, decided without touching the OS. Pure.
//!
//! Replacing an open menu closes it, so the live tray only rebuilds when the
//! *structure* changes and otherwise updates item text in place.

use crate::codes::{split_code_in_half, CodeView};
use crate::state::VaultStatus;

/// A long menu is unreadable; the rest are one click away in the window.
pub const MAX_TRAY_ACCOUNTS: usize = 8;
const MAX_LABEL_CHARS: usize = 26;
const ELLIPSIS: char = '…';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayRow {
    pub account_id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayModel {
    pub placeholder: Option<&'static str>,
    pub countdown_text: Option<String>,
    pub rows: Vec<TrayRow>,
    pub hidden_accounts: usize,
    pub is_autostart_enabled: bool,
}

/// Everything that decides which items exist and in what order. While it stays
/// the same, only text changes (codes, countdown), which can be updated in place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuStructure {
    placeholder: Option<&'static str>,
    has_countdown: bool,
    row_ids: Vec<String>,
    hidden_accounts: usize,
    is_autostart_enabled: bool,
}

/// Cuts a label to `max_chars` characters (not bytes), ending in an ellipsis.
pub fn truncate_label(label: &str, max_chars: usize) -> String {
    if label.chars().count() <= max_chars {
        return label.to_string();
    }
    let kept: String = label.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{}{ELLIPSIS}", kept.trim_end())
}

pub fn more_accounts_text(hidden_accounts: usize) -> String {
    format!("…and {hidden_accounts} more — open DeskAuth21")
}

pub fn placeholder_text(status: VaultStatus, has_no_accounts: bool) -> Option<&'static str> {
    match status {
        VaultStatus::Uninitialized => Some("Open DeskAuth21 to create your vault"),
        VaultStatus::Locked => Some("Locked — open DeskAuth21 to unlock"),
        VaultStatus::Unlocked if has_no_accounts => Some("No accounts — open DeskAuth21 to import"),
        VaultStatus::Unlocked => None,
    }
}

/// Codes that share a period expire together, so one countdown for the soonest
/// expiry replaces a "(17s)" on every row.
fn countdown_text(views: &[CodeView]) -> Option<String> {
    let soonest = views.iter().map(|view| view.seconds_left).min()?;
    Some(format!("Codes refresh in {soonest}s"))
}

fn row_for(view: &CodeView) -> TrayRow {
    let label = truncate_label(&view.label, MAX_LABEL_CHARS);
    TrayRow { account_id: view.id.clone(), text: format!("{label}   {}", split_code_in_half(&view.code)) }
}

pub fn build_tray_model(status: VaultStatus, views: &[CodeView], is_autostart_enabled: bool) -> TrayModel {
    TrayModel {
        placeholder: placeholder_text(status, views.is_empty()),
        countdown_text: countdown_text(views),
        rows: views.iter().take(MAX_TRAY_ACCOUNTS).map(row_for).collect(),
        hidden_accounts: views.len().saturating_sub(MAX_TRAY_ACCOUNTS),
        is_autostart_enabled,
    }
}

impl TrayModel {
    pub fn structure(&self) -> MenuStructure {
        MenuStructure {
            placeholder: self.placeholder,
            has_countdown: self.countdown_text.is_some(),
            row_ids: self.rows.iter().map(|row| row.account_id.clone()).collect(),
            hidden_accounts: self.hidden_accounts,
            is_autostart_enabled: self.is_autostart_enabled,
        }
    }
}

#[cfg(test)]
mod tests;
