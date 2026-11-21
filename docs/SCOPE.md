# EH3D-DESIGN-HISTORY: scope

Status: **early code**. EH3D-DESIGN-HISTORY is a library and a command line that read a history of operations chained by SHA-256, check it, print it and compare two histories. It has been tried on data made by its own tests only. This page says what it is for, what it will not do and where it sits among the other programs of the suite.

## Purpose

The semantic history of a design: what changed (a component moved, a net modified, a footprint changed, a rule altered) kept as a chain of operations that can be compared by meaning and searched for the change that made a design bad.

## What it does

* Reads a history (a JSON file within a size limit, with or without a byte order mark) of operations numbered and chained by the SHA-256 of the one before, and refuses unknown fields.
* Checks the chain: a change in the past breaks every hash after it, and an operation needs a kind and an object.
* Checks the named states of the history: a state must point to an operation of the history and must have a name.
* Reports an operation on an object that was deleted earlier, until an operation of the creation kind makes it valid again; the two kinds are settings and nothing fails.
* Compares two histories operation by operation and says where they separate; finds the first bad operation by halving, with a predicate given by the caller.
* Writes the history with the fields in a fixed order and the states sorted by name, so the same content gives the same bytes; runs a seeded twin of the design tool with injectable faults and a recovery.

## What it will not do

* It is not a general version-control system and does not merge concurrent edits.
* It does not read a design or compare two designs by their content: it compares histories of operations.
* It does not read the clock or the host: identities and time stamps are given by the caller.
* It stores no private project.
* It does not claim that anything was proven on a real design: the evidence of every capability is `simulator` or `host-test`.

## Where it sits

* **It reads:** a history file (JSON) and a settings file (the kinds of operation that create and delete an object, and the size limits).
* **It writes:** the history it is asked to print, to the standard output or to a file the person names; it never writes to the history it reads.
* **It is read by:** nothing reads it yet: the server will read the history to show the visual difference (added, removed, changed) once the design tool produces operations.
* **It depends on:** nothing of the suite at build time; the producer of the operations belongs to the design tool (EH3D-PHOENIX-PCB).

## What it never decides

Whether a change to a design is right, and the limits, the interlocks and the emergency stop of any machine, belong to the person who designs and to the machine. A clean history shows that the chain was not altered; it does not show that the design is good. Nothing here starts motion.

The requirements of the program, with the state of each and the test that shows it, are in `REQUIREMENTS.md`; the state in a paragraph is in `STATUS.md`.
