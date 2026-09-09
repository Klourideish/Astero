use crate::{
    acquisition::{self, ArgumentError},
    descriptors,
};
use astero_core::input::string_references::{StringLimits, StringReferenceLimits};
use std::ffi::OsString;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub acquisition: acquisition::Request,
    pub limits: StringReferenceLimits,
}
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
    let mut args = args.into_iter();
    let mut lower = Vec::new();
    let (mut references, mut per, mut total) = (None, None, None);
    while let Some(option) = args.next() {
        let selected = match option.to_str() {
            Some("--max-string-references") => Some((&mut references, "--max-string-references")),
            Some("--max-scan-bytes-per-reference") => {
                Some((&mut per, "--max-scan-bytes-per-reference"))
            }
            Some("--max-total-scan-bytes") => Some((&mut total, "--max-total-scan-bytes")),
            _ => None,
        };
        if let Some((slot, name)) = selected {
            if slot.is_some() {
                return Err(ArgumentError::Duplicate(name));
            }
            let value = args.next().ok_or(ArgumentError::MissingValue(name))?;
            *slot = Some(acquisition::limit(name, value)?);
        } else {
            lower.push(option);
            if let Some(value) = args.next() {
                lower.push(value);
            }
        }
    }
    let request = descriptors::parse(lower)?;
    Ok(Request {
        acquisition: request.acquisition,
        limits: StringReferenceLimits {
            descriptors: request.limits,
            max_string_references: references
                .ok_or(ArgumentError::Missing("--max-string-references"))?,
            strings: StringLimits {
                max_scan_bytes_per_reference: per
                    .ok_or(ArgumentError::Missing("--max-scan-bytes-per-reference"))?,
                max_total_scan_bytes: total
                    .ok_or(ArgumentError::Missing("--max-total-scan-bytes"))?,
            },
        },
    })
}
