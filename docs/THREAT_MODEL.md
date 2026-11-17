# EH3D-DESIGN-HISTORY: threat model

Basic model for early code. The rows marked partial have a mitigation written and tests for it, run only on data made by the tests themselves; the rows marked planned have nothing yet. No row is assessed against real data.

| Threat | Planned mitigation | Planned test | State | Residual risk |
|---|---|---|---|---|
| A hostile history entry that references objects that do not exist | an operation needs a kind and an object, a state must point to an operation of the history, and a reference to a deleted object is reported | `an_operation_needs_a_kind_and_an_object`, `a_state_must_point_to_an_operation_of_the_history`, `a_reference_to_a_deleted_object_is_reported_and_creating_it_again_clears_it` | partial | hostile samples written by hand; no fuzzing yet |
| A history that reveals private design content when shared | export of the history is explicit and can be redacted | leak test | planned | not assessed: nothing is implemented |
| Corrupt or malicious project or input files (oversized entries, path tricks, deep nesting, resource exhaustion) | the JSON reader refuses bad syntax, unknown fields and oversized input, and a change in the past breaks the chain of the history | `bad_json_unknown_fields_and_the_size_limits_are_refused`, `a_change_in_the_past_breaks_the_chain` | partial | hostile samples written by hand; no fuzzing yet |
| Temporary files, logs and reports that reveal private designs | no private data in outputs; redaction | automated pattern search | planned | not assessed: nothing is implemented |
| Compromised dependency or binary | pinned versions, licence and advisory checks, software bill of materials for releases | CI checks | planned | not assessed: nothing is implemented |
| Power loss or forced close during a save | atomic write: temporary file, then rename; old file never deleted first | interruption tests at controlled points | planned | not assessed: nothing is implemented |

Minimum negative tests once there is code: hostile files and resource limits; secrets absent from logs, errors and reports; pinned dependencies and licence review before publishing. Report a vulnerability privately as described in SECURITY.md, without attaching private designs.
