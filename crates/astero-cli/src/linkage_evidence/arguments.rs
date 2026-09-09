use crate::{
    acquisition::{self, ArgumentError},
    symbols,
};

use std::ffi::OsString;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub symbols: symbols::Request,
    pub max_relocations: u64,
}
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
    let mut args = args.into_iter();
    let mut lower = Vec::new();
    let mut maximum = None;
    while let Some(option) = args.next() {
        if option == "--max-relocations" {
            let name = "--max-relocations";
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
        max_relocations: maximum.ok_or(ArgumentError::Missing("--max-relocations"))?,
    })
}
