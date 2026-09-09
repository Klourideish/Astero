use crate::{
    acquisition::{self, ArgumentError},
    symbols,
};
use astero_core::input::classification::ClassificationLimits;
use std::ffi::OsString;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub symbols: symbols::Request,
    pub limits: ClassificationLimits,
}
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
    let mut args = args.into_iter();
    let mut lower = Vec::new();
    let mut maximum = None;
    while let Some(option) = args.next() {
        if option == "--max-classifications" {
            let name = "--max-classifications";
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
        symbols: symbols::parse(lower)?,
        limits: ClassificationLimits {
            max_classifications: maximum.ok_or(ArgumentError::Missing("--max-classifications"))?,
        },
    })
}
