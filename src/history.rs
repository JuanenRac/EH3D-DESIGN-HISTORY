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
    /// The place of the operation in the list (0 is the first), whatever number it carries; `None` for the history as a whole or a state.
    pub index: Option<usize>,
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
    /// The names of the states that exist in only one of the two or point to a different operation, sorted.
    pub states_differ: Vec<String>,
}

/// The history.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct History {
    /// The operations, oldest first.
    #[serde(default)]
    pub entries: Vec<Entry>,
    /// Named states: the name, and the hash of the operation the design was at.
    #[serde(default, deserialize_with = "unique_map")]
    pub states: BTreeMap<String, String>,
}

/// A JSON object read as a map that refuses a key written twice: without this the last one would silently take the place of the others.
fn unique_map<'de, D, V>(deserializer: D) -> Result<BTreeMap<String, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct Unique<V>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>> serde::de::Visitor<'de> for Unique<V> {
        type Value = BTreeMap<String, V>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("an object")
        }

        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, V>()? {
                if out.contains_key(&key) {
                    return Err(serde::de::Error::custom(format!(
                        "the key `{key}` is written twice"
                    )));
                }
                out.insert(key, value);
            }
            Ok(out)
        }
    }
    deserializer.deserialize_map(Unique(std::marker::PhantomData))
}

impl History {
    /// Read a history from JSON, within the bounds of the configuration.
    pub fn from_json(text: &str, config: &Config) -> Result<History, HistoryError> {
        if text.len() as u64 > config.max_file_bytes {
            return Err(HistoryError::TooLarge {
                what: "history bytes",
            });
        }
        // a file saved by an editor of Windows may start with the mark of the byte order
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        // A JSON array would be read as a list of the fields in order, so only an object is a history.
        if !text.trim_start().starts_with('{') {
            return Err(HistoryError::BadJson("a JSON object was expected".into()));
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
                index: Some(index),
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
            if name.trim().is_empty() {
                out.push(Finding {
                    code: Code::BadState,
                    seq: 0,
                    index: None,
                    message: "a named state has an empty name".into(),
                });
            }
            if !hashes.contains(hash.as_str()) {
                out.push(Finding {
                    code: Code::UnknownState,
                    seq: 0,
                    index: None,
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
        let names: BTreeSet<&String> = self.states.keys().chain(other.states.keys()).collect();
        let states_differ = names
            .into_iter()
            .filter(|n| self.states.get(*n) != other.states.get(*n))
            .cloned()
            .collect();
        Divergence {
            states_differ,
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
    fn the_hashes_of_a_fixed_history_are_the_known_ones() {
        // fixed vectors: a change of the order of the fields or of the way the content is written changes every hash that was ever quoted
        let h = sample();
        let hashes: Vec<&str> = h.entries.iter().map(|e| e.hash.as_str()).collect();
        assert_eq!(
            hashes,
            [
                "961f111c48427100ec44776ea2b209699c2d4959e826f0e82c39822078a9337c",
                "86e4eb71c19ce71f5042ae29291741a3763d36ae3743a9d89f1072160ae4ff7d",
                "5edf3be1c191192608e0134c32f5a836e8cc1ff24380c1f80913e5a17acaf7cb"
            ]
        );
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
    fn a_state_with_an_empty_name_is_reported_whatever_it_points_to() {
        let mut h = sample();
        let last = h.entries[2].hash.clone();
        h.name_state("  ", &last);
        assert_eq!(codes(&h), vec![Code::BadState]);
        h.name_state("", "no-such-hash");
        assert_eq!(
            codes(&h),
            vec![Code::BadState, Code::UnknownState, Code::BadState]
        );
        assert_eq!(Code::BadState.as_str(), "PHOENIX-DHIS-005");
    }

    #[test]
    fn a_file_that_starts_with_the_byte_order_mark_is_read_like_one_that_does_not() {
        let config = Config::default();
        assert!(History::from_json("\u{feff}{}", &config).is_ok());
        assert!(History::from_json("\u{feff}\n  {}", &config).is_ok());
        // a mark in front of something that is not an object is still refused
        assert!(matches!(
            History::from_json("\u{feff}[]", &config),
            Err(HistoryError::BadJson(_))
        ));
        assert!(matches!(
            History::from_json("\u{feff}", &config),
            Err(HistoryError::BadJson(_))
        ));
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

    #[test]
    fn the_same_operations_in_two_histories_give_the_same_hashes_and_the_stored_one_is_the_expected_one(
    ) {
        let (a, b) = (sample(), sample());
        assert_eq!(a, b);
        for e in &a.entries {
            assert_eq!(e.hash, e.expected_hash());
        }
    }

    #[test]
    fn a_changed_summary_is_one_finding_and_names_the_place_of_the_operation() {
        let mut h = sample();
        h.entries[1].summary = "tampered".into();
        let found = h.check(&Config::default());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].code, Code::BrokenChain);
        assert_eq!(found[0].index, Some(1));
        assert_eq!(found[0].seq, 2);
    }

    #[test]
    fn two_operations_that_swap_places_break_the_chain() {
        let mut h = sample();
        h.entries.swap(0, 1);
        let found = codes(&h);
        assert!(!found.is_empty());
        assert!(found.iter().all(|c| *c == Code::BrokenChain));
    }

    #[test]
    fn an_empty_history_is_clean_compares_equal_to_itself_and_has_no_bad_operation() {
        let h = History::default();
        assert!(codes(&h).is_empty());
        let d = h.compare(&History::default());
        assert_eq!(d.common, 0);
        assert!(d.only_a.is_empty() && d.only_b.is_empty() && d.states_differ.is_empty());
        assert!(h.first_bad(|_| true).is_none());
    }

    #[test]
    fn a_history_compared_with_itself_shares_everything() {
        let h = sample();
        let d = h.compare(&h.clone());
        assert_eq!(d.common, 3);
        assert!(d.only_a.is_empty() && d.only_b.is_empty());
    }

    #[test]
    fn the_comparison_read_from_the_other_side_swaps_what_only_each_one_did() {
        let a = sample();
        let mut b = sample();
        b.append("net-modified", "n2", "joined n2", None, None)
            .unwrap();
        let (ab, ba) = (a.compare(&b), b.compare(&a));
        assert_eq!(ab.common, ba.common);
        assert_eq!(ab.only_a, ba.only_b);
        assert_eq!(ab.only_b, ba.only_a);
        assert_eq!(ab.only_b.len(), 1);
    }

    #[test]
    fn the_halving_finds_the_operation_where_the_design_turns_bad_for_every_place() {
        let mut h = History::default();
        for n in 0..40 {
            h.append("component-moved", &format!("c{n}"), "", None, None)
                .unwrap();
        }
        for turn in 1..=40usize {
            let found = h.first_bad(|prefix| prefix.len() >= turn).unwrap();
            assert_eq!(found.seq, turn as u64, "{turn}");
        }
        assert!(h.first_bad(|_| false).is_none());
    }

    #[test]
    fn a_long_history_is_chained_and_clean_without_a_panic() {
        let mut h = History::default();
        for n in 0..5000 {
            h.append("component-moved", &format!("c{}", n % 50), "", None, None)
                .unwrap();
        }
        assert_eq!(h.entries.len(), 5000);
        assert!(codes(&h).is_empty());
        assert_eq!(h.entries[4999].parent, h.entries[4998].hash);
    }

    #[test]
    fn a_text_that_is_not_an_object_is_an_error_and_never_a_panic() {
        let config = Config::default();
        for text in [
            "",
            " ",
            "null",
            "[]",
            "[1,2]",
            "12",
            "\"x\"",
            "{",
            "{\"entries\":3}",
        ] {
            assert!(
                matches!(
                    History::from_json(text, &config),
                    Err(HistoryError::BadJson(_))
                ),
                "{text:?}"
            );
        }
    }

    #[test]
    fn an_object_with_nothing_in_it_is_an_empty_history() {
        let h = History::from_json("{}", &Config::default()).unwrap();
        assert_eq!(h, History::default());
    }

    #[test]
    fn naming_a_state_again_replaces_it_and_a_state_on_a_real_operation_is_clean() {
        let mut h = sample();
        let first = h.entries[0].hash.clone();
        let last = h.entries[2].hash.clone();
        h.name_state("release", &first);
        h.name_state("release", &last);
        assert_eq!(h.states.len(), 1);
        assert_eq!(h.states["release"], last);
        assert!(codes(&h).is_empty());
    }

    #[test]
    fn a_refused_operation_leaves_the_history_as_it_was() {
        let mut h = sample();
        let before = h.clone();
        assert!(h.append("   ", "c1", "", None, None).is_err());
        assert!(h.append("moved", "", "", None, None).is_err());
        assert_eq!(h, before);
    }

    #[test]
    fn text_in_any_script_is_written_and_read_back_with_the_same_hashes() {
        let mut h = History::default();
        h.append(
            "object-created",
            "c1",
            "añadido \u{30b3}\u{30f3}\u{30c7}\u{30f3}\u{30b5} \u{7535}\u{5bb9}",
            Some("0,0"),
            None,
        )
        .unwrap();
        let text = h.to_json().unwrap();
        let again = History::from_json(&text, &Config::default()).unwrap();
        assert_eq!(again, h);
        assert!(codes(&again).is_empty());
        assert_eq!(again.to_json().unwrap(), text);
    }

    #[test]
    fn the_codes_are_written_in_order_and_none_repeats() {
        let all = [
            Code::BrokenChain,
            Code::BadEntry,
            Code::UnknownState,
            Code::DeletedReference,
            Code::BadState,
        ];
        let texts: Vec<&str> = all.iter().map(|c| c.as_str()).collect();
        assert_eq!(
            texts,
            [
                "PHOENIX-DHIS-001",
                "PHOENIX-DHIS-002",
                "PHOENIX-DHIS-003",
                "PHOENIX-DHIS-004",
                "PHOENIX-DHIS-005"
            ]
        );
    }

    #[test]
    fn a_state_written_twice_is_refused_and_not_replaced_in_silence() {
        let text = r#"{"states":{"release":"a","release":"b"}}"#;
        match History::from_json(text, &Config::default()) {
            Err(HistoryError::BadJson(message)) => {
                assert!(message.contains("release"), "{message}")
            }
            other => panic!("{other:?}"),
        }
        assert!(History::from_json(r#"{"states":{"a":"1","b":"1"}}"#, &Config::default()).is_ok());
    }

    #[test]
    fn a_history_cut_at_any_place_is_an_error_and_never_a_panic() {
        let mut h = sample();
        let last = h.entries[2].hash.clone();
        h.name_state("release \u{7248}", &last);
        let text = h.to_json().unwrap();
        for end in (0..text.len()).filter(|i| text.is_char_boundary(*i)) {
            assert!(
                matches!(
                    History::from_json(&text[..end], &Config::default()),
                    Err(HistoryError::BadJson(_))
                ),
                "cut at {end}"
            );
        }
    }

    #[test]
    fn every_field_of_an_operation_is_covered_by_its_hash() {
        for field in 0..7 {
            let mut h = sample();
            let e = &mut h.entries[2];
            match field {
                0 => e.seq = 9,
                1 => e.parent.push('0'),
                2 => e.kind.push('x'),
                3 => e.object.push('x'),
                4 => e.summary.push('x'),
                5 => e.before = Some(String::new()),
                _ => e.after = Some(String::new()),
            }
            assert_ne!(e.hash, e.expected_hash(), "field {field}");
            assert!(codes(&h).contains(&Code::BrokenChain), "field {field}");
        }
    }

    #[test]
    fn a_text_that_moves_from_one_field_to_the_next_gives_another_hash() {
        let mut a = History::default();
        a.append("moved", "c1", "ab", None, Some("c")).unwrap();
        let mut b = History::default();
        b.append("moved", "c1", "a", Some("b"), Some("c")).unwrap();
        let mut c = History::default();
        c.append("moved", "c1", "ab", Some(""), Some("c")).unwrap();
        assert_ne!(a.entries[0].hash, b.entries[0].hash);
        assert_ne!(a.entries[0].hash, c.entries[0].hash);
    }

    #[test]
    fn the_halving_asks_only_a_few_times_even_in_a_long_history() {
        let mut h = History::default();
        for _ in 0..1024 {
            h.append("component-moved", "c1", "", None, None).unwrap();
        }
        let asked = std::cell::Cell::new(0u32);
        let found = h.first_bad(|prefix| {
            asked.set(asked.get() + 1);
            prefix.len() >= 700
        });
        assert_eq!(found.map(|e| e.seq), Some(700));
        assert!(asked.get() <= 12, "{}", asked.get());
    }
}
