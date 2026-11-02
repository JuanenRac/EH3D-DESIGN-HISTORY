# EH3D-DESIGN-HISTORY: scope

Status: **planned**. EH3D-DESIGN-HISTORY has no code. This page says what it is meant to do, what it will not do, and where it sits. Nothing here has been built or run.

## Purpose

EH3D-DESIGN-HISTORY is planned as a version-control layer with the semantic history and the logical evolution of the design.

Semantic version history: records what changed (component moved, net modified, footprint changed, rule altered) and compares revisions by meaning.

## What it is planned to do

* Record semantic events: component moved, net modified, footprint changed, rule altered, simulation model updated, enclosure dimension changed.
* Compare two revisions by meaning, not only by a visual difference.
* Help to locate which change introduced a regression.
* Feed the visual difference of the server (added, removed, changed).
* Live in the history domain of the project, as non-critical data: a reference to an object that was deleted later becomes a diagnostic, not a failure.

## What it will not do

* It is not a replacement for a general version-control system and does not merge concurrent edits.
* It stores no private projects in this repository.
* Its writers do not read the clock or the host: identities and time stamps are given by the caller.

## Planned shape

A library without a window, written in Rust (planned), with a command-line interface and, where it applies, a kind of task for the server. Programs of the suite exchange work through project files or through the server, never by depending on each other.

Crates, names and file layout will be decided together with the first code; no crate exists yet.

## Where it sits

It belongs to the PHOENIX family of the EH3D Fabrication Suite: independent repositories that relate through versioned contracts. The design tool (EH3D-PHOENIX-PCB) does not depend at build time on the repositories that depend on it.
See DEPENDENCIES.md for the relations, REQUIREMENTS.md for the requirements and STATUS.md for the honest state.
