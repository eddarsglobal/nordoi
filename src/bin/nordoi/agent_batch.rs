//! APP-005 — bounded native JSONL batch execution for AI agents.
//!
//! Real Rust NORDOI binary; actual V1.3 rule engine / P2.7 file publisher.
//! This is NOT a distributed transaction or a power-loss durable queue.
//! Every request is parsed and admitted BEFORE any output directory is
//! created. Every accepted job receives its own fixed, non-user-controlled
//! subdirectory beneath a separately granted existing output directory.
//! Success: one authentic APP-004 JSON receipt per input line, in order.
//! Failure: one native APP-004 JSON error line, nonzero exit. If a later
//! job fails, earlier published jobs are NOT rolled back; output is a
//! prefix of successful receipts followed by an error with
//! publication=UNKNOWN. Do not infer all-or-nothing from the stream.

use super::*;

const MAX_AGENT_BATCH_REQUESTS: usize = 8;
const MAX_AGENT_BATCH_LINE_BYTES: usize = 4096;
const MAX_AGENT_BATCH_INPUT_BYTES: usize =
    MAX_AGENT_BATCH_REQUESTS * (MAX_AGENT_BATCH_LINE_BYTES + 1);

pub(super) fn parse_agent_batch_lines(bytes: &[u8]) -> Result<Vec<u32>, &'static str> {
    if bytes.len() > MAX_AGENT_BATCH_INPUT_BYTES {
        return Err("AGENT_BATCH_TOO_LARGE");
    }
    if bytes.is_empty() {
        return Err("EMPTY_AGENT_BATCH");
    }

    let mut lines = bytes.split(|byte| *byte == b'\n').collect::<Vec<_>>();
    if bytes.last() == Some(&b'\n') {
        lines.pop();
    }
    if lines.is_empty() {
        return Err("EMPTY_AGENT_BATCH");
    }
    if lines.len() > MAX_AGENT_BATCH_REQUESTS {
        return Err("TOO_MANY_AGENT_REQUESTS");
    }

    let mut jobs = Vec::with_capacity(lines.len());
    for line in lines {
        if line.is_empty() {
            return Err("EMPTY_AGENT_BATCH_LINE");
        }
        if line.len() > MAX_AGENT_BATCH_LINE_BYTES {
            return Err("AGENT_BATCH_LINE_TOO_LARGE");
        }
        let key_code = agent_request::parse_agent_request(line)?;
        jobs.push(key_code);
    }
    Ok(jobs)
}

fn allowed_output_parent(granted: &Path) -> bool {
    // symlink_metadata does not dereference the final component. A direct
    // symlink to some other directory is not a permissible output parent.
    match fs::symlink_metadata(granted) {
        Ok(meta) => meta.is_dir() && !meta.file_type().is_symlink(),
        Err(_) => false,
    }
}

fn fresh_job_path(output_root: &Path, index: usize) -> PathBuf {
    // Only this Rust code constructs the child name: no attacker-controlled
    // JSON data or separators can influence a destination.
    output_root.join(format!("job-{index:04}"))
}

pub(super) fn run_agent_batch_jsonl_cli(
    source_root: &Path,
    entry_module: &str,
    granted_output_root: &Path,
) -> u8 {
    // The source root, module and grant are CLI parameters, never JSON data.
    // Reject unauthorized output roots before any effects.
    if !allowed_output_parent(granted_output_root) {
        return app003::emit_app004_error(EXIT_RUNTIME, "INVALID_EXPLICIT_BATCH_GRANT");
    }

    let mut input = Vec::new();
    match io::stdin()
        .lock()
        .take((MAX_AGENT_BATCH_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut input)
    {
        Ok(_) => {}
        Err(_) => return app003::emit_app004_error(EXIT_IO, "AGENT_BATCH_STDIN_IO"),
    }
    let jobs = match parse_agent_batch_lines(&input) {
        Ok(parsed) => parsed,
        Err(code) => return app003::emit_app004_error(EXIT_FRONTEND, code),
    };

    // Complete admission phase: no valid request may be acted upon if a
    // later request is malformed. Existing targets are checked first.
    for i in 1..=jobs.len() {
        let path = fresh_job_path(granted_output_root, i);
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            _ => return app003::emit_app004_error(EXIT_RUNTIME, "BATCH_TARGET_NOT_FRESH"),
        }
    }

    // Execution is intentionally sequential, without retries or rollback.
    // Atomic publication is per job through P2.7 only, NOT batch-wide.
    for (index, code) in jobs.into_iter().enumerate() {
        let output = fresh_job_path(granted_output_root, index + 1);
        if fs::create_dir(&output).is_err() {
            return app003::emit_app004_error(EXIT_IO, "BATCH_JOB_DIRECTORY_CREATE_DENIED");
        }
        let status = app003::run_app004_agent_json_cli(
            source_root,
            entry_module,
            code,
            Some(&output),
        );
        if status != EXIT_OK {
            return status;
        }
    }
    EXIT_OK
}

#[cfg(test)]
mod tests {
    use super::parse_agent_batch_lines;

    const A: &str = r#"{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":73}"#;
    const B: &str = r#"{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":59}"#;

    #[test]
    fn accepts_two_requests_and_preserves_order() {
        let payload = format!("{A}\n{B}\n");
        assert_eq!(parse_agent_batch_lines(payload.as_bytes()), Ok(vec![73, 59]));
    }

    #[test]
    fn accepts_one_request_without_final_newline() {
        assert_eq!(parse_agent_batch_lines(A.as_bytes()), Ok(vec![73]));
    }

    #[test]
    fn rejects_empty_double_newline_and_whole_batch_is_invalid() {
        assert!(parse_agent_batch_lines(b"").is_err());
        assert!(parse_agent_batch_lines(format!("{A}\n\n{B}").as_bytes()).is_err());
    }

    #[test]
    fn rejects_nine_request_lines_before_any_publication() {
        let payload = (0..9).map(|_| A).collect::<Vec<_>>().join("\n");
        assert_eq!(
            parse_agent_batch_lines(payload.as_bytes()),
            Err("TOO_MANY_AGENT_REQUESTS"),
        );
    }

    #[test]
    fn rejects_late_invalid_request_and_giant_input() {
        assert!(parse_agent_batch_lines(format!("{A}\n{{invalid}}").as_bytes()).is_err());
        assert_eq!(
            parse_agent_batch_lines(&vec![b'x'; 8 * 4097 + 1]),
            Err("AGENT_BATCH_TOO_LARGE"),
        );
    }
}
