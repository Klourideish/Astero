//! Explicit staging selection and read-only output. No guest entry is called.
use crate::{
    acquisition::{ArgumentError, limit},
    load_plan,
};
use astero_core::input::staging::{self, StagedGuestImage, StagingLimits};
use std::{ffi::OsString, fmt::Write, sync::Arc};
pub struct Request {
    pub plan: load_plan::Request,
    pub limits: StagingLimits,
}
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
    let mut args = args.into_iter();
    let mut lower = Vec::new();
    let mut maximum = None;
    while let Some(a) = args.next() {
        if a == "--max-mapped-bytes" {
            if maximum.is_some() {
                return Err(ArgumentError::Duplicate("--max-mapped-bytes"));
            }
            maximum = Some(limit(
                "--max-mapped-bytes",
                args.next()
                    .ok_or(ArgumentError::MissingValue("--max-mapped-bytes"))?,
            )?);
        } else {
            lower.push(a);
            if let Some(v) = args.next() {
                lower.push(v);
            }
        }
    }
    Ok(Request {
        plan: load_plan::parse(lower)?,
        limits: StagingLimits {
            max_mapped_bytes: maximum.ok_or(ArgumentError::Missing("--max-mapped-bytes"))?,
        },
    })
}
pub fn render(image: &StagedGuestImage) -> String {
    let mut out =
        String::from("STAGING RESULT\nBackend: owned bytes; native execution not enabled\n");
    let s = image.snapshot();
    let p = image.plan();
    writeln!(out,"Source: {:?} {:?}\nBias: {:#x} Entry: {:?}\nStatus: {:?}\nSegments: {} Mapped: {} Copied: {} Zero-filled: {}\nApplied relocations: {} Pending relocations: {} Unresolved references: {}\nProtections: {:?}\nReady for execution: false",p.headers().source().identity(),p.headers().source().provenance(),p.image_bias().0,p.entry(),s.status,s.mapped_segments,s.mapped_bytes,s.copied_bytes,s.zero_filled_bytes,s.applied_relocations,s.pending_relocations,s.unresolved_references,s.protections).unwrap();
    for seg in p.segments().iter().take(16) {
        writeln!(
            out,
            "range {:#x}..{:#x} intended {:?}",
            seg.mapping.range.start.0,
            seg.mapping.range.start.0 + seg.mapping.range.size,
            seg.mapping.permissions
        )
        .unwrap();
    }
    if p.segments().len() > 16 {
        writeln!(out, "{} ranges omitted", p.segments().len() - 16).unwrap();
    }
    for b in p.blockers().iter().take(16) {
        writeln!(out, "{:?}: {:?}", staging::classify_blocker(b), b).unwrap();
    }
    if p.blockers().len() > 16 {
        writeln!(out, "{} blockers omitted", p.blockers().len() - 16).unwrap();
    }
    out.push_str("NO GUEST CODE EXECUTED\n");
    out
}
pub fn execute(r: &Request) -> Result<String, String> {
    let plan = load_plan::execute(&r.plan).map_err(|e| format!("Planning: {e:?}"))?;
    let (image, observer) = staging::stage(Arc::new(plan), r.limits);
    let mut image = image.map_err(|e| {
        format!(
            "{e}; active mappings after failure: {}",
            observer.active_mappings()
        )
    })?;
    let mut text = render(&image);
    image.release();
    writeln!(
        text,
        "Teardown: {} active mappings",
        observer.active_mappings()
    )
    .unwrap();
    Ok(text)
}
