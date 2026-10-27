//! Deciding what to do when the user points the app at a different vault folder. Pure.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelocationPlan {
    /// A vault already exists there (for example made on another device): use it.
    /// The current vault is left untouched where it is.
    UseExisting,
    /// Nothing is there yet: copy the current vault in, then switch.
    MoveThenSwitch,
    /// No vault anywhere: just remember the folder for when one is created.
    SwitchOnly,
}

/// An existing vault at the destination always wins and is never overwritten,
/// so choosing a folder can never destroy accounts.
pub fn plan_relocation(destination_has_vault: bool, current_has_vault: bool) -> RelocationPlan {
    match (destination_has_vault, current_has_vault) {
        (true, _) => RelocationPlan::UseExisting,
        (false, true) => RelocationPlan::MoveThenSwitch,
        (false, false) => RelocationPlan::SwitchOnly,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_existing_destination_vault_is_used_and_never_overwritten() {
        assert_eq!(plan_relocation(true, true), RelocationPlan::UseExisting);
        assert_eq!(plan_relocation(true, false), RelocationPlan::UseExisting);
    }

    #[test]
    fn the_current_vault_moves_into_an_empty_folder() {
        assert_eq!(plan_relocation(false, true), RelocationPlan::MoveThenSwitch);
    }

    #[test]
    fn with_no_vault_anywhere_only_the_folder_is_remembered() {
        assert_eq!(plan_relocation(false, false), RelocationPlan::SwitchOnly);
    }
}
