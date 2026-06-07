//! Local crash-report directory support.
//!
//! This is the local directory/listing and envelope persistence foundation from
//! upstream `crash/dir.zig`, `crash/sentry_envelope.zig`, and the local
//! transport path in `crash/sentry.zig`. It does not initialize Sentry or
//! capture crash envelopes.

use std::cmp::Ordering;
use std::ffi::OsString;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde_json::{Map, Value};
use uuid::Uuid;

const ROASTTY_BUNDLE_ID: &str = "com.termsurf.roastty";
const CRASH_REPORT_EXTENSION: &str = "roasttycrash";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Report {
    pub name: OsString,
    pub modified: SystemTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CrashDir {
    path: PathBuf,
}

impl CrashDir {
    pub(crate) fn new(path: impl Into<PathBuf>) -> CrashDir {
        CrashDir { path: path.into() }
    }

    pub(crate) fn default() -> CrashDir {
        CrashDir::new(default_dir_path())
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn reports(&self) -> std::io::Result<Vec<Report>> {
        let entries = match std::fs::read_dir(&self.path) {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err),
        };

        let mut reports = Vec::new();
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if !file_type.is_file() {
                continue;
            }

            let metadata = entry.metadata()?;
            reports.push(Report {
                name: entry.file_name(),
                modified: metadata.modified()?,
            });
        }

        reports.sort_by(|lhs, rhs| report_order(lhs, rhs));
        Ok(reports)
    }

    pub(crate) fn persist_event_envelope(
        &self,
        bytes: &[u8],
    ) -> Result<PersistOutcome, PersistError> {
        let envelope = Envelope::parse(bytes)?;
        if !envelope.has_event() {
            return Ok(PersistOutcome::Discarded);
        }

        let name = generated_report_name();
        self.write_report(bytes, &name)
    }

    pub(crate) fn persist_event_envelope_with_name(
        &self,
        bytes: &[u8],
        name: &str,
    ) -> Result<PersistOutcome, PersistError> {
        let envelope = Envelope::parse(bytes)?;
        if !envelope.has_event() {
            return Ok(PersistOutcome::Discarded);
        }

        self.write_report(bytes, name)
    }

    fn write_report(&self, bytes: &[u8], name: &str) -> Result<PersistOutcome, PersistError> {
        let report_path = self.report_path_for_name(name)?;
        std::fs::create_dir_all(&self.path)?;
        std::fs::write(&report_path, bytes)?;

        Ok(PersistOutcome::Written(report_path))
    }

    fn report_path_for_name(&self, name: &str) -> Result<PathBuf, PersistError> {
        if !valid_report_name(name) {
            return Err(PersistError::InvalidReportName);
        }

        Ok(self.path.join(name))
    }
}

pub(crate) fn default_dir_path() -> PathBuf {
    default_dir_path_from_home(std::env::var_os("HOME"))
}

fn default_dir_path_from_home(home: Option<OsString>) -> PathBuf {
    if let Some(home) = home.filter(|value| !value.is_empty()) {
        return PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join(ROASTTY_BUNDLE_ID)
            .join("crash");
    }

    std::env::temp_dir().join("roastty").join("crash")
}

fn report_order(lhs: &Report, rhs: &Report) -> Ordering {
    rhs.modified
        .cmp(&lhs.modified)
        .then_with(|| lhs.name.cmp(&rhs.name))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PersistOutcome {
    Written(PathBuf),
    Discarded,
}

#[derive(Debug)]
pub(crate) enum PersistError {
    Envelope(EnvelopeError),
    Io(std::io::Error),
    InvalidReportName,
}

impl From<EnvelopeError> for PersistError {
    fn from(err: EnvelopeError) -> PersistError {
        PersistError::Envelope(err)
    }
}

impl From<std::io::Error> for PersistError {
    fn from(err: std::io::Error) -> PersistError {
        PersistError::Io(err)
    }
}

fn generated_report_name() -> String {
    format!("{}.{}", Uuid::new_v4(), CRASH_REPORT_EXTENSION)
}

fn valid_report_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut components = Path::new(name).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Envelope {
    pub headers: Map<String, Value>,
    pub items: Vec<EnvelopeItem>,
}

impl Envelope {
    pub(crate) fn parse(bytes: &[u8]) -> Result<Envelope, EnvelopeError> {
        let mut parser = EnvelopeParser::new(bytes);
        let headers = parser.parse_envelope_headers()?;
        let mut items = Vec::new();

        while let Some(item) = parser.parse_item()? {
            items.push(item);
        }

        Ok(Envelope { headers, items })
    }

    pub(crate) fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_json_object(&mut out, &self.headers);

        for item in &self.items {
            out.push(b'\n');
            let encoded = item.encoded();
            write_json_object(&mut out, &encoded.headers);
            out.push(b'\n');
            out.extend_from_slice(&encoded.payload);
        }

        out
    }

    fn has_event(&self) -> bool {
        self.items
            .iter()
            .any(|item| matches!(item.item_type(), ItemType::Event))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EnvelopeItem {
    Encoded(EncodedItem),
}

impl EnvelopeItem {
    pub(crate) fn encoded(&self) -> &EncodedItem {
        match self {
            EnvelopeItem::Encoded(item) => item,
        }
    }

    pub(crate) fn item_type(&self) -> &ItemType {
        &self.encoded().item_type
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EncodedItem {
    pub headers: Map<String, Value>,
    pub item_type: ItemType,
    pub payload: Vec<u8>,
}

impl EncodedItem {
    pub(crate) fn decode_attachment(&self) -> Result<Attachment, AttachmentDecodeError> {
        if !matches!(self.item_type, ItemType::Attachment) {
            return Err(AttachmentDecodeError::UnsupportedType);
        }

        let filename = match self.headers.get("filename") {
            Some(Value::String(value)) => value.clone(),
            Some(_) => return Err(AttachmentDecodeError::InvalidFieldType),
            None => return Err(AttachmentDecodeError::MissingRequiredField),
        };
        let attachment_type = match self.headers.get("attachment_type") {
            Some(Value::String(value)) => Some(value.clone()),
            Some(_) => return Err(AttachmentDecodeError::InvalidFieldType),
            None => None,
        };

        Ok(Attachment {
            filename,
            attachment_type,
            payload: self.payload.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Attachment {
    pub filename: String,
    pub attachment_type: Option<String>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ItemType {
    Unknown(String),
    Event,
    Transaction,
    Attachment,
    Session,
    Sessions,
    Statsd,
    MetricMeta,
    UserFeedback,
    ClientReport,
    ReplayEvent,
    ReplayRecording,
    Profile,
    CheckIn,
}

impl ItemType {
    fn from_header(value: &str) -> ItemType {
        match value {
            "event" => ItemType::Event,
            "transaction" => ItemType::Transaction,
            "attachment" => ItemType::Attachment,
            "session" => ItemType::Session,
            "sessions" => ItemType::Sessions,
            "statsd" => ItemType::Statsd,
            "metric_meta" => ItemType::MetricMeta,
            "user_feedback" => ItemType::UserFeedback,
            "client_report" => ItemType::ClientReport,
            "replay_event" => ItemType::ReplayEvent,
            "replay_recording" => ItemType::ReplayRecording,
            "profile" => ItemType::Profile,
            "check_in" => ItemType::CheckIn,
            other => ItemType::Unknown(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EnvelopeError {
    MalformedHeaders,
    ItemMalformedHeaders,
    ItemTypeMissing,
    ItemLengthMalformed,
    ItemPayloadTooShort,
    ItemPayloadNoNewline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AttachmentDecodeError {
    MissingRequiredField,
    InvalidFieldType,
    UnsupportedType,
}

struct EnvelopeParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> EnvelopeParser<'a> {
    fn new(bytes: &'a [u8]) -> EnvelopeParser<'a> {
        EnvelopeParser { bytes, pos: 0 }
    }

    fn parse_envelope_headers(&mut self) -> Result<Map<String, Value>, EnvelopeError> {
        let line = self.take_line().ok_or(EnvelopeError::MalformedHeaders)?;
        parse_json_object(line).map_err(|_| EnvelopeError::MalformedHeaders)
    }

    fn parse_item(&mut self) -> Result<Option<EnvelopeItem>, EnvelopeError> {
        let Some(line) = self.take_line() else {
            return Ok(None);
        };
        if trim_ascii(line).is_empty() {
            return Ok(None);
        }

        let headers = parse_json_object(line).map_err(|_| EnvelopeError::ItemMalformedHeaders)?;
        let item_type = match headers.get("type") {
            Some(Value::String(value)) => ItemType::from_header(value),
            Some(_) | None => return Err(EnvelopeError::ItemTypeMissing),
        };
        let length = match headers.get("length") {
            Some(Value::Number(number)) => {
                Some(number.as_u64().ok_or(EnvelopeError::ItemLengthMalformed)?)
            }
            Some(_) => return Err(EnvelopeError::ItemLengthMalformed),
            None => None,
        };

        let payload = if let Some(length) = length {
            let len = usize::try_from(length).map_err(|_| EnvelopeError::ItemLengthMalformed)?;
            self.take_exact_payload(len)?
        } else {
            self.take_line()
                .map(|line| line.to_vec())
                .ok_or(EnvelopeError::ItemPayloadTooShort)?
        };

        Ok(Some(EnvelopeItem::Encoded(EncodedItem {
            headers,
            item_type,
            payload,
        })))
    }

    fn take_line(&mut self) -> Option<&'a [u8]> {
        if self.pos >= self.bytes.len() {
            return None;
        }

        let start = self.pos;
        match self.bytes[start..].iter().position(|&byte| byte == b'\n') {
            Some(offset) => {
                let end = start + offset;
                self.pos = end + 1;
                Some(&self.bytes[start..end])
            }
            None => {
                self.pos = self.bytes.len();
                Some(&self.bytes[start..])
            }
        }
    }

    fn take_exact_payload(&mut self, len: usize) -> Result<Vec<u8>, EnvelopeError> {
        if self.bytes.len().saturating_sub(self.pos) < len {
            return Err(EnvelopeError::ItemPayloadTooShort);
        }

        let payload = self.bytes[self.pos..self.pos + len].to_vec();
        self.pos += len;

        if self.pos < self.bytes.len() {
            if self.bytes[self.pos] != b'\n' {
                return Err(EnvelopeError::ItemPayloadNoNewline);
            }
            self.pos += 1;
        }

        Ok(payload)
    }
}

fn parse_json_object(line: &[u8]) -> Result<Map<String, Value>, ()> {
    match serde_json::from_slice::<Value>(line).map_err(|_| ())? {
        Value::Object(object) => Ok(object),
        _ => Err(()),
    }
}

fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map(|index| index + 1)
        .unwrap_or(start);
    &bytes[start..end]
}

fn write_json_object(out: &mut Vec<u8>, object: &Map<String, Value>) {
    let value = Value::Object(object.clone());
    out.extend_from_slice(
        serde_json::to_string(&value)
            .expect("JSON serialization")
            .as_bytes(),
    );
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::thread;
    use std::time::Duration;

    use super::*;
    use crate::os::temp_dir::TempDir;

    fn names(reports: &[Report]) -> Vec<String> {
        reports
            .iter()
            .map(|report| report.name.to_string_lossy().into_owned())
            .collect()
    }

    fn event_envelope() -> &'static [u8] {
        br#"{}
{"type":"event","length":16}
{"crashed":true}"#
    }

    #[test]
    fn crash_default_path_uses_bundle_id_application_support_with_home() {
        let path = default_dir_path_from_home(Some(OsString::from("/Users/tester")));
        assert_eq!(
            path,
            PathBuf::from("/Users/tester")
                .join("Library")
                .join("Application Support")
                .join("com.termsurf.roastty")
                .join("crash")
        );
    }

    #[test]
    fn crash_default_path_falls_back_to_scoped_temp_subdir_without_home() {
        let path = default_dir_path_from_home(None);
        assert_eq!(path, std::env::temp_dir().join("roastty").join("crash"));
    }

    #[test]
    fn crash_reports_missing_directory_is_empty() {
        let temp = TempDir::new().unwrap();
        let dir = CrashDir::new(temp.path().join("missing"));
        assert!(dir.reports().unwrap().is_empty());
    }

    #[test]
    fn crash_reports_filter_non_files_and_return_basenames() {
        let temp = TempDir::new().unwrap();
        let dir_path = temp.path().join("crash");
        fs::create_dir(&dir_path).unwrap();
        File::create(dir_path.join("a.ghosttycrash")).unwrap();
        fs::create_dir(dir_path.join("nested")).unwrap();

        let dir = CrashDir::new(&dir_path);
        assert_eq!(dir.path(), dir_path.as_path());
        let reports = dir.reports().unwrap();
        assert_eq!(names(&reports), vec!["a.ghosttycrash"]);
    }

    #[test]
    fn crash_reports_sort_newest_first_with_name_tiebreak() {
        let temp = TempDir::new().unwrap();
        let dir_path = temp.path().join("crash");
        fs::create_dir(&dir_path).unwrap();

        File::create(dir_path.join("old.ghosttycrash")).unwrap();
        thread::sleep(Duration::from_millis(20));
        File::create(dir_path.join("new.ghosttycrash")).unwrap();

        let reports = CrashDir::new(&dir_path).reports().unwrap();
        assert_eq!(
            names(&reports),
            vec!["new.ghosttycrash", "old.ghosttycrash"]
        );

        let tied = [
            Report {
                name: "b.ghosttycrash".into(),
                modified: SystemTime::UNIX_EPOCH,
            },
            Report {
                name: "a.ghosttycrash".into(),
                modified: SystemTime::UNIX_EPOCH,
            },
        ];
        let mut tied = tied.to_vec();
        tied.sort_by(report_order);
        assert_eq!(names(&tied), vec!["a.ghosttycrash", "b.ghosttycrash"]);
    }

    #[test]
    fn crash_persist_event_envelope_writes_exact_bytes() {
        let temp = TempDir::new().unwrap();
        let dir = CrashDir::new(temp.path().join("crash"));
        let bytes = event_envelope();

        let outcome = dir
            .persist_event_envelope_with_name(bytes, "fixed.roasttycrash")
            .unwrap();

        let PersistOutcome::Written(path) = outcome else {
            panic!("expected written report");
        };
        assert_eq!(path, dir.path().join("fixed.roasttycrash"));
        assert_eq!(fs::read(path).unwrap(), bytes);
        assert_eq!(names(&dir.reports().unwrap()), vec!["fixed.roasttycrash"]);
    }

    #[test]
    fn crash_persist_generated_name_uses_roastty_extension() {
        let temp = TempDir::new().unwrap();
        let dir = CrashDir::new(temp.path().join("crash"));

        let outcome = dir.persist_event_envelope(event_envelope()).unwrap();

        let PersistOutcome::Written(path) = outcome else {
            panic!("expected written report");
        };
        assert_eq!(
            path.extension().and_then(|extension| extension.to_str()),
            Some(CRASH_REPORT_EXTENSION)
        );
        assert!(path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap()
            .ends_with(".roasttycrash"));
        assert_eq!(fs::read(path).unwrap(), event_envelope());
    }

    #[test]
    fn crash_persist_discards_non_event_envelopes_before_creating_directory() {
        for bytes in [
            br#"{}
{"type":"session"}
{}"# as &[u8],
            br#"{}
{"type":"attachment","length":4,"filename":"test.txt"}
ABCD"#,
        ] {
            let temp = TempDir::new().unwrap();
            let dir = CrashDir::new(temp.path().join("crash"));

            let outcome = dir
                .persist_event_envelope_with_name(bytes, "discarded.roasttycrash")
                .unwrap();

            assert_eq!(outcome, PersistOutcome::Discarded);
            assert!(!dir.path().exists());
        }
    }

    #[test]
    fn crash_persist_malformed_envelope_errors_before_creating_directory() {
        let temp = TempDir::new().unwrap();
        let dir = CrashDir::new(temp.path().join("crash"));

        let err = dir
            .persist_event_envelope_with_name(b"[]", "bad.roasttycrash")
            .unwrap_err();

        assert!(matches!(
            err,
            PersistError::Envelope(EnvelopeError::MalformedHeaders)
        ));
        assert!(!dir.path().exists());
    }

    #[test]
    fn crash_persist_rejects_report_names_that_escape_directory() {
        for name in ["", "../escape.roasttycrash", "nested/escape.roasttycrash"] {
            let temp = TempDir::new().unwrap();
            let dir = CrashDir::new(temp.path().join("crash"));

            let err = dir
                .persist_event_envelope_with_name(event_envelope(), name)
                .unwrap_err();

            assert!(matches!(err, PersistError::InvalidReportName));
            assert!(!dir.path().exists());
        }
    }

    #[test]
    fn envelope_parse_empty() {
        let envelope = Envelope::parse(br#"{}"#).unwrap();
        assert!(envelope.headers.is_empty());
        assert!(envelope.items.is_empty());
    }

    #[test]
    fn envelope_parse_session_and_multiple_items() {
        let envelope = Envelope::parse(
            br#"{}
{"type":"session","length":218}
{"init":true,"sid":"c148cc2f-5f9f-4231-575c-2e85504d6434","status":"abnormal","errors":0,"started":"2024-08-29T02:38:57.607016Z","duration":0.000343,"attrs":{"release":"0.1.0-HEAD+d37b7d09","environment":"production"}}
{"type":"attachment","length":4,"filename":"test.txt"}
ABCD"#,
        )
        .unwrap();

        assert_eq!(envelope.items.len(), 2);
        assert_eq!(envelope.items[0].item_type(), &ItemType::Session);
        assert_eq!(envelope.items[1].item_type(), &ItemType::Attachment);
        assert_eq!(envelope.items[1].encoded().payload, b"ABCD");
    }

    #[test]
    fn envelope_parse_no_length_payload_and_trailing_newline() {
        let envelope = Envelope::parse(
            br#"{}
{"type":"session"}
{}
"#,
        )
        .unwrap();

        assert_eq!(envelope.items.len(), 1);
        assert_eq!(envelope.items[0].item_type(), &ItemType::Session);
        assert_eq!(envelope.items[0].encoded().payload, b"{}");
    }

    #[test]
    fn envelope_parse_no_length_payload_preserves_carriage_return() {
        let envelope = Envelope::parse(b"{}\n{\"type\":\"session\"}\nABC\r\n").unwrap();

        assert_eq!(envelope.items.len(), 1);
        assert_eq!(envelope.items[0].item_type(), &ItemType::Session);
        assert_eq!(envelope.items[0].encoded().payload, b"ABC\r");
    }

    #[test]
    fn envelope_parse_exact_length_payload_accepts_eof_without_newline() {
        let envelope = Envelope::parse(
            br#"{}
{"type":"attachment","length":4,"filename":"test.txt"}
ABCD"#,
        )
        .unwrap();

        assert_eq!(envelope.items.len(), 1);
        assert_eq!(envelope.items[0].encoded().payload, b"ABCD");
    }

    #[test]
    fn envelope_parse_unknown_type_preserves_encoded_item() {
        let envelope = Envelope::parse(
            br#"{}
{"type":"future_item","length":2}
OK"#,
        )
        .unwrap();

        assert_eq!(
            envelope.items[0].item_type(),
            &ItemType::Unknown("future_item".to_string())
        );
        let serialized = envelope.serialize();
        let reparsed = Envelope::parse(&serialized).unwrap();
        assert_eq!(reparsed.items[0].item_type(), envelope.items[0].item_type());
        assert_eq!(reparsed.items[0].encoded().payload, b"OK");
    }

    #[test]
    fn envelope_attachment_decode_validates_headers() {
        let envelope = Envelope::parse(
            br#"{}
{"type":"attachment","length":4,"filename":"test.txt","attachment_type":"event.attachment"}
ABCD"#,
        )
        .unwrap();
        let attachment = envelope.items[0].encoded().decode_attachment().unwrap();
        assert_eq!(attachment.filename, "test.txt");
        assert_eq!(
            attachment.attachment_type.as_deref(),
            Some("event.attachment")
        );
        assert_eq!(attachment.payload, b"ABCD");

        let missing = Envelope::parse(
            br#"{}
{"type":"attachment","length":4}
ABCD"#,
        )
        .unwrap();
        assert_eq!(
            missing.items[0].encoded().decode_attachment(),
            Err(AttachmentDecodeError::MissingRequiredField)
        );

        let non_string_filename = Envelope::parse(
            br#"{}
{"type":"attachment","length":4,"filename":7}
ABCD"#,
        )
        .unwrap();
        assert_eq!(
            non_string_filename.items[0].encoded().decode_attachment(),
            Err(AttachmentDecodeError::InvalidFieldType)
        );

        let non_string_type = Envelope::parse(
            br#"{}
{"type":"attachment","length":4,"filename":"test.txt","attachment_type":7}
ABCD"#,
        )
        .unwrap();
        assert_eq!(
            non_string_type.items[0].encoded().decode_attachment(),
            Err(AttachmentDecodeError::InvalidFieldType)
        );

        let session = Envelope::parse(
            br#"{}
{"type":"session","length":2}
{}"#,
        )
        .unwrap();
        assert_eq!(
            session.items[0].encoded().decode_attachment(),
            Err(AttachmentDecodeError::UnsupportedType)
        );
    }

    #[test]
    fn envelope_serialize_round_trips_items() {
        let envelope = Envelope::parse(
            br#"{"dsn":"local"}
{"type":"session","length":2}
{}
{"type":"attachment","length":4,"filename":"test.txt"}
ABCD"#,
        )
        .unwrap();

        let serialized = envelope.serialize();
        assert!(!serialized.windows(2).any(|window| window == b"\n\n"));
        let reparsed = Envelope::parse(&serialized).unwrap();
        assert_eq!(reparsed.headers, envelope.headers);
        assert_eq!(reparsed.items, envelope.items);
    }

    #[test]
    fn envelope_parse_rejects_malformed_inputs() {
        assert_eq!(Envelope::parse(b"[]"), Err(EnvelopeError::MalformedHeaders));
        assert_eq!(
            Envelope::parse(
                br#"{}
[]"#
            ),
            Err(EnvelopeError::ItemMalformedHeaders)
        );
        assert_eq!(
            Envelope::parse(
                br#"{}
{"length":1}
X"#
            ),
            Err(EnvelopeError::ItemTypeMissing)
        );
        assert_eq!(
            Envelope::parse(
                br#"{}
{"type":"session","length":"1"}
X"#
            ),
            Err(EnvelopeError::ItemLengthMalformed)
        );
        assert_eq!(
            Envelope::parse(
                br#"{}
{"type":"session","length":4}
AB"#
            ),
            Err(EnvelopeError::ItemPayloadTooShort)
        );
        assert_eq!(
            Envelope::parse(
                br#"{}
{"type":"session","length":2}
ABX"#
            ),
            Err(EnvelopeError::ItemPayloadNoNewline)
        );
        assert_eq!(
            Envelope::parse(
                br#"{}
{"type":"session","length":18446744073709551615}
ABCD"#
            ),
            Err(EnvelopeError::ItemPayloadTooShort)
        );
    }
}
