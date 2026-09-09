use crate::{
    acquisition::{self, ArgumentError},
    dynamic,
};
use astero_core::input::descriptors::DescriptorLimits;
use std::ffi::OsString;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub acquisition: acquisition::Request,
    pub limits: DescriptorLimits,
}
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
    let mut args = args.into_iter();
    let mut dynamic_args = Vec::new();
    let mut descriptors = None;
    while let Some(option) = args.next() {
        if option == "--max-descriptors" {
            let name = "--max-descriptors";
            if descriptors.is_some() {
                return Err(ArgumentError::Duplicate(name));
            }
            let value = args.next().ok_or(ArgumentError::MissingValue(name))?;
            descriptors = Some(acquisition::limit(name, value)?);
        } else {
            dynamic_args.push(option);
            if let Some(value) = args.next() {
                dynamic_args.push(value);
            }
        }
    }
    let request = dynamic::parse(dynamic_args)?;
    Ok(Request {
        acquisition: request.acquisition,
        limits: DescriptorLimits {
            dynamic: request.limits,
            max_descriptors: descriptors.ok_or(ArgumentError::Missing("--max-descriptors"))?,
        },
    })
}
