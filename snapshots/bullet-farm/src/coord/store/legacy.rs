use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom},
};

use crate::coord::{
    CoordError,
    model::{LEGACY_SCHEMA_VERSION, Record},
};

const MAX_RECORD_BYTES: usize = bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES;
const MAX_LOG_BYTES: u64 = 64 * 1024 * 1024;

pub(in crate::coord) fn read_records(file: &File) -> Result<Vec<Record>, CoordError> {
    let mut reader = file.try_clone().map_err(CoordError::io)?;
    reader.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    read_records_with_limits(&mut BufReader::new(reader), MAX_RECORD_BYTES, MAX_LOG_BYTES)
}

pub(in crate::coord) fn read_record_bytes(bytes: &[u8]) -> Result<Vec<Record>, CoordError> {
    if bytes.len() as u64 > MAX_LOG_BYTES {
        return Err(CoordError::new(
            "CORRUPT_COORD_LOG",
            format!("coordination log exceeds {MAX_LOG_BYTES} bytes"),
        ));
    }
    read_records_with_limits(
        &mut BufReader::new(Cursor::new(bytes)),
        MAX_RECORD_BYTES,
        MAX_LOG_BYTES,
    )
}

fn read_records_with_limits<R: BufRead>(
    reader: &mut R,
    max_record_bytes: usize,
    max_total_bytes: u64,
) -> Result<Vec<Record>, CoordError> {
    let mut records = Vec::new();
    let mut line = Vec::new();
    let mut total = 0_u64;
    loop {
        line.clear();
        let remaining = max_total_bytes.saturating_sub(total);
        let read_limit = (max_record_bytes as u64 + 2).min(remaining + 1);
        let read = Read::by_ref(reader)
            .take(read_limit)
            .read_until(b'\n', &mut line)
            .map_err(CoordError::io)?;
        if read == 0 {
            break;
        }
        total = total.saturating_add(read as u64);
        if total > max_total_bytes {
            return Err(CoordError::new(
                "CORRUPT_COORD_LOG",
                format!("coordination log exceeds {max_total_bytes} bytes"),
            ));
        }
        let index = records.len() + 1;
        if line.last() != Some(&b'\n') {
            return Err(CoordError::new(
                "CORRUPT_COORD_LOG",
                format!("line {index} has no final LF commit marker"),
            ));
        }
        line.pop();
        if line.last() == Some(&b'\r') {
            return Err(CoordError::new(
                "CORRUPT_COORD_LOG",
                format!("line {index} uses CRLF instead of its exact LF commit marker"),
            ));
        }
        if line.len() > max_record_bytes {
            return Err(CoordError::new(
                "CORRUPT_COORD_LOG",
                format!("line {index} exceeds {max_record_bytes} bytes"),
            ));
        }
        let value = bullet_wire::decode_unique_value(&line).map_err(|error| {
            CoordError::new(
                "CORRUPT_COORD_LOG",
                format!("line {index} is invalid strict JSON: {error}"),
            )
        })?;
        let record: Record = serde_json::from_value(value).map_err(|error| {
            CoordError::new(
                "CORRUPT_COORD_LOG",
                format!("line {index} does not match the coordination record schema: {error}"),
            )
        })?;
        if record.schema_version() != LEGACY_SCHEMA_VERSION {
            return Err(CoordError::new(
                "UNSUPPORTED_SCHEMA",
                format!("line {index} uses an unsupported schema"),
            ));
        }
        records.push(record);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Write};

    use super::read_records_with_limits;

    fn claim(at: &str, expires: &str) -> String {
        format!(
            r#"{{"kind":"claim","schema_version":1,"at_unix_ms":{at},"claim_id":"clm_test","agent":"test-agent","lane":"test-lane","repo":"bullet-farm","paths":["src"],"expires_unix_ms":{expires}}}"#
        )
    }

    fn ledger(bytes: &[u8]) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().expect("temporary ledger");
        file.write_all(bytes).expect("write ledger");
        file.flush().expect("flush ledger");
        file
    }

    #[test]
    fn refuses_unsafe_numbers_and_bounded_input() {
        let unsafe_number =
            ledger(format!("{}\n", claim("9007199254740992", "9007199254770992")).as_bytes());
        let error = read_records_with_limits(
            &mut BufReader::new(unsafe_number.reopen().expect("reopen ledger")),
            4096,
            4096,
        )
        .expect_err("an unsafe coordination integer must fail closed");
        assert_eq!(error.code(), "CORRUPT_COORD_LOG");
        assert!(error.to_string().contains("UNSAFE_JSON_INTEGER"));

        let valid = claim("1000", "31000");
        for (bytes, reason) in [
            (valid.as_bytes().to_vec(), "no final LF commit marker"),
            (format!("{valid}\r\n").into_bytes(), "uses CRLF"),
        ] {
            let hostile = ledger(&bytes);
            let error = read_records_with_limits(
                &mut BufReader::new(hostile.reopen().expect("reopen ledger")),
                4096,
                4096,
            )
            .expect_err("a non-LF coordination commit marker must fail closed");
            assert!(error.to_string().contains(reason), "{error}");
        }

        let per_line = ledger(format!("{valid}\n").as_bytes());
        let error = read_records_with_limits(
            &mut BufReader::new(per_line.reopen().expect("reopen ledger")),
            valid.len() - 1,
            4096,
        )
        .expect_err("an oversized line must fail closed");
        assert!(error.to_string().contains("line 1 exceeds"));

        let two_records = format!("{valid}\n{valid}\n");
        let over_total = ledger(two_records.as_bytes());
        let error = read_records_with_limits(
            &mut BufReader::new(over_total.reopen().expect("reopen ledger")),
            valid.len(),
            two_records.len() as u64 - 1,
        )
        .expect_err("an oversized ledger must fail closed");
        assert!(error.to_string().contains("coordination log exceeds"));
    }
}
