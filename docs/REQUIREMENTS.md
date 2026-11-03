# EH3D-DESIGN-HISTORY: requirements

This register is the single source of the state of each capability. Status values: planned, prototype, implemented, validated. Every requirement is `prototype` today (code and tests exist, only on data made by the tests); none has been used on real data or validated. High test coverage is not proof of geometric or electrical correctness, so each requirement says how it will be checked and what result is expected.

| Requirement | Description | Status | How it will be checked | Expected result | Known limits |
|---|---|---|---|---|---|
| DHIS-001 | Record semantic change events keyed by stable object identity. | prototype | automatic: property tests | Renaming an object does not change its identity in the history. | An operation names the stable identity of its object as text; whether that identity survives a rename is the job of the design tool, which produces the operations. |
| DHIS-002 | Compare two revisions by meaning. | prototype | automatic: reference revisions | Each reference change is reported as the expected event. | Compares operation by operation (same content, same hash) from the start and reports where the two histories separate; it does not compare two designs by their content. |
| DHIS-003 | Locate the change that introduced a regression. | prototype | automatic: a search over revisions with a predicate | The first bad revision is found. | Finds the first bad operation by halving, which assumes that once the design is bad it stays bad; the predicate is given by the caller. |
| DHIS-004 | Handle a reference to an object that was deleted later. | prototype | automatic | A diagnostic is raised; nothing fails. | Reports an operation on an object deleted earlier, until an operation of the creation kind makes it valid again; the two kinds are settings. Nothing fails. |
| DHIS-005 | Deterministic serialisation of the history. | prototype | automatic: round trip | Same content gives the same bytes. | The JSON is written with the fields in a fixed order and the states sorted by name; the same content gives the same bytes. |

Rules: a claim of correctness names the corpus, the tolerance and the tool version it holds for. A phase is complete when the measured rate does not drop, every failure is classified and fixed or documented, and the measured figure is reported.
