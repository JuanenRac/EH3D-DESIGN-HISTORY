// SPDX-License-Identifier: GPL-3.0-or-later
//! The history: operations chained by hash, named states, the checks and the comparison of two histories.

use crate::config::Config;
use crate::error::HistoryError;
use crate::Code;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

/// One operation of the design.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// 1 for the first operation, then up by one.
    pub seq: u64,
    /// The hash of the previous operation (empty for the first).
    pub parent: String,
    /// What happened, such as `component-moved` or `net-modified`.
    pub kind: String,
    /// The stable identity of the object it concerns. Renaming an object does not change it.
    pub object: String,
    /// A sentence for a person.
    #[serde(default)]
    pub summary: String,
    /// The value before, as text.
    #[serde(default)]
    pub before: Option<String>,
    /// The value after, as text.
    #[serde(default)]
    pub after: Option<String>,
    /// The SHA-256 of the content above (lower-case hexadecimal).
    pub hash: String,
}

impl Entry {
    fn compute_hash(
        seq: u64,
        parent: &str,
        kind: &str,
        object: &str,
        summary: &str,
        before: &Option<String>,
        after: &Option<String>,
    ) -> String {
        let text =
            serde_json::json!([seq, parent, kind, object, summary, before, after]).to_string();
        Sha256::digest(text.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// The hash this entry should have given its content.
    pub fn expected_hash(&self) -> String {
        Entry::compute_hash(
            self.seq,
            &self.parent,
            &self.kind,
            &self.object,
            &self.summary,
            &self.before,
            &self.after,
        )
    }
}

/// Something wrong in a history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The code.
    pub code: Code,
    /// The number of the operation it is about (0 for the history as a whole or a state).
    pub seq: u64,
    /// What is wrong, in English.
    pub message: String,
}

/// Where two histories separate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divergence {
    /// How many operations they share from the start (same content, same hash).
    pub common: usize,
    /// What only the first one did afterwards.
    pub only_a: Vec<Entry>,
    /// What only the second one did afterwards.
    pub only_b: Vec<Entry>,
}

/// The history.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct History {
    /// The operations, oldest first.
    #[serde(default)]
    pub entries: Vec<Entry>,
    /// Named states: the name, and the hash of the operation the design was at.
    #[serde(default)]
    pub states: BTreeMap<String, String>,
}

impl History {
    /// Read a history from JSON, within the bounds of the configuration.
    pub fn from_json(text: &str, config: &Config) -> Result<History, HistoryError> {
        if text.len() as u64 > config.max_file_bytes {
            return Err(HistoryError::TooLarge {
                what: "history bytes",
            });
        }
        let history: History =
            serde_json::from_str(text).map_err(|e| HistoryError::BadJson(e.to_string()))?;
        if history.entries.len() > config.max_entries {
            return Err(HistoryError::TooLarge { what: "operations" });
        }
        Ok(history)
    }

    /// The history as JSON: the same content always gives the same bytes.
    pub fn to_json(&self) -> Result<String, HistoryError> {
        serde_json::to_string_pretty(self).map_err(|e| HistoryError::BadJson(e.to_string()))
    }

    /// Add an operation at the end, numbered and chained. The kind and the object must not be empty.
    pub fn append(
        &mut self,
        kind: &str,
        object: &str,
        summary: &str,
        before: Option<&str>,
        after: Option<&str>,
    ) -> Result<&Entry, HistoryError> {
        if kind.trim().is_empty() || object.trim().is_empty() {
            return Err(HistoryError::BadEntry);
        }
        let seq = self.entries.len() as u64 + 1;
        let parent = self
            .entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_default();
        let (before, after) = (before.map(str::to_string), after.map(str::to_string));
        let hash = Entry::compute_hash(seq, &parent, kind, object, summary, &before, &after);
        self.entries.push(Entry {
            seq,
            parent,
            kind: kind.to_string(),
            object: object.to_string(),
            summary: summary.to_string(),
            before,
            after,
            hash,
        });
        Ok(&self.entries[self.entries.len() - 1])
    }

    /// Name the state the design is at after the operation with this hash.
    pub fn name_state(&mut self, name: &str, hash: &str) {
        self.states.insert(name.to_string(), hash.to_string());
    }

    /// What is wrong, in order of operation.
    pub fn check(&self, config: &Config) -> Vec<Finding> {
        let mut out = Vec::new();
        let mut previous = String::new();
        let mut deleted: BTreeSet<&str> = BTreeSet::new();
        let mut hashes = BTreeSet::new();
        for (index, e) in self.entries.iter().enumerate() {
            let at = |code, message: String| Finding {
                code,
                seq: e.seq,
                message,
            };
            if e.seq != index as u64 + 1 {
                out.push(at(
                    Code::BrokenChain,
                    format!("the number should be {}", index + 1),
                ));
            }
            if e.parent != previous {
                out.push(at(
                    Code::BrokenChain,
                    "does not follow the previous operation".into(),
                ));
            }
            if e.hash != e.expected_hash() {
                out.push(at(
                    Code::BrokenChain,
                    "the content does not match its hash".into(),
                ));
            }
            if e.kind.trim().is_empty() || e.object.trim().is_empty() {
                out.push(at(
                    Code::BadEntry,
                    "an operation needs a kind and an object".into(),
                ));
            }
            if e.kind == config.create_kind {
                deleted.remove(e.object.as_str());
            } else if deleted.contains(e.object.as_str()) {
                out.push(at(
                    Code::DeletedReference,
                    format!(
                        "refers to `{}`, which an earlier operation deleted",
                        e.object
                    ),
                ));
            }
            if e.kind == config.delete_kind {
                deleted.insert(e.object.as_str());
            }
            hashes.insert(e.hash.as_str());
            previous = e.hash.clone();
        }
        for (name, hash) in &self.states {
            if !hashes.contains(hash.as_str()) {
                out.push(Finding {
                    code: Code::UnknownState,
                    seq: 0,
                    message: format!(
                        "the state `{name}` points to an operation that is not in the history"
                    ),
                });
            }
        }
        out
    }

    /// Where this history and another separate: the operations they share from the start and what each did afterwards.
    pub fn compare(&self, other: &History) -> Divergence {
        let common = self
            .entries
            .iter()
            .zip(&other.entries)
            .take_while(|(a, b)| a.hash == b.hash && a == b)
            .count();
        Divergence {
            common,
            only_a: self.entries[common..].to_vec(),
            only_b: other.entries[common..].to_vec(),
        }
    }

    /// The first operation after which `is_bad` holds, found by halving (it assumes that once the design is bad it stays bad). `is_bad` is given the operations
    /// up to and including the one being tried. `None` when the whole history is good.
    pub fn first_bad<F: Fn(&[Entry]) -> bool>(&self, is_bad: F) -> Option<&Entry> {
        if self.entries.is_empty() || !is_bad(&self.entries) {
            return None;
        }
        let (mut low, mut high) = (0usize, self.entries.len());
        // invariant: the prefix of `high` operations is bad and the prefix of `low` operations is not
        while high - low > 1 {
            let middle = low + (high - low) / 2;
            if is_bad(&self.entries[..middle]) {
                high = middle;
            } else {
                low = middle;
            }
        }
        Some(&self.entries[high - 1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> History {
        let mut h = History::default();
        h.append("object-created", "c1", "added R1", None, Some("10k"))
            .unwrap();
        h.append(
            "component-moved",
            "c1",
            "moved R1",
            Some("0,0"),
            Some("5,5"),
        )
        .unwrap();
        h.append("net-modified", "n1", "joined n1", None, None)
            .unwrap();
        h
    }

    fn codes(h: &History) -> Vec<Code> {
        h.check(&Config::default()).iter().map(|f| f.code).collect()
    }

    #[test]
    fn appended_operations_are_numbered_chained_and_clean() {
        let h = sample();
        assert_eq!(
            h.entries.iter().map(|e| e.seq).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(h.entries[1].parent, h.entries[0].hash);
        assert_eq!(h.entries[0].hash.len(), 64);
        assert!(codes(&h).is_empty());
    }

    #[test]
    fn an_operation_needs_a_kind_and_an_object() {
        let mut h = History::default();
        assert!(matches!(
            h.append("", "c1", "", None, None),
            Err(HistoryError::BadEntry)
        ));
        assert!(matches!(
            h.append("moved", "  ", "", None, None),
            Err(HistoryError::BadEntry)
        ));
        assert!(h.entries.is_empty());
    }

    #[test]
    fn a_change_in_the_past_breaks_the_chain() {
        let mut h = sample();
        h.entries[1].after = Some("9,9".into());
        let found = h.check(&Config::default());
        assert!(found
            .iter()
            .any(|f| f.code == Code::BrokenChain && f.seq == 2));
        // a removed operation breaks the numbering and the link
        let mut g = sample();
        g.entries.remove(1);
        assert!(codes(&g).contains(&Code::BrokenChain));
    }

    #[test]
    fn a_state_must_point_to_an_operation_of_the_history() {
        let mut h = sample();
        let first = h.entries[0].hash.clone();
        h.name_state("release", &first);
        assert!(codes(&h).is_empty());
        h.name_state("lost", &"0".repeat(64));
        assert_eq!(codes(&h), vec![Code::UnknownState]);
    }

    #[test]
    fn a_reference_to_a_deleted_object_is_reported_and_creating_it_again_clears_it() {
        let mut h = sample();
        h.append("object-deleted", "c1", "removed R1", None, None)
            .unwrap();
        h.append("component-moved", "c1", "moved a ghost", None, None)
            .unwrap();
        assert_eq!(codes(&h), vec![Code::DeletedReference]);
        h.append("object-created", "c1", "added R1 again", None, None)
            .unwrap();
        h.append("component-moved", "c1", "moved it", None, None)
            .unwrap();
        assert_eq!(codes(&h), vec![Code::DeletedReference]);
        // the names of the two kinds are settings
        let other = Config {
            delete_kind: "part-removed".into(),
            ..Config::default()
        };
        assert!(h.check(&other).is_empty());
    }

    #[test]
    fn two_histories_are_compared_from_where_they_separate() {
        let a = sample();
        let mut b = sample();
        b.append(
            "rule-altered",
            "r1",
            "wider clearance",
            Some("0.2"),
            Some("0.3"),
        )
        .unwrap();
        let d = a.compare(&b);
        assert_eq!((d.common, d.only_a.len(), d.only_b.len()), (3, 0, 1));
        assert_eq!(d.only_b[0].kind, "rule-altered");
        let mut c = History::default();
        c.append("object-created", "c1", "added R1", None, Some("10k"))
            .unwrap();
        c.append(
            "component-moved",
            "c1",
            "moved R1",
            Some("0,0"),
            Some("7,7"),
        )
        .unwrap();
        let d = a.compare(&c);
        assert_eq!((d.common, d.only_a.len(), d.only_b.len()), (1, 2, 1));
        assert_eq!(a.compare(&a).common, 3);
    }

    #[test]
    fn the_first_bad_operation_is_found_by_halving() {
        let mut h = History::default();
        for i in 0..20 {
            let kind = if i == 13 {
                "regression"
            } else {
                "component-moved"
            };
            h.append(kind, "c1", "", None, None).unwrap();
        }
        let found = h.first_bad(|prefix| prefix.iter().any(|e| e.kind == "regression"));
        assert_eq!(found.map(|e| e.seq), Some(14));
        assert!(h.first_bad(|_| false).is_none());
        assert!(History::default().first_bad(|_| true).is_none());
        assert_eq!(h.first_bad(|p| !p.is_empty()).map(|e| e.seq), Some(1));
    }

    #[test]
    fn the_same_content_gives_the_same_bytes_and_reads_back() {
        let h = sample();
        let text = h.to_json().unwrap();
        assert_eq!(text, sample().to_json().unwrap());
        assert_eq!(History::from_json(&text, &Config::default()).unwrap(), h);
    }

    #[test]
    fn bad_json_unknown_fields_and_the_size_limits_are_refused() {
        let config = Config::default();
        for text in ["", "[]", r#"{"entries":3}"#, r#"{"extra":1}"#] {
            assert!(
                matches!(
                    History::from_json(text, &config),
                    Err(HistoryError::BadJson(_))
                ),
                "{text:?}"
            );
        }
        let text = sample().to_json().unwrap();
        let small = Config {
            max_file_bytes: 10,
            ..Config::default()
        };
        assert!(matches!(
            History::from_json(&text, &small),
            Err(HistoryError::TooLarge { .. })
        ));
        let few = Config {
            max_entries: 2,
            ..Config::default()
        };
        assert!(matches!(
            History::from_json(&text, &few),
            Err(HistoryError::TooLarge { .. })
        ));
    }
}
