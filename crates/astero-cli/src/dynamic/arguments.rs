use crate::{
    acquisition::{self, ArgumentError},
    inspection,
};
use astero_core::input::dynamic::DynamicLimits;
use std::ffi::OsString;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub acquisition: acquisition::Request,
    pub limits: DynamicLimits,
}
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
    let mut args = args.into_iter();
    let mut header_args = Vec::new();
    let mut entries = None;
    while let Some(option) = args.next() {
        if option == "--max-dynamic-entries" {
            let name = "--max-dynamic-entries";
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
    let request = inspection::parse(header_args)?;
    Ok(Request {
        acquisition: request.acquisition,
        limits: DynamicLimits {
            max_program_headers: request.limits.max_program_headers,
            max_dynamic_entries: entries.ok_or(ArgumentError::Missing("--max-dynamic-entries"))?,
        },
    })
}
