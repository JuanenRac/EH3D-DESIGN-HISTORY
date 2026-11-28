# Changelog

All notable changes to this project are documented here.

## [Unreleased]

## [0.0.6] - A name read from a file can no longer forge a line of the output

- **A name read from a file can no longer forge a line of the output:** line breaks and control characters in the error messages (and Unicode line separators anywhere in the output) are printed as spaces. What is compared, hashed and the exit codes do not change.
- **1 more test** of the Rust code, for the above.

## [0.0.5] - Correct recovery of damaged histories, and many more tests

- **The recovery of a damaged history cuts at the right place:** it looked for the number the faulty operation carried, so an operation numbered 0 stayed in the result and one that copied the number of an earlier one cut away good operations. It now uses the place in the list, and drops a state with an empty name.
- **A named state written twice is refused** instead of the last one replacing the first in silence.
- **A line feed in the file can no longer write lines of its own** in the output of `check`, `log` and `compare`.
- **10 more tests** of the Rust code: every field is covered by its hash, a history cut at every place, a seeded campaign of damage and the number of questions of the search by halving.
- **More tests, for the shared tools of the base and for the history:** `tools/test_shared_tools.py` adds 65 tests that run the lessons linter, the workflow check, the CHANGELOG check of a pull request and the banner check on small inputs made by the test itself (findings, baselines, exit codes and the shape of what they print). The Rust code gets 14 more: a changed summary is one finding that names the place of the operation, two operations that swap places break the chain, an empty history is clean and has no bad operation, the comparison read from the other side swaps what only each one did, the halving finds the turning point for each of 40 places, a history of 5000 operations is chained and clean, a text that is not an object is an error and never a panic, naming a state again replaces it, a refused operation leaves the history as it was, text in any script is written and read back with the same hashes, and the codes keep their order.
- **The register of requirements names a test for each row:** `docs/REQUIREMENTS.md` has one row for each capability with an identifier (`DHIS-001` on), its state (planned, prototype, implemented or validated), how far it is (done, partial, pending or blocked), its evidence (simulator, host-test or none), the test that shows it and its limits; a capability that is not written names the test that will show it. `tests/reqs.rs` reads the table and fails when a test does not exist, when a state does not agree with the progress or the evidence, when `docs/STATUS.md` does not say the same numbers, when a README does not link the documents or when a document holds a date or a private reference. `docs/SCOPE.md`, `docs/STATUS.md` and `docs/NOT_YET.md` are rewritten to the same shape and the eight READMEs link the five documents.
- The shared workflow checker (`tools/workflow_hygiene.py`) is the current one of the documentation hub: the filters under a trigger (branches, paths, types) are no longer read as events.

## [0.0.4] - Files with a byte order mark and stricter settings

- **A test fixes the known value of the hash of every operation of a fixed history:** until now the tests compared two calculations of the same code, so an accidental change of the order of the fields or of the way the content is written would have changed every hash that was ever quoted without a test failing. The test writes the expected hash of a fixed case.

- **The threat model says what the code already does:** the rows of `docs/THREAT_MODEL.md` that have a mitigation in the code and a test for it are marked `partial` and name the tests and what is still not covered; the rows without either stay `planned`.

- **A finding says where its operation is, and `compare` also compares the named states:** `Finding` has a new field `index` (the place of the operation in the list, whatever number it carries; the text of `check` adds `(position N)` when the number does not match), `Divergence` has `states_differ`, and `compare` lists `state differs: NAME` and exits with 1 when two histories with the same operations name different states. Tests cover it.
- **Simulation mode:** add `src/simulation.rs`, a host twin of the design tool: a seeded producer of operations (the same seed gives the same history), five injectable faults (changed content, missing operation, dangling state, use after deletion, operation without a kind), a recovery that keeps the longest clean prefix and a producer that resumes from a recovered history. Nine deterministic tests cover them. The manifest now says `evidence: simulator`; nothing has run against a real design tool.
- **A history file saved with the byte order mark is read:** a file that starts with the mark an editor of Windows writes is read like one without it; a mark in front of something that is not an object is still refused.
- **The check names a state with an empty name:** `PHOENIX-DHIS-005` is reported for a named state whose name is empty or only blanks, whatever operation it points to.
- **The settings refuse a kind that creates and deletes, and a limit of zero bytes:** `create_kind` and `delete_kind` with the same name, or `max_file_bytes = 0`, are a bad value of their line (the two kinds can still be swapped).

## [0.0.3] - Split CI with a build cache and a pinned Rust toolchain

- **The shared validation tools are updated:** the validator also runs the workflow and lessons checks of the suite with their latest rules, including a check to run before every push.
- **The CI validation checks the workflows and the lessons of past CI failures against a baseline:** `tools/workflow_hygiene.py` and `tools/lessons_lint.py` (copies of the shared tools) run in the validation, and only findings that are not in `tools/workflow_hygiene_baseline.json` and `tools/lessons_lint_baseline.json` fail it, so what exists today stays as it is and a new one is caught. `python tools/workflow_hygiene.py --root . --detail` and `python tools/lessons_lint.py --repo .` list the findings.
- **The validation of the CI is the reusable one of the documentation hub:** the job `validate` calls `validate.yml` of EH3D-DOCS (pinned by commit) and the check of the CHANGELOG is a job of its own.
- **The Rust jobs keep a cache of the build:** `rust.yml` uses `Swatinem/rust-cache`, pinned by its commit, after the installation of Rust, so a run that changes little does not build the dependencies again.
- **The CI is split in two workflows:** the validation of the repository (manifest, documentation, pictures, tools) stays in `ci.yml` and the Rust jobs move to `rust.yml`, which does not run for a change that only touches Markdown files or pictures, so a change of the documentation no longer builds the code on two systems. The names of the jobs are the same.
- **The Rust toolchain is pinned:** `rust-toolchain.toml` fixes the version of Rust (the same one in every repository of the suite) with `rustfmt` and `clippy`, the workflows install that version and not `stable`, so a new release of Rust cannot turn the CI red by itself.

## [0.0.2] - First release: the library, the command line, the stable error codes and the tests

- **Brand:** Add the application icon and the logos of the program.
- **Shared checks updated:** the validator also compares the badge and the Version row of the README with the manifest, fails when one of its checks cannot run, and the CHANGELOG step of the workflow reads the base branch from the environment.

- Add the library: operations keyed by the stable identity of an object and chained by SHA-256, named states, a serialisation that gives the same bytes for the same content, the comparison of two histories from where they separate and the search of the first bad operation by halving.
- Report, with stable codes `PHOENIX-DHIS-001` to `PHOENIX-DHIS-004`: a broken chain, an operation without a kind or an object, a state that points to no operation and a reference to an object deleted earlier.
- Add the command line (`check`, `log`, `compare`) and a settings file. Run format, lints and tests of the Rust code on Linux and Windows in the CI.
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.

## [0.0.1] - The base of the repository

- **The base that every repository of the suite has:** the manifest, the README files in eight languages, this changelog, the CI workflow, the issue and pull-request templates, and the shared checks of EH3D-DOCS (the repository check and the drawers of the two README pictures).
- **A planned project, described honestly:** the scope, the requirements (all planned), the relations with other repositories, the threat model, the decisions that apply and the rules for writing code are in `docs/`; there is no code yet.
