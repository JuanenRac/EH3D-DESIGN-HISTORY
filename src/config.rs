// SPDX-License-Identifier: GPL-3.0-or-later
//! The settings of the library. Every tuned value is a line of the settings file; a missing line keeps the conservative default.

use crate::error::HistoryError;

/// The settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The most bytes a history file may have.
    pub max_file_bytes: u64,
    /// The most operations a history may hold.
    pub max_entries: usize,
    /// The kind of operation that deletes an object (a reference to the object after it is reported).
    pub delete_kind: String,
    /// The kind of operation that creates an object (it makes a deleted identity valid again).
    pub create_kind: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            max_file_bytes: 64 * 1024 * 1024,
            max_entries: 1_000_000,
            delete_kind: "object-deleted".into(),
            create_kind: "object-created".into(),
        }
    }
}

fn kind_name(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

impl Config {
    /// Parse a settings file (`key = value` lines, `#` comments).
    pub fn parse(text: &str) -> Result<Config, HistoryError> {
        let mut config = Config::default();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let raw = raw.trim();
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let (key, value) = raw
                .split_once('=')
                .ok_or(HistoryError::ConfigSyntax { line })?;
            let (key, value) = (key.trim(), value.trim());
            let bad = || HistoryError::ConfigBadValue {
                line,
                key: key.to_string(),
            };
            match key {
                "max_file_bytes" => config.max_file_bytes = value.parse().map_err(|_| bad())?,
                "max_entries" => {
                    let n: usize = value.parse().map_err(|_| bad())?;
                    if n == 0 {
                        return Err(bad());
                    }
                    config.max_entries = n;
                }
                "delete_kind" => {
                    if !kind_name(value) {
                        return Err(bad());
                    }
                    config.delete_kind = value.to_string();
                }
                "create_kind" => {
                    if !kind_name(value) {
                        return Err(bad());
                    }
                    config.create_kind = value.to_string();
                }
                _ => {
                    return Err(HistoryError::ConfigUnknownKey {
                        line,
                        key: key.to_string(),
                    })
                }
            }
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_name_the_kinds_of_creation_and_deletion() {
        let c = Config::parse("").unwrap();
        assert_eq!(c.delete_kind, "object-deleted");
        assert_eq!(c.create_kind, "object-created");
    }

    #[test]
    fn lines_change_values_and_bad_lines_are_refused() {
        let c = Config::parse("# c\ndelete_kind = part-removed\nmax_entries = 10\n").unwrap();
        assert_eq!(
            (c.delete_kind.as_str(), c.max_entries),
            ("part-removed", 10)
        );
        assert!(matches!(
            Config::parse("x\n"),
            Err(HistoryError::ConfigSyntax { line: 1 })
        ));
        assert!(matches!(
            Config::parse("a = 1\n"),
            Err(HistoryError::ConfigUnknownKey { line: 1, .. })
        ));
        for bad in [
            "delete_kind = Bad Kind\n",
            "create_kind =\n",
            "max_entries = 0\n",
        ] {
            assert!(
                matches!(
                    Config::parse(bad),
                    Err(HistoryError::ConfigBadValue { line: 1, .. })
                ),
                "{bad:?}"
            );
        }
    }
}
