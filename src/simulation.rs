// SPDX-License-Identifier: GPL-3.0-or-later
//! The simulation mode: a host twin of the design tool that produces operations, faults to inject into a history and the recovery from them.
//!
//! The producer of the operations belongs to the design tool, which does not exist yet. [`DesignerTwin`] stands in for it: given a seed it makes the same
//! history every time (it never reads the clock), so the checks, the comparison and the search by halving can be exercised against a long history, against
//! every kind of fault ([`Fault`]) and against the recovery ([`recover`]). Nothing here has run against a real design tool.

use crate::config::Config;
use crate::history::{Entry, History};
use crate::Code;

/// A small deterministic generator (SplitMix64): the same seed always gives the same numbers.
#[derive(Debug, Clone)]
pub struct SimRng(u64);

impl SimRng {
    /// A generator that starts from `seed`.
    pub fn new(seed: u64) -> SimRng {
        SimRng(seed)
    }

    /// The next number.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number below `bound` (`bound` must not be 0).
    pub fn below(&mut self, bound: u64) -> u64 {
        self.next_u64() % bound
    }
}

/// A fault to inject into a history, as a damaged file or a faulty producer would cause it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// The content of the operation with this number is changed after the fact.
    Tamper(u64),
    /// The operation with this number disappears.
    Drop(u64),
    /// A named state points to an operation that does not exist.
    DanglingState,
    /// An operation refers to an object after it was deleted.
    UseAfterDelete,
    /// The last operation loses its kind.
    EmptyKind,
}

/// The twin of the design tool: a seeded producer of operations.
#[derive(Debug, Clone)]
pub struct DesignerTwin {
    rng: SimRng,
    config: Config,
    live: Vec<String>,
    created: u64,
}

impl DesignerTwin {
    /// A twin that produces the same operations for the same seed.
    pub fn new(seed: u64, config: Config) -> DesignerTwin {
        DesignerTwin {
            rng: SimRng::new(seed),
            config,
            live: Vec::new(),
            created: 0,
        }
    }

    /// A twin that goes on from an existing history (for example a recovered one): it knows which objects are alive in it.
    pub fn resume(seed: u64, config: Config, history: &History) -> DesignerTwin {
        let mut twin = DesignerTwin::new(seed, config);
        for e in &history.entries {
            if e.kind == twin.config.create_kind {
                twin.created += 1;
                twin.live.push(e.object.clone());
            } else if e.kind == twin.config.delete_kind {
                twin.live.retain(|o| o != &e.object);
            }
        }
        twin
    }

    /// Add `steps` operations to `history`: objects are created, moved, modified and deleted, and an object is never used after it was deleted.
    pub fn run(&mut self, history: &mut History, steps: usize) {
        for _ in 0..steps {
            let roll = self.rng.below(10);
            if self.live.is_empty() || roll < 3 {
                self.created += 1;
                let object = format!("obj-{:04}", self.created);
                let after = format!("{},{}", self.rng.below(100), self.rng.below(100));
                let kind = self.config.create_kind.clone();
                history
                    .append(&kind, &object, "created", None, Some(&after))
                    .expect("the twin makes valid operations");
                self.live.push(object);
            } else if roll < 4 && self.live.len() > 1 {
                let at = self.rng.below(self.live.len() as u64) as usize;
                let object = self.live.remove(at);
                let kind = self.config.delete_kind.clone();
                history
                    .append(&kind, &object, "deleted", None, None)
                    .expect("the twin makes valid operations");
            } else {
                let at = self.rng.below(self.live.len() as u64) as usize;
                let object = self.live[at].clone();
                let before = format!("{}", self.rng.below(100));
                let after = format!("{}", self.rng.below(100));
                let kind = if roll < 7 { "moved" } else { "modified" };
                history
                    .append(kind, &object, "changed", Some(&before), Some(&after))
                    .expect("the twin makes valid operations");
            }
        }
    }

    /// A new history of `steps` operations with a state named `latest` on the last one.
    pub fn history(&mut self, steps: usize) -> History {
        let mut history = History::default();
        self.run(&mut history, steps);
        if let Some(last) = history.entries.last() {
            let hash = last.hash.clone();
            history.name_state("latest", &hash);
        }
        history
    }
}

/// Inject a fault into a history. Returns `false` when the history is too short for the fault (nothing is changed then).
pub fn inject(history: &mut History, fault: Fault, config: &Config) -> bool {
    match fault {
        Fault::Tamper(seq) => match history.entries.iter_mut().find(|e| e.seq == seq) {
            Some(entry) => {
                entry.after = Some(format!(
                    "{}-tampered",
                    entry.after.clone().unwrap_or_default()
                ));
                true
            }
            None => false,
        },
        Fault::Drop(seq) => {
            let before = history.entries.len();
            history.entries.retain(|e| e.seq != seq);
            history.entries.len() != before
        }
        Fault::DanglingState => {
            history.states.insert("ghost".into(), "0".repeat(64));
            true
        }
        Fault::UseAfterDelete => {
            let deleted = history
                .entries
                .iter()
                .find(|e| e.kind == config.delete_kind)
                .map(|e| e.object.clone());
            match deleted {
                Some(object) => history
                    .append("moved", &object, "after deletion", None, Some("1,1"))
                    .is_ok(),
                None => false,
            }
        }
        Fault::EmptyKind => match history.entries.last_mut() {
            Some(entry) => {
                entry.kind.clear();
                true
            }
            None => false,
        },
    }
}

/// The recovery: keep the longest prefix of operations that the checks accept, drop the states that point outside it and say how many operations were lost.
pub fn recover(history: &History, config: &Config) -> (History, usize) {
    let mut keep = history.entries.len();
    for finding in history.check(config) {
        if finding.code != Code::UnknownState && finding.seq > 0 {
            if let Some(index) = history.entries.iter().position(|e| e.seq == finding.seq) {
                keep = keep.min(index);
            }
        }
    }
    let entries: Vec<Entry> = history.entries[..keep].to_vec();
    let mut states = history.states.clone();
    states.retain(|_, hash| entries.iter().any(|e| &e.hash == hash));
    let lost = history.entries.len() - entries.len();
    (History { entries, states }, lost)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn twin(seed: u64) -> DesignerTwin {
        DesignerTwin::new(seed, Config::default())
    }

    fn codes(history: &History) -> Vec<Code> {
        history
            .check(&Config::default())
            .iter()
            .map(|f| f.code)
            .collect()
    }

    #[test]
    fn the_same_seed_gives_the_same_history_and_another_seed_a_different_one() {
        let a = twin(7).history(60);
        let b = twin(7).history(60);
        assert_eq!(a.to_json().unwrap(), b.to_json().unwrap());
        assert_ne!(a.to_json().unwrap(), twin(8).history(60).to_json().unwrap());
    }

    #[test]
    fn a_history_made_by_the_twin_is_always_clean() {
        for seed in 0..20 {
            let h = twin(seed).history(80);
            assert_eq!(h.entries.len(), 80);
            assert!(codes(&h).is_empty(), "seed {seed}");
        }
    }

    #[test]
    fn every_fault_is_found_with_its_code() {
        let config = Config::default();
        let cases = [
            (Fault::Tamper(10), Code::BrokenChain),
            (Fault::Drop(10), Code::BrokenChain),
            (Fault::DanglingState, Code::UnknownState),
            (Fault::UseAfterDelete, Code::DeletedReference),
            (Fault::EmptyKind, Code::BadEntry),
        ];
        for (fault, code) in cases {
            let mut h = twin(3).history(200);
            assert!(inject(&mut h, fault, &config), "{fault:?}");
            assert!(codes(&h).contains(&code), "{fault:?}");
        }
    }

    #[test]
    fn a_fault_on_a_history_too_short_changes_nothing() {
        let config = Config::default();
        let mut h = History::default();
        for fault in [
            Fault::Tamper(1),
            Fault::Drop(1),
            Fault::UseAfterDelete,
            Fault::EmptyKind,
        ] {
            assert!(!inject(&mut h, fault, &config));
        }
        assert!(h.entries.is_empty());
    }

    #[test]
    fn recovery_keeps_the_clean_prefix_and_the_tool_can_go_on() {
        let config = Config::default();
        let original = twin(11).history(50);
        let mut broken = original.clone();
        inject(&mut broken, Fault::Tamper(30), &config);
        inject(&mut broken, Fault::DanglingState, &config);
        let (mut repaired, lost) = recover(&broken, &config);
        assert_eq!(repaired.entries.len(), 29);
        assert_eq!(lost, 21);
        assert!(codes(&repaired).is_empty());
        // what was kept is exactly the start of the original
        assert_eq!(original.compare(&repaired).common, 29);
        // the producer goes on from the recovered history
        let mut again = DesignerTwin::resume(12, Config::default(), &repaired);
        again.run(&mut repaired, 10);
        assert!(codes(&repaired).is_empty());
        assert_eq!(repaired.entries.len(), 39);
    }

    #[test]
    fn recovery_of_a_clean_history_loses_nothing() {
        let h = twin(5).history(40);
        let (same, lost) = recover(&h, &Config::default());
        assert_eq!(lost, 0);
        assert_eq!(same, h);
    }

    #[test]
    fn a_dropped_operation_is_recovered_up_to_the_gap() {
        let config = Config::default();
        let mut h = twin(9).history(40);
        inject(&mut h, Fault::Drop(15), &config);
        let (repaired, lost) = recover(&h, &config);
        assert_eq!(repaired.entries.len(), 14);
        assert_eq!(lost, 25);
        assert!(codes(&repaired).is_empty());
    }

    #[test]
    fn the_search_by_halving_finds_the_operation_that_made_the_design_bad() {
        let h = twin(21).history(100);
        let target = h.entries[63].hash.clone();
        let found = h
            .first_bad(|prefix| prefix.iter().any(|e| e.hash == target))
            .unwrap();
        assert_eq!(found.seq, 64);
        assert!(h.first_bad(|_| false).is_none());
    }

    #[test]
    fn two_producers_that_separate_are_compared_from_where_they_diverge() {
        let base = twin(2).history(20);
        let mut a = base.clone();
        let mut b = base.clone();
        let mut ta = DesignerTwin::resume(100, Config::default(), &base);
        let mut tb = DesignerTwin::resume(200, Config::default(), &base);
        ta.run(&mut a, 6);
        tb.run(&mut b, 9);
        let d = a.compare(&b);
        assert!(d.common >= 20);
        assert_eq!(d.common + d.only_a.len(), 26);
        assert_eq!(d.common + d.only_b.len(), 29);
    }
}
