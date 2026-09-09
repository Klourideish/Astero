use super::Request;
use astero_core::input::acquisition::AcquisitionLimits;
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArgumentError {
    UnknownOption(OsString),
    Missing(&'static str),
    MissingValue(&'static str),
    Duplicate(&'static str),
    InvalidLimit {
        option: &'static str,
        value: OsString,
    },
}
impl std::fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ArgumentError {}

/// Only limits are text-decoded; the path stays native, including non-UTF-8 values.
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
    let mut args = args.into_iter();
    let (mut path, mut bytes, mut calls) = (None, None, None);
    while let Some(option) = args.next() {
        let name = if option == OsStr::new("--path") {
            "--path"
        } else if option == OsStr::new("--max-bytes") {
            "--max-bytes"
        } else if option == OsStr::new("--max-read-calls") {
            "--max-read-calls"
        } else {
            return Err(ArgumentError::UnknownOption(option));
        };
        let duplicate = match name {
            "--path" => path.is_some(),
            "--max-bytes" => bytes.is_some(),
            _ => calls.is_some(),
        };
        if duplicate {
            return Err(ArgumentError::Duplicate(name));
        }
        let value = args.next().ok_or(ArgumentError::MissingValue(name))?;
        match name {
            "--path" => path = Some(PathBuf::from(value)),
            "--max-bytes" => bytes = Some(limit(name, value)?),
            _ => calls = Some(limit(name, value)?),
        }
    }
    Ok(Request {
        path: path.ok_or(ArgumentError::Missing("--path"))?,
        limits: AcquisitionLimits {
            max_bytes: bytes.ok_or(ArgumentError::Missing("--max-bytes"))?,
            max_read_calls: calls.ok_or(ArgumentError::Missing("--max-read-calls"))?,
        },
    })
}
pub(crate) fn limit(option: &'static str, value: OsString) -> Result<u64, ArgumentError> {
    value
        .to_str()
        .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|s| s.parse().ok())
        .ok_or(ArgumentError::InvalidLimit { option, value })
}
