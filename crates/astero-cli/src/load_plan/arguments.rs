use crate::{
    acquisition::{self, ArgumentError},
    ps5_identity,
};
use astero_core::input::load_plan::{PlanningLimits, VirtualAddress};
use std::{ffi::OsString, path::PathBuf};
#[derive(Clone, Debug)]
pub struct Request {
    pub identity: ps5_identity::Request,
    pub providers: Vec<(PathBuf, Option<Vec<u8>>)>,
    pub bias: VirtualAddress,
    pub limits: PlanningLimits,
}
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
    let mut args = args.into_iter();
    let mut lower = Vec::new();
    let mut providers: Vec<(PathBuf, Option<Vec<u8>>)> = Vec::new();
    let (mut bias, mut maximum, mut records) = (None, None, None);
    while let Some(o) = args.next() {
        if o == "--provider" {
            providers.push((
                args.next()
                    .ok_or(ArgumentError::MissingValue("--provider"))?
                    .into(),
                None,
            ));
        } else if o == "--provider-alias" {
            let a = args
                .next()
                .ok_or(ArgumentError::MissingValue("--provider-alias"))?;
            let last = providers
                .last_mut()
                .ok_or(ArgumentError::Missing("--provider"))?;
            if last.1.is_some() {
                return Err(ArgumentError::Duplicate("--provider-alias"));
            }
            last.1 = Some(
                a.to_str()
                    .ok_or_else(|| ArgumentError::UnknownOption(a.clone()))?
                    .as_bytes()
                    .to_vec(),
            );
        } else if o == "--image-bias" || o == "--max-providers" || o == "--max-plan-records" {
            let (name, slot) = if o == "--image-bias" {
                ("--image-bias", &mut bias)
            } else if o == "--max-providers" {
                ("--max-providers", &mut maximum)
            } else {
                ("--max-plan-records", &mut records)
            };
            if slot.is_some() {
                return Err(ArgumentError::Duplicate(name));
            }
            *slot = Some(acquisition::limit(
                name,
                args.next().ok_or(ArgumentError::MissingValue(name))?,
            )?);
        } else {
            lower.push(o);
            if let Some(v) = args.next() {
                lower.push(v)
            }
        }
    }
    Ok(Request {
        identity: ps5_identity::parse(lower)?,
        providers,
        bias: VirtualAddress(bias.ok_or(ArgumentError::Missing("--image-bias"))?),
        limits: PlanningLimits {
            max_providers: maximum.ok_or(ArgumentError::Missing("--max-providers"))?,
            max_plan_records: records.ok_or(ArgumentError::Missing("--max-plan-records"))?,
        },
    })
}
