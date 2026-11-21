# EH3D-DESIGN-HISTORY: requirements

This register is the single source of the state of each capability: `STATUS.md` and the README files cite it and never state a state that it does not. A state is one of `planned`, `prototype`, `implemented` and `validated`. `validated` means that a person has run the capability on a real machine and the evidence of the manifest says `bench` or `field`; nothing here is validated, because nothing of this program has been run with a real design tool or a real design.

The register has 24 requirements: 18 implemented, 4 prototype, 2 planned and 0 validated. By progress: 18 done, 4 partial, 0 pending and 2 blocked. High coverage is not proof of correctness, so each row says how it is checked: the test that names it, and what it shows.

## How to read the register

* **ID:** `DHIS-NNN`, never reused and never renumbered; a new capability takes the next number.
* **State:** `implemented` is code that is complete and tested on a computer, `prototype` is part of the capability, `planned` is not written.
* **Progress:** `done`, `partial`, `pending` (can be written without hardware) or `blocked` (waits for hardware or for another program of the suite).
* **Evidence:** `simulator` when the test runs against a seeded twin of the design tool that makes the operations, with faults injected on purpose, `host-test` when it checks a calculation, a reader or a screen-less piece, `none` when nothing is written yet.
* **Test:** the acceptance criterion: `file::name` is the test `name` of `tests/file.rs` (`src/file::name` is a test inside `src/file.rs`), which must exist. For a capability that is not written the cell says `planned:` and the name of the test that will show it; the day that test exists, the row must change.
* **Limits:** what is not done or not proven; a row that is `partial` or `blocked` always says it.

The check is `tests/reqs.rs`: it reads this table, finds each test and fails when a state, a test or a count does not agree.

## The history and its chain

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| DHIS-001 | Record semantic change events keyed by the stable identity of the object. | prototype | partial | host-test | `src/history::appended_operations_are_numbered_chained_and_clean` | An operation names the identity of its object as text; whether that identity survives a rename is the job of the design tool, which produces the operations. |
| DHIS-002 | Compare two revisions by meaning. | prototype | partial | host-test | `src/history::two_histories_are_compared_from_where_they_separate` | It compares operation by operation (the same content has the same hash) and reports where two histories separate; it does not compare two designs by their content. |
| DHIS-003 | Locate the change that introduced a regression. | prototype | partial | host-test | `src/history::the_first_bad_operation_is_found_by_halving` | It halves, which assumes that once the design is bad it stays bad; the predicate is given by the caller. |
| DHIS-004 | Handle a reference to an object that was deleted later: a diagnostic, never a failure. | prototype | partial | host-test | `src/history::a_reference_to_a_deleted_object_is_reported_and_creating_it_again_clears_it` | The operation kinds that create and delete are settings; the design tool has to produce them. |
| DHIS-005 | Deterministic serialisation of the history: the same content gives the same bytes and reads back. | implemented | done | host-test | `src/history::the_same_content_gives_the_same_bytes_and_reads_back` | - |
| DHIS-006 | An operation needs a kind and an object. | implemented | done | host-test | `src/history::an_operation_needs_a_kind_and_an_object` | - |
| DHIS-007 | The hashes of a fixed history are known values, so a change of the format cannot go unnoticed. | implemented | done | host-test | `src/history::the_hashes_of_a_fixed_history_are_the_known_ones` | - |
| DHIS-008 | A change in the past breaks the chain from there on. | implemented | done | host-test | `src/history::a_change_in_the_past_breaks_the_chain` | - |
| DHIS-009 | A state must point to an operation of the history. | implemented | done | host-test | `src/history::a_state_must_point_to_an_operation_of_the_history` | - |
| DHIS-010 | A state with an empty name is reported whatever it points to. | implemented | done | host-test | `src/history::a_state_with_an_empty_name_is_reported_whatever_it_points_to` | - |
| DHIS-011 | A file that starts with a byte order mark is read like one that does not. | implemented | done | host-test | `src/history::a_file_that_starts_with_the_byte_order_mark_is_read_like_one_that_does_not` | - |
| DHIS-012 | Bad JSON, unknown fields and the size limits are refused without a panic. | implemented | done | host-test | `src/history::bad_json_unknown_fields_and_the_size_limits_are_refused` | - |

## The command line and the settings

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| DHIS-013 | `check`, `log` and `compare` exit with the code that tells the result. | implemented | done | host-test | `src/cli::check_log_and_compare_exit_codes` | - |
| DHIS-014 | Two histories with the same operations but different states are not the same. | implemented | done | host-test | `src/cli::two_histories_with_the_same_operations_but_different_states_are_not_the_same` | - |
| DHIS-015 | Bad input and bad arguments exit with 2. | implemented | done | host-test | `src/cli::bad_input_and_bad_arguments_exit_2` | - |
| DHIS-016 | A settings file changes the kinds of creation and deletion; one kind cannot both create and delete. | implemented | done | host-test | `src/cli::a_settings_file_changes_the_kinds_of_creation_and_deletion` | - |
| DHIS-017 | A bad line of the settings file is refused. | implemented | done | host-test | `src/config::lines_change_values_and_bad_lines_are_refused` | - |

## Simulation of the design tool

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| DHIS-018 | A seeded twin makes the same history for the same seed and another for another seed; what it makes is always clean. | implemented | done | simulator | `src/simulation::the_same_seed_gives_the_same_history_and_another_seed_a_different_one` | The twin imitates the form of the operations, not a real design tool. |
| DHIS-019 | Every fault that can be injected is found with its code. | implemented | done | simulator | `src/simulation::every_fault_is_found_with_its_code` | - |
| DHIS-020 | Recovery keeps the clean prefix and the tool can go on; a dropped operation is recovered up to the gap. | implemented | done | simulator | `src/simulation::recovery_keeps_the_clean_prefix_and_the_tool_can_go_on` | - |
| DHIS-021 | The search by halving finds the operation that made the design bad. | implemented | done | simulator | `src/simulation::the_search_by_halving_finds_the_operation_that_made_the_design_bad` | - |
| DHIS-022 | Two producers that separate are compared from where they diverge. | implemented | done | simulator | `src/simulation::two_producers_that_separate_are_compared_from_where_they_diverge` | - |
| DHIS-023 | Read the operations that a real design tool produces. | planned | blocked | none | `planned:producer::a_history_from_the_design_tool_is_read_and_checked` | Waits for the design tool (EH3D-PHOENIX-PCB) to produce operations; until then only data made by the tests has been used. |
| DHIS-024 | Feed the visual difference of the server (added, removed, changed) from the history. | planned | blocked | none | `planned:server_diff::the_difference_of_two_histories_becomes_added_removed_and_changed` | Waits for the server to ask for it; the comparison of two histories exists, the format the server needs does not. |
