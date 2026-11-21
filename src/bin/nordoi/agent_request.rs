//! APP-004-R2 — bounded machine request JSON into the real NORDOI engine.
//!
//! Authorization is NEVER read from request JSON: the source root, entry
//! module, and FileWrite grant are independent explicit CLI arguments.
//! The untrusted request supplies only a schema and one u32 input value.
//! No external dependencies, no network, no inference runtime.
//!
//! Contract: one top-level JSON object (max 4096 bytes) with exactly
//! {"schema":"NORDOI_AGENT_REQUEST_V1","key_code":73}, in either key order,
//! permitting RFC 8259 JSON whitespace and valid string escapes.
//! Duplicate/unknown keys, wrong types, trailing bytes or malformed UTF-8
//! are rejected before any source loading, evaluation or filesystem effects.

use super::*;

const MAX_APP004_REQUEST_BYTES: u64 = 4096;
const REQUEST_SCHEMA_V1: &str = "NORDOI_AGENT_REQUEST_V1";

struct RequestCursor<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> RequestCursor<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn bytes(&self) -> &[u8] {
        self.input.as_bytes()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes().get(self.pos).copied()
    }

    fn whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }

    fn take(&mut self, wanted: u8) -> bool {
        if self.peek() == Some(wanted) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn hex4(&mut self) -> Result<u16, &'static str> {
        let mut number = 0u16;
        for _ in 0..4 {
            let b = self.peek().ok_or("INVALID_REQUEST_JSON")?;
            let digit = match b {
                b'0'..=b'9' => (b - b'0') as u16,
                b'a'..=b'f' => (b - b'a' + 10) as u16,
                b'A'..=b'F' => (b - b'A' + 10) as u16,
                _ => return Err("INVALID_REQUEST_JSON"),
            };
            number = (number << 4) | digit;
            self.pos += 1;
        }
        Ok(number)
    }

    fn quoted_string(&mut self) -> Result<String, &'static str> {
        if !self.take(b'"') {
            return Err("INVALID_REQUEST_JSON");
        }
        let mut result = String::new();
        loop {
            match self.peek().ok_or("INVALID_REQUEST_JSON")? {
                b'"' => {
                    self.pos += 1;
                    return Ok(result);
                }
                b'\\' => {
                    self.pos += 1;
                    let code = self.peek().ok_or("INVALID_REQUEST_JSON")?;
                    self.pos += 1;
                    match code {
                        b'"' => result.push('"'),
                        b'\\' => result.push('\\'),
                        b'/' => result.push('/'),
                        b'b' => result.push('\u{0008}'),
                        b'f' => result.push('\u{000c}'),
                        b'n' => result.push('\n'),
                        b'r' => result.push('\r'),
                        b't' => result.push('\t'),
                        b'u' => {
                            let mut point = self.hex4()? as u32;
                            if (0xd800..=0xdbff).contains(&point) {
                                if !self.take(b'\\') || !self.take(b'u') {
                                    return Err("INVALID_REQUEST_JSON");
                                }
                                let low = self.hex4()? as u32;
                                if !(0xdc00..=0xdfff).contains(&low) {
                                    return Err("INVALID_REQUEST_JSON");
                                }
                                point = 0x10000 + ((point - 0xd800) << 10) + (low - 0xdc00);
                            } else if (0xdc00..=0xdfff).contains(&point) {
                                return Err("INVALID_REQUEST_JSON");
                            }
                            result.push(char::from_u32(point).ok_or("INVALID_REQUEST_JSON")?);
                        }
                        _ => return Err("INVALID_REQUEST_JSON"),
                    }
                }
                0..=31 => return Err("INVALID_REQUEST_JSON"),
                _ => {
                    // UTF-8 was validated before parsing. Advance by the
                    // full Unicode scalar, not one byte of a multibyte char.
                    let character = self.input[self.pos..]
                        .chars()
                        .next()
                        .ok_or("INVALID_REQUEST_JSON")?;
                    result.push(character);
                    self.pos += character.len_utf8();
                }
            }
        }
    }

    fn u32_number(&mut self) -> Result<u32, &'static str> {
        let begin = self.pos;
        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => return Err("INVALID_REQUEST_KEY_CODE"),
        }
        self.input[begin..self.pos]
            .parse::<u32>()
            .map_err(|_| "INVALID_REQUEST_KEY_CODE")
    }
}

pub(super) fn parse_agent_request(bytes: &[u8]) -> Result<u32, &'static str> {
    if bytes.len() > MAX_APP004_REQUEST_BYTES as usize {
        return Err("REQUEST_TOO_LARGE");
    }
    let source = std::str::from_utf8(bytes).map_err(|_| "INVALID_REQUEST_UTF8")?;
    let mut p = RequestCursor::new(source);
    p.whitespace();
    if !p.take(b'{') {
        return Err("INVALID_REQUEST_JSON");
    }
    p.whitespace();
    let mut seen_schema = false;
    let mut score: Option<u32> = None;
    if p.take(b'}') {
        return Err("MISSING_REQUEST_FIELD");
    }
    loop {
        p.whitespace();
        let key = p.quoted_string()?;
        p.whitespace();
        if !p.take(b':') {
            return Err("INVALID_REQUEST_JSON");
        }
        p.whitespace();
        match key.as_str() {
            "schema" => {
                if seen_schema {
                    return Err("DUPLICATE_REQUEST_FIELD");
                }
                seen_schema = true;
                let schema = p.quoted_string()?;
                if schema != REQUEST_SCHEMA_V1 {
                    return Err("INVALID_REQUEST_SCHEMA");
                }
            }
            "key_code" => {
                if score.is_some() {
                    return Err("DUPLICATE_REQUEST_FIELD");
                }
                score = Some(p.u32_number()?);
            }
            _ => return Err("UNKNOWN_REQUEST_FIELD"),
        }
        p.whitespace();
        if p.take(b'}') {
            break;
        }
        if !p.take(b',') {
            return Err("INVALID_REQUEST_JSON");
        }
        p.whitespace();
        if p.peek() == Some(b'}') {
            return Err("INVALID_REQUEST_JSON");
        }
    }
    p.whitespace();
    if p.pos != p.bytes().len() {
        return Err("INVALID_REQUEST_JSON");
    }
    if !seen_schema || score.is_none() {
        return Err("MISSING_REQUEST_FIELD");
    }
    score.ok_or("MISSING_REQUEST_FIELD")
}

pub(super) fn run_agent_request_json_cli(
    source_root: &Path,
    entry_module: &str,
    grant: &Path,
) -> u8 {
    // Read only the bounded data request. No request key can grant FileWrite.
    let mut bytes = Vec::new();
    match io::stdin()
        .lock()
        .take(MAX_APP004_REQUEST_BYTES + 1)
        .read_to_end(&mut bytes)
    {
        Ok(_) => {}
        Err(_) => return app003::emit_app004_error(EXIT_IO, "REQUEST_STDIN_IO"),
    };
    let key_code = match parse_agent_request(&bytes) {
        Ok(value) => value,
        Err(reason) => return app003::emit_app004_error(EXIT_FRONTEND, reason),
    };
    app003::run_app004_agent_json_cli(source_root, entry_module, key_code, Some(grant))
}

#[cfg(test)]
mod tests {
    use super::parse_agent_request;

    #[test]
    fn parser_accepts_both_key_orders_and_json_whitespace() {
        assert_eq!(
            parse_agent_request(b"{\"schema\":\"NORDOI_AGENT_REQUEST_V1\",\"key_code\":73}"),
            Ok(73)
        );
        assert_eq!(
            parse_agent_request(b" \t{\n\"key_code\" : 59, \"schema\" : \"NORDOI_AGENT_REQUEST_V1\"}\r\n"),
            Ok(59)
        );
    }

    #[test]
    fn parser_rejects_duplicates_extra_keys_and_wrong_types() {
        for source in [
            r#"{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":73,"key_code":1}"#,
            r#"{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":73,"output_dir":"/tmp"}"#,
            r#"{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":"73"}"#,
            r#"{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":true}"#,
            r#"{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":-1}"#,
            r#"{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":73.0}"#,
            r#"{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":01}"#,
        ] {
            assert!(parse_agent_request(source.as_bytes()).is_err(), "{source}");
        }
    }

    #[test]
    fn parser_handles_escaped_key_and_prevents_trailing_bytes() {
        assert_eq!(
            parse_agent_request(br#"{"sche\u006da":"NORDOI_AGENT_REQUEST_V1","key_code":100}"#),
            Ok(100)
        );
        assert!(parse_agent_request(
            br#"{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":73}{}"#
        ).is_err());
        assert!(parse_agent_request(
            br#"{"schema":"NORDOI_AGENT_REQUEST_V1","key_code":73,}"#
        ).is_err());
    }

    #[test]
    fn parser_rejects_unpaired_surrogates_invalid_utf8_and_excess_size() {
        assert!(parse_agent_request(br#"{"schema":"\uD800","key_code":1}"#).is_err());
        assert!(parse_agent_request(&[0xff, 0xfe]).is_err());
        assert!(parse_agent_request(&vec![b' '; 4097]).is_err());
    }
}
