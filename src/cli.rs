// SPDX-License-Identifier: GPL-3.0-or-later
//! The command line. It is a thin layer: parsing the arguments and printing; the checks live in the other modules.
//!
//! Exit codes: 0 everything is as expected, 1 the history has findings or two histories differ, 2 the input or the arguments are unusable.

use crate::config::Config;
use crate::error::HistoryError;
use crate::history::History;
use std::fs;
use std::io::Write;
use std::path::Path;

const USAGE: &str = "\
eh3d-design-history [--config FILE] <command>

  check <history.json>                     verify the chain of operations, the states and the references to deleted objects
  log <history.json>                       print the operations, oldest first
  compare <a.json> <b.json>                print where two histories separate and what each did afterwards

Exit codes: 0 nothing found or equal, 1 findings or different, 2 unusable input.
";

fn read_text(path: &Path, limit: u64) -> Result<String, HistoryError> {
    let io = |source| HistoryError::Io {
        path: path.to_path_buf(),
        source,
    };
    if fs::metadata(path).map_err(io)?.len() > limit {
        return Err(HistoryError::TooLarge { what: "input file" });
    }
    fs::read_to_string(path).map_err(io)
}

/// Run the command line with the given arguments (without the program name). Returns the exit code.
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    match dispatch(args, out) {
        Ok(code) => code,
        Err(None) => {
            let _ = err.write_all(USAGE.as_bytes());
            2
        }
        Err(Some(error)) => {
            let _ = writeln!(err, "error: {error}");
            2
        }
    }
}

fn dispatch(args: &[String], out: &mut dyn Write) -> Result<i32, Option<HistoryError>> {
    let mut args = args.to_vec();
    let mut config = Config::default();
    if args.first().map(String::as_str) == Some("--config") {
        if args.len() < 3 {
            return Err(None);
        }
        config = Config::parse(&read_text(Path::new(&args[1]), 1024 * 1024).map_err(Some)?)
            .map_err(Some)?;
        args.drain(..2);
    }
    let load = |path: &String| -> Result<History, Option<HistoryError>> {
        let text = read_text(Path::new(path), config.max_file_bytes).map_err(Some)?;
        History::from_json(&text, &config).map_err(Some)
    };
    let rest: Vec<&str> = args.iter().map(String::as_str).collect();
    match rest.as_slice() {
        ["check", _] => {
            let history = load(&args[1])?;
            let findings = history.check(&config);
            for f in &findings {
                let _ = writeln!(out, "{} #{}: {}", f.code.as_str(), f.seq, f.message);
            }
            let _ = writeln!(
                out,
                "# {} operations, {} findings",
                history.entries.len(),
                findings.len()
            );
            Ok(if findings.is_empty() { 0 } else { 1 })
        }
        ["log", _] => {
            let history = load(&args[1])?;
            for e in &history.entries {
                let _ = writeln!(out, "{} {} {} {}", e.seq, e.kind, e.object, e.summary);
            }
            Ok(0)
        }
        ["compare", _, _] => {
            let (a, b) = (load(&args[1])?, load(&args[2])?);
            let d = a.compare(&b);
            let _ = writeln!(out, "common {}", d.common);
            for e in &d.only_a {
                let _ = writeln!(out, "only first: {} {} {}", e.seq, e.kind, e.object);
            }
            for e in &d.only_b {
                let _ = writeln!(out, "only second: {} {} {}", e.seq, e.kind, e.object);
            }
            Ok(if d.only_a.is_empty() && d.only_b.is_empty() {
                0
            } else {
                1
            })
        }
        _ => Err(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(list: &[&str]) -> (i32, String, String) {
        let args: Vec<String> = list.iter().map(|s| s.to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    fn history(extra: bool) -> String {
        let mut h = History::default();
        h.append("object-created", "c1", "added R1", None, Some("10k"))
            .unwrap();
        h.append(
            "component-moved",
            "c1",
            "moved R1",
            Some("0,0"),
            Some("5,5"),
        )
        .unwrap();
        if extra {
            h.append("rule-altered", "r1", "wider clearance", None, None)
                .unwrap();
        }
        h.to_json().unwrap()
    }

    #[test]
    fn check_log_and_compare_exit_codes() {
        let work = tempfile::tempdir().unwrap();
        let a = work.path().join("a.json");
        let b = work.path().join("b.json");
        let broken = work.path().join("broken.json");
        fs::write(&a, history(false)).unwrap();
        fs::write(&b, history(true)).unwrap();
        fs::write(&broken, history(false).replace("5,5", "6,6")).unwrap();
        let (a, b, broken) = (
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            broken.to_str().unwrap(),
        );
        assert_eq!(call(&["check", a]).0, 0);
        let (code, text, _) = call(&["check", broken]);
        assert_eq!(code, 1);
        assert!(text.contains("PHOENIX-DHIS-001"), "{text}");
        let (code, text, _) = call(&["log", a]);
        assert_eq!(code, 0);
        assert!(text.contains("component-moved"));
        assert_eq!(call(&["compare", a, a]).0, 0);
        let (code, text, _) = call(&["compare", a, b]);
        assert_eq!(code, 1);
        assert!(
            text.contains("common 2") && text.contains("only second: 3 rule-altered r1"),
            "{text}"
        );
    }

    #[test]
    fn bad_input_and_bad_arguments_exit_2() {
        let work = tempfile::tempdir().unwrap();
        let broken = work.path().join("broken.json");
        fs::write(&broken, "{").unwrap();
        let (code, _, err) = call(&["check", broken.to_str().unwrap()]);
        assert_eq!(code, 2);
        assert!(err.contains("cannot be read"));
        assert_eq!(call(&[]).0, 2);
        assert_eq!(call(&["compare", "only-one.json"]).0, 2);
        assert_eq!(call(&["check", "no-such-file.json"]).0, 2);
        assert_eq!(call(&["--config", "x"]).0, 2);
    }

    #[test]
    fn a_settings_file_changes_the_kinds_of_creation_and_deletion() {
        let work = tempfile::tempdir().unwrap();
        let h = work.path().join("h.json");
        let cfg = work.path().join("c.cfg");
        let mut history = History::default();
        history
            .append("part-removed", "c1", "", None, None)
            .unwrap();
        history
            .append("component-moved", "c1", "", None, None)
            .unwrap();
        fs::write(&h, history.to_json().unwrap()).unwrap();
        fs::write(&cfg, "delete_kind = part-removed\n").unwrap();
        assert_eq!(call(&["check", h.to_str().unwrap()]).0, 0);
        let (code, text, _) = call(&[
            "--config",
            cfg.to_str().unwrap(),
            "check",
            h.to_str().unwrap(),
        ]);
        assert_eq!(code, 1);
        assert!(text.contains("PHOENIX-DHIS-004"), "{text}");
    }
}
