# Changelog

All notable changes to this project are documented here.

## [Unreleased]

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
