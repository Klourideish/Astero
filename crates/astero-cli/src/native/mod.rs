//! Explicit M27-to-native realization. No entry point is invoked.
use crate::{
    acquisition::{ArgumentError, limit},
    staging,
};
#[cfg(all(windows, target_arch = "x86_64"))]
use astero_core::input::native::NativeLimits;
use std::ffi::OsString;
pub struct Request {
    pub staging: staging::Request,
    pub max_native_bytes: u64,
}
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
    let mut a = args.into_iter();
    let mut lower = Vec::new();
    let mut maximum = None;
    while let Some(o) = a.next() {
        if o == "--max-native-bytes" {
            if maximum.is_some() {
                return Err(ArgumentError::Duplicate("--max-native-bytes"));
            }
            maximum = Some(limit(
                "--max-native-bytes",
                a.next()
                    .ok_or(ArgumentError::MissingValue("--max-native-bytes"))?,
            )?);
        } else {
            lower.push(o);
            if let Some(v) = a.next() {
                lower.push(v);
            }
        }
    }
    Ok(Request {
        staging: staging::parse(lower)?,
        max_native_bytes: maximum.ok_or(ArgumentError::Missing("--max-native-bytes"))?,
    })
}
pub fn execute(r: &Request) -> Result<String, String> {
    #[cfg(all(windows, target_arch = "x86_64"))]
    {
        execute_windows(r)
    }
    #[cfg(not(all(windows, target_arch = "x86_64")))]
    {
        let _ = r;
        Err("Native VM: UnsupportedHost".into())
    }
}
#[cfg(all(windows, target_arch = "x86_64"))]
fn execute_windows(r: &Request) -> Result<String, String> {
    use astero_core::input::{native, staging as stage};
    use std::{fmt::Write, sync::Arc};
    let plan = crate::load_plan::execute(&r.staging.plan).map_err(|e| format!("Plan: {e:?}"))?;
    let (result, byte_observer) = stage::stage(Arc::new(plan), r.staging.limits);
    let mut staged = result.map_err(|e| e.to_string())?;
    let (result, observer) = native::realize(
        &staged,
        NativeLimits {
            max_reserved_bytes: r.max_native_bytes,
            max_committed_bytes: r.max_native_bytes,
        },
    );
    let mut image = result.map_err(|e| {
        format!(
            "{e}; active reservations: {}; release error: {}",
            observer.active_reservations(),
            observer.release_error()
        )
    })?;
    let n = image.snapshot();
    let s = image.staging_snapshot();
    let mut text = format!(
        "NATIVE VM RESULT\nArtifact: {:?}\nGuest envelope: {:?}\nHost envelope: {:?}\nCorrespondence: exact identity; no fallback\nPage size: {} Allocation granularity: {}\nSegments: {} Committed bytes: {} Materialized bytes: {}\nFile-backed bytes: {} Zero-fill bytes: {}\nApplied relocations: {} Pending relocations: {} Unresolved references: {}\nStatus: NativeBacked (pending/readiness evidence below)\nOS protections verified: {} Readback verified before protection: {} Instruction cache flushed: {}\nWidened pages: {}\nRELRO: deferred until final relocation/provider closure\nReady for execution: false\n",
        image.plan().headers().source().identity(),
        n.guest_envelope,
        n.host_envelope,
        n.geometry.page_size,
        n.geometry.allocation_granularity,
        s.mapped_segments,
        n.committed_bytes,
        n.copied_bytes,
        s.copied_bytes,
        s.zero_filled_bytes,
        s.applied_relocations,
        s.pending_relocations,
        s.unresolved_references,
        n.protections_verified,
        n.readback_before_protection,
        n.instruction_cache_flushed,
        n.widened_pages
    );
    writeln!(text, "Native lifecycle: {:?}", image.state()).unwrap();
    writeln!(
        text,
        "Pending work: {}",
        s.pending_relocations > 0
            || s.unresolved_references > 0
            || !image.plan().blockers().is_empty()
    )
    .unwrap();
    for seg in image.plan().segments().iter().take(16) {
        writeln!(
            text,
            "logical {:#x}..{:#x} requested {:?}",
            seg.mapping.range.start.0,
            seg.mapping.range.start.0 + seg.mapping.range.size,
            seg.mapping.permissions
        )
        .unwrap();
    }
    if let astero_core::input::inspection::InspectionOutcome::Complete(headers) =
        image.plan().headers().outcome()
    {
        for h in headers
            .program_headers()
            .iter()
            .filter(|h| h.kind == 0x6474e552)
            .take(16)
        {
            writeln!(
                text,
                "RELRO declared VA={:#x} size={} (retained; not finalized)",
                h.virtual_address, h.memory_size
            )
            .unwrap();
        }
    }
    for p in image.pages().iter().take(16) {
        writeln!(
            text,
            "page {:#x} effective {:?} widened={}",
            p.range.start.0, p.protection, p.widened
        )
        .unwrap();
    }
    if image.pages().len() > 16 {
        writeln!(
            text,
            "{} pages omitted; complete page evidence retained in API",
            image.pages().len() - 16
        )
        .unwrap();
    }
    image.release().map_err(|e| e.to_string())?;
    staged.release();
    writeln!(text,"Teardown: {} native reservations; {} byte mappings; release error={}\nNO GUEST CODE EXECUTED",observer.active_reservations(),byte_observer.active_mappings(),observer.release_error()).unwrap();
    Ok(text)
}
