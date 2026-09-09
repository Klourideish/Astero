use crate::acquisition::{self, ArgumentError};
use astero_core::input::inspection::InspectionLimits;
use std::ffi::OsString;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub acquisition: acquisition::Request,
    pub limits: InspectionLimits,
}
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
    let mut args = args.into_iter();
    let mut acquisition_args = Vec::new();
    let mut headers = None;
    while let Some(option) = args.next() {
        let name = match option.to_str() {
            Some("--path") => "--path",
            Some("--max-bytes") => "--max-bytes",
            Some("--max-read-calls") => "--max-read-calls",
            Some("--max-program-headers") => "--max-program-headers",
            _ => return Err(ArgumentError::UnknownOption(option)),
        };
        let value = args.next().ok_or(ArgumentError::MissingValue(name))?;
        if name == "--max-program-headers" {
            if headers.is_some() {
                return Err(ArgumentError::Duplicate(name));
            }
            headers = Some(acquisition::limit(name, value)?);
        } else {
            acquisition_args.extend([option, value]);
        }
    }
    Ok(Request {
        acquisition: acquisition::parse(acquisition_args)?,
        limits: InspectionLimits {
            max_program_headers: headers.ok_or(ArgumentError::Missing("--max-program-headers"))?,
        },
    })
}
