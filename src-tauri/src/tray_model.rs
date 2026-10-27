//! What the tray menu should contain, decided without touching the OS. Pure.
//!
//! On macOS and Windows a click opens the app and the menu is a short list of actions. On Linux
//! tray icons cannot report clicks, so the menu also lists every account, grouped by the app it
//! came from. Replacing an open menu closes it, so the live tray only rebuilds when
//! the *structure* changes and otherwise updates item text in place.

use crate::codes::{split_code_in_half, CodeView};
use crate::state::VaultStatus;
use crate::vendor::Vendor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayRow {
    pub account_id: String,
    pub text: String,
}

/// A heading for an app, or one of its accounts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayEntry {
    Header(String),
    Account(TrayRow),
}

/// What the menu is built for, beyond the accounts themselves.
#[derive(Debug, Clone, Copy)]
pub struct TrayOptions {
    pub is_autostart_enabled: bool,
    /// Show each account's code in the menu (otherwise the name only; a click still copies it).
    pub shows_codes: bool,
    /// List the accounts in the menu. Only where the icon cannot be clicked (Linux).
    pub lists_accounts: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayModel {
    pub placeholder: Option<&'static str>,
    pub countdown_text: Option<String>,
    pub entries: Vec<TrayEntry>,
    pub is_unlocked: bool,
    pub is_autostart_enabled: bool,
}

/// Everything that decides which items exist and in what order. While it stays
/// the same, only text changes (codes, countdown), which can be updated in place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuStructure {
    placeholder: Option<&'static str>,
    has_countdown: bool,
    entry_keys: Vec<String>,
    is_unlocked: bool,
    is_autostart_enabled: bool,
}

pub fn placeholder_text(status: VaultStatus, has_no_accounts: bool) -> Option<&'static str> {
    match status {
        VaultStatus::Uninitialized => Some("Open DA21 to create your vault"),
        VaultStatus::Locked => Some("Locked — open DA21 to unlock"),
        VaultStatus::Unlocked if has_no_accounts => Some("No accounts — open DA21 to import"),
        VaultStatus::Unlocked => None,
    }
}

/// Codes that share a period expire together, so one countdown for the soonest
/// expiry replaces a "(17s)" on every row.
fn countdown_text(views: &[CodeView]) -> Option<String> {
    let soonest = views.iter().map(|view| view.seconds_left).min()?;
    Some(format!("Codes refresh in {soonest}s"))
}

/// The full name, never cut off. With codes hidden the row is just the name; a click still copies the code.
fn row_for(view: &CodeView, shows_codes: bool) -> TrayRow {
    let text = if shows_codes { format!("{}   {}", view.label, split_code_in_half(&view.code)) } else { view.label.clone() };
    TrayRow { account_id: view.id.clone(), text }
}

/// Apps A to Z with "Other" last, accounts A to Z inside each, like the window and the panel.
fn group_by_vendor(views: &[CodeView]) -> Vec<(Vendor, Vec<&CodeView>)> {
    let mut groups: Vec<(Vendor, Vec<&CodeView>)> = Vec::new();
    for view in views {
        match groups.iter_mut().find(|(vendor, _)| *vendor == view.vendor) {
            Some((_, members)) => members.push(view),
            None => groups.push((view.vendor, vec![view])),
        }
    }
    for (_, members) in &mut groups {
        members.sort_by_key(|view| view.label.to_lowercase());
    }
    groups.sort_by_key(|(vendor, _)| (*vendor == Vendor::Other, vendor.display_name().to_lowercase()));
    groups
}

fn entries_for(views: &[CodeView], shows_codes: bool) -> Vec<TrayEntry> {
    let mut entries = Vec::new();
    for (vendor, members) in group_by_vendor(views) {
        entries.push(TrayEntry::Header(format!("{} ({})", vendor.display_name(), members.len())));
        entries.extend(members.into_iter().map(|view| TrayEntry::Account(row_for(view, shows_codes))));
    }
    entries
}

pub fn build_tray_model(status: VaultStatus, views: &[CodeView], options: TrayOptions) -> TrayModel {
    let lists = options.lists_accounts;
    TrayModel {
        placeholder: if lists { placeholder_text(status, views.is_empty()) } else { None },
        countdown_text: if lists && options.shows_codes { countdown_text(views) } else { None },
        entries: if lists { entries_for(views, options.shows_codes) } else { Vec::new() },
        is_unlocked: status == VaultStatus::Unlocked,
        is_autostart_enabled: options.is_autostart_enabled,
    }
}

impl TrayModel {
    pub fn structure(&self) -> MenuStructure {
        MenuStructure {
            placeholder: self.placeholder,
            has_countdown: self.countdown_text.is_some(),
            entry_keys: self.entries.iter().map(entry_key).collect(),
            is_unlocked: self.is_unlocked,
            is_autostart_enabled: self.is_autostart_enabled,
        }
    }

    /// The accounts in menu order, which is the order of the live items.
    pub fn account_rows(&self) -> impl Iterator<Item = &TrayRow> {
        self.entries.iter().filter_map(|entry| match entry {
            TrayEntry::Account(row) => Some(row),
            TrayEntry::Header(_) => None,
        })
    }
}

fn entry_key(entry: &TrayEntry) -> String {
    match entry {
        TrayEntry::Header(text) => format!("header:{text}"),
        TrayEntry::Account(row) => format!("account:{}", row.account_id),
    }
}

#[cfg(test)]
mod tests;
