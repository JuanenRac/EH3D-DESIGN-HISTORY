# EH3D-DESIGN-HISTORY: status

## Current state

EH3D-DESIGN-HISTORY is early code. Version 0.0.1 holds only the base that every repository of the suite carries (manifest, README files in eight languages, changelog, CI, shared checks) and the written scope, requirements, dependencies, threat model and decisions. It also has a Rust library and a command line that read a history of operations chained by SHA-256, check it, print it and compare two histories. They have run only on data made by their own tests; nothing has been run on a machine with a real design, and no requirement is validated.

## Capabilities

The official state of each capability is in REQUIREMENTS.md: 5 requirements, all `prototype`. This page never states a state that REQUIREMENTS.md does not.

## Open questions

* How fine-grained the semantic events of the first version should be.

## Needs from other repositories

* EH3D-PHOENIX-PCB: stable identities and the model of the design
* EH3D-COMMON: canonical JSON and the container

## Known technical debt and omitted tests

14 tests of the Rust code and the 3 of the base; the Rust tests build their own histories and never touch a real design. `cargo deny` is not run yet.
