//! Changing which folder the vault lives in, so it can sit in a synced folder.

use crate::error::CommandError;
use crate::persistence::{read_vault_file, write_atomically};
use crate::settings_store::save_settings;
use crate::state::AppState;
use crate::vault_relocation::{plan_relocation, RelocationPlan};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelocationOutcome {
    /// The chosen folder is where the vault already is.
    Unchanged,
    /// The current vault was copied into the folder, which is now in use.
    Moved,
    /// A vault was already in the folder and is now in use; the vault is locked.
    UsedExisting,
    /// There was no vault anywhere; the folder is remembered for when one is created.
    Selected,
}

pub fn change_vault_folder(state: &AppState, folder: &Path) -> Result<RelocationOutcome, CommandError> {
    let folder_text = folder
        .to_str()
        .ok_or_else(|| CommandError::InvalidSetting("a folder whose name is not valid text".into()))?;
    let current_path = state.vault_path();
    let new_path = state.vault_path_in(folder_text);
    if new_path == current_path {
        return Ok(RelocationOutcome::Unchanged);
    }
    let plan = plan_relocation(new_path.exists(), current_path.exists());
    if plan == RelocationPlan::MoveThenSwitch {
        copy_vault_file(&current_path, &new_path)?;
    }
    remember_folder(state, folder_text)?;
    Ok(finish_relocation(state, plan, &current_path))
}

/// Copies and then reads back, because a half-copied vault in a sync folder
/// would replicate everywhere before anyone noticed.
fn copy_vault_file(from: &Path, to: &Path) -> Result<(), CommandError> {
    let bytes = read_vault_file(from)?;
    write_atomically(to, &bytes)?;
    if std::fs::read(to)? != bytes {
        return Err(std::io::Error::other("the copied vault did not match the original").into());
    }
    Ok(())
}

fn remember_folder(state: &AppState, folder: &str) -> Result<(), CommandError> {
    let mut settings = state.settings();
    settings.vault_folder = Some(folder.to_string());
    save_settings(&state.settings_path(), &settings)?;
    state.replace_settings(settings);
    Ok(())
}

fn finish_relocation(state: &AppState, plan: RelocationPlan, old_path: &Path) -> RelocationOutcome {
    match plan {
        RelocationPlan::MoveThenSwitch => {
            remove_old_copy(old_path);
            RelocationOutcome::Moved
        }
        RelocationPlan::UseExisting => {
            state.vault_guard().relock();
            RelocationOutcome::UsedExisting
        }
        RelocationPlan::SwitchOnly => {
            state.vault_guard().relock();
            RelocationOutcome::Selected
        }
    }
}

fn remove_old_copy(old_path: &Path) {
    if let Err(error) = std::fs::remove_file(old_path) {
        eprintln!("moved the vault but could not remove the old copy: {error}");
    }
}

#[cfg(test)]
mod tests;
