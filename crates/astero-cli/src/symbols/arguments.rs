use crate::{
    acquisition::{self, ArgumentError},
    hash_metadata,
};
use astero_core::input::{hash_metadata::HashMetadataLimits, symbols::SymbolObservationLimits};
use std::ffi::OsString;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub acquisition: acquisition::Request,
    pub hash: HashMetadataLimits,
    pub symbols: SymbolObservationLimits,
}
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
    let mut args = args.into_iter();
    let mut lower = Vec::new();
    let (mut descriptors, mut symbols, mut lookups, mut per, mut total) =
        (None, None, None, None, None);
    while let Some(option) = args.next() {
        let selected = match option.to_str() {
            Some("--max-descriptors") => Some((&mut descriptors, "--max-descriptors")),
            Some("--max-symbols") => Some((&mut symbols, "--max-symbols")),
            Some("--max-name-lookups") => Some((&mut lookups, "--max-name-lookups")),
            Some("--max-name-scan-bytes") => Some((&mut per, "--max-name-scan-bytes")),
            Some("--max-total-name-scan-bytes") => {
                Some((&mut total, "--max-total-name-scan-bytes"))
            }
            _ => None,
        };
        if let Some((slot, name)) = selected {
            if slot.is_some() {
                return Err(ArgumentError::Duplicate(name));
            }
            *slot = Some(acquisition::limit(
                name,
                args.next().ok_or(ArgumentError::MissingValue(name))?,
            )?);
        } else {
            lower.push(option);
            if let Some(value) = args.next() {
                lower.push(value);
            }
        }
    }
    let lower = hash_metadata::parse(lower)?;
    Ok(Request {
        acquisition: lower.acquisition,
        hash: lower.limits,
        symbols: SymbolObservationLimits {
            max_descriptors: descriptors.ok_or(ArgumentError::Missing("--max-descriptors"))?,
            max_symbols: symbols.ok_or(ArgumentError::Missing("--max-symbols"))?,
            max_name_lookups: lookups.ok_or(ArgumentError::Missing("--max-name-lookups"))?,
            max_name_scan_bytes: per.ok_or(ArgumentError::Missing("--max-name-scan-bytes"))?,
            max_total_name_scan_bytes: total
                .ok_or(ArgumentError::Missing("--max-total-name-scan-bytes"))?,
        },
    })
}
