# Changelog

All notable changes to this project are documented here.

## [Unreleased]

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
