use crate::{
    acquisition::{self, ArgumentError},
    linkage_evidence,
};

use std::ffi::OsString;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub linkage: linkage_evidence::Request,
    pub max_identity_records: u64,
}
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
    let mut args = args.into_iter();
    let mut lower = Vec::new();
    let mut maximum = None;
    while let Some(option) = args.next() {
        if option == "--max-identity-records" {
            let name = "--max-identity-records";
            if maximum.is_some() {
                return Err(ArgumentError::Duplicate(name));
            }
            maximum = Some(acquisition::limit(
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
    Ok(Request {
        linkage: linkage_evidence::parse(lower)?,
        max_identity_records: maximum.ok_or(ArgumentError::Missing("--max-identity-records"))?,
    })
}
