use crate::{
    acquisition::{self, ArgumentError},
    dynamic,
};
use astero_core::input::hash_metadata::{HashLimits, HashMetadataLimits};
use std::ffi::OsString;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub acquisition: acquisition::Request,
    pub limits: HashMetadataLimits,
}
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
    let mut args = args.into_iter();
    let mut header_args = Vec::new();
    let mut entries = None;
    while let Some(option) = args.next() {
        if option == "--max-hash-words" {
            let name = "--max-hash-words";
            if entries.is_some() {
                return Err(ArgumentError::Duplicate(name));
            }
            let value = args.next().ok_or(ArgumentError::MissingValue(name))?;
            entries = Some(acquisition::limit(name, value)?);
        } else {
            header_args.push(option);
            if let Some(value) = args.next() {
                header_args.push(value);
            }
        }
    }
    let request = dynamic::parse(header_args)?;
    Ok(Request {
        acquisition: request.acquisition,
        limits: HashMetadataLimits {
            dynamic: request.limits,
            hash: HashLimits {
                max_words: entries.ok_or(ArgumentError::Missing("--max-hash-words"))?,
            },
        },
    })
}
