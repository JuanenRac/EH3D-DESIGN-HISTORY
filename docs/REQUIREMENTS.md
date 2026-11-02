# EH3D-DESIGN-HISTORY: requirements

This register is the single source of the state of each capability. Status values: planned, prototype, implemented, validated. Every requirement is `planned` today: nothing is implemented or verified. High test coverage is not proof of geometric or electrical correctness, so each requirement says how it will be checked and what result is expected.

| Requirement | Description | Status | How it will be checked | Expected result | Known limits |
|---|---|---|---|---|---|
| DHIS-001 | Record semantic change events keyed by stable object identity. | planned | automatic: property tests | Renaming an object does not change its identity in the history. | Not started: no limits known yet. |
| DHIS-002 | Compare two revisions by meaning. | planned | automatic: reference revisions | Each reference change is reported as the expected event. | Not started: no limits known yet. |
| DHIS-003 | Locate the change that introduced a regression. | planned | automatic: a search over revisions with a predicate | The first bad revision is found. | Not started: no limits known yet. |
| DHIS-004 | Handle a reference to an object that was deleted later. | planned | automatic | A diagnostic is raised; nothing fails. | Not started: no limits known yet. |
| DHIS-005 | Deterministic serialisation of the history. | planned | automatic: round trip | Same content gives the same bytes. | Not started: no limits known yet. |

Rules: a claim of correctness names the corpus, the tolerance and the tool version it holds for. A phase is complete when the measured rate does not drop, every failure is classified and fixed or documented, and the measured figure is reported.
