// SPDX-License-Identifier: GPL-3.0-or-later
//! EH3D-DESIGN-HISTORY: the semantic history of a design.
//!
//! A history is a list of **operations** (a component moved, a net modified, a rule altered), each one keyed by the stable identity of the object it concerns and
//! chained to the previous one by a SHA-256, so that a change in the past is found and two histories can be compared by meaning: where they agree, and what each one
//! did after they separated. Named **states** point to an operation. The history is plain JSON and the same content always gives the same bytes.
//!
//! This crate never reads the clock and never reads a design: the model that produces the operations belongs to the design tool.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cli;
pub mod config;
pub mod error;
pub mod history;

pub use error::HistoryError;
pub use history::{Entry, History};

/// The stable code of a finding. Codes are never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Code {
    /// The numbering, the link to the previous operation or the hash of an operation does not hold.
    BrokenChain,
    /// An operation without a kind or without an object.
    BadEntry,
    /// A named state points to an operation that does not exist.
    UnknownState,
    /// An operation refers to an object that an earlier operation deleted.
    DeletedReference,
}

impl Code {
    /// The text of the code, such as `PHOENIX-DHIS-001`.
    pub fn as_str(self) -> &'static str {
        match self {
            Code::BrokenChain => "PHOENIX-DHIS-001",
            Code::BadEntry => "PHOENIX-DHIS-002",
            Code::UnknownState => "PHOENIX-DHIS-003",
            Code::DeletedReference => "PHOENIX-DHIS-004",
        }
    }
}
