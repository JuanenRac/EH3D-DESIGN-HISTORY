# EH3D-DESIGN-HISTORY: status

## Current state

EH3D-DESIGN-HISTORY is early code on a computer: a Rust library and a command line (`check`, `log`, `compare`) that read a history of operations chained by SHA-256, check it, print it and compare two histories. A seeded twin of the design tool makes the operations, injects five faults and recovers to the last clean operation. All of it has run only on data made by its own tests; nothing has been run with a real design tool and no requirement is validated.

## Capabilities

The official state of each capability is in `REQUIREMENTS.md`. The register has 24 requirements: 18 implemented, 4 prototype, 2 planned and 0 validated. By progress: 18 done, 4 partial, 0 pending and 2 blocked. This page never states a state that `REQUIREMENTS.md` does not.

## Open questions

* How fine-grained the semantic events of the first version should be.
* Which operation kinds the design tool will produce.

## What it needs from other repositories

* EH3D-PHOENIX-PCB: stable identities, the model of the design and the producer of the operations.
* EH3D-COMMON: canonical JSON and the container.

## Known technical debt and omitted tests

* 37 tests of the Rust code (nine of them keep this register true) and the 3 of the base; the Rust tests build their own histories and never touch a real design.
* Nine of the tests run the simulation: the five faults that can be injected (a changed operation, a missing one, a state that points nowhere, a use after deletion, an operation without a kind), the recovery to the last clean operation and the search by halving over a long history. What is pending is the proof against a real design tool, which does not exist yet.
* `cargo deny` is not run yet.
