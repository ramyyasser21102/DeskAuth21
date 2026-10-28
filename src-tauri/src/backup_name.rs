//! The suggested file name for a backup. Pure: the date comes from the caller.

const SECONDS_PER_DAY: u64 = 86_400;
/// Days from 0000-03-01 to 1970-01-01, the shift that puts leap days at the end of a year.
const DAYS_TO_UNIX_EPOCH: u64 = 719_468;
const DAYS_PER_ERA: u64 = 146_097;
const DAYS_PER_FOUR_YEARS: u64 = 1_460;
const DAYS_PER_CENTURY: u64 = 36_524;
const DAYS_PER_YEAR: u64 = 365;
const YEARS_PER_ERA: u64 = 400;
const MONTH_STEP_NUMERATOR: u64 = 5;
const MONTH_STEP_OFFSET: u64 = 2;
const MONTH_STEP_DENOMINATOR: u64 = 153;
const MONTHS_PER_YEAR: u64 = 12;
const MARCH_BASED_JANUARY: u64 = 10;
const MONTHS_FROM_MARCH_TO_DECEMBER: u64 = 9;

/// The extension is not the vault's own, so syncing never mistakes a backup for a conflicted copy.
pub const BACKUP_EXTENSION: &str = "backup";

/// "DA21-backup-2026-10-28.backup" for the day `unix_seconds` falls on (UTC).
pub fn backup_file_name(unix_seconds: u64) -> String {
    let (year, month, day) = civil_date(unix_seconds / SECONDS_PER_DAY);
    format!("DA21-backup-{year:04}-{month:02}-{day:02}.{BACKUP_EXTENSION}")
}

/// Howard Hinnant's days-to-calendar-date algorithm, for days since 1970-01-01.
fn civil_date(days_since_epoch: u64) -> (u64, u64, u64) {
    let days = days_since_epoch + DAYS_TO_UNIX_EPOCH;
    let era = days / DAYS_PER_ERA;
    let day_of_era = days % DAYS_PER_ERA;
    let year_of_era = (day_of_era - day_of_era / DAYS_PER_FOUR_YEARS + day_of_era / DAYS_PER_CENTURY - day_of_era / (DAYS_PER_ERA - 1)) / DAYS_PER_YEAR;
    let day_of_year = day_of_era - (DAYS_PER_YEAR * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (MONTH_STEP_NUMERATOR * day_of_year + MONTH_STEP_OFFSET) / MONTH_STEP_DENOMINATOR;
    let day = day_of_year - (MONTH_STEP_DENOMINATOR * month_index + MONTH_STEP_OFFSET) / MONTH_STEP_NUMERATOR + 1;
    let month = if month_index < MARCH_BASED_JANUARY { month_index + 3 } else { month_index - MONTHS_FROM_MARCH_TO_DECEMBER };
    let year = year_of_era + era * YEARS_PER_ERA + u64::from(month <= 2);
    debug_assert!(month <= MONTHS_PER_YEAR);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_backup_after_the_day() {
        assert_eq!(backup_file_name(0), "DA21-backup-1970-01-01.backup");
        assert_eq!(backup_file_name(1_782_864_000), "DA21-backup-2026-07-01.backup");
    }

    #[test]
    fn handles_leap_days_and_year_ends() {
        assert_eq!(backup_file_name(951_782_400), "DA21-backup-2000-02-29.backup");
        assert_eq!(backup_file_name(1_735_689_599), "DA21-backup-2024-12-31.backup");
        assert_eq!(backup_file_name(1_735_689_600), "DA21-backup-2025-01-01.backup");
    }

    #[test]
    fn the_extension_is_not_the_vaults_so_sync_ignores_backups() {
        assert!(!backup_file_name(0).ends_with(".da21"));
    }
}
