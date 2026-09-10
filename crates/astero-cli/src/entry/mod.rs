//! Explicit native-image/runtime preparation. Never enters guest code.
use crate::{
    acquisition::{ArgumentError, limit},
    native,
};
use std::ffi::OsString;
pub struct Request {
    pub close_entry: bool,
    pub native: native::Request,
    pub limits: astero_core::input::entry::RuntimeLimits,
}
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
    let mut close_entry = false;
    let mut a = args.into_iter();
    let mut lower = Vec::new();
    let mut values = [None; 4];
    let names = [
        "--stack-base",
        "--stack-bytes",
        "--tls-base",
        "--max-runtime-bytes",
    ];
    while let Some(o) = a.next() {
        if o == "--close-entry" {
            if close_entry {
                return Err(ArgumentError::Duplicate("--close-entry"));
            }
            close_entry = true;
            continue;
        }
        if let Some(i) = names.iter().position(|n| o == *n) {
            if values[i].is_some() {
                return Err(ArgumentError::Duplicate(names[i]));
            }
            values[i] = Some(limit(
                names[i],
                a.next().ok_or(ArgumentError::MissingValue(names[i]))?,
            )?);
        } else {
            lower.push(o);
            if let Some(v) = a.next() {
                lower.push(v)
            }
        }
    }
    let mut v = [0; 4];
    for i in 0..4 {
        v[i] = values[i].ok_or(ArgumentError::Missing(names[i]))?;
    }
    Ok(Request {
        close_entry,
        native: native::parse(lower)?,
        limits: astero_core::input::entry::RuntimeLimits {
            stack_base: v[0],
            stack_bytes: v[1],
            tls_base: v[2],
            max_runtime_bytes: v[3],
        },
    })
}
pub fn execute(r: &Request) -> Result<String, String> {
    #[cfg(all(windows, target_arch = "x86_64"))]
    {
        run_windows(r)
    }
    #[cfg(not(all(windows, target_arch = "x86_64")))]
    {
        let _ = r;
        Err("UnsupportedHost".into())
    }
}
#[cfg(all(windows, target_arch = "x86_64"))]
fn run_windows(r: &Request) -> Result<String, String> {
    use astero_core::input::{entry, native, staging};
    use std::{fmt::Write, sync::Arc};
    let plan =
        crate::load_plan::execute(&r.native.staging.plan).map_err(|e| format!("Plan: {e:?}"))?;
    let (s, bytes) = staging::stage(Arc::new(plan), r.native.staging.limits);
    let mut s = s.map_err(|e| e.to_string())?;
    let (n, image_observer) = native::realize(
        &s,
        native::NativeLimits {
            max_reserved_bytes: r.native.max_native_bytes,
            max_committed_bytes: r.native.max_native_bytes,
        },
    );
    let n = n.map_err(|e| e.to_string())?;
    let registry =
        entry::PreparedRegistry::new(Vec::new(), 0).map_err(|e| format!("Registry: {e:?}"))?;
    let (mut guest, obs) =
        entry::prepare(n, r.limits, registry, None).map_err(|e| e.to_string())?;
    let mut text = format!(
        "ENTRY READINESS / PRE-CLOSURE EVIDENCE\nArtifact: {:?}\nNative mapping: {:?}\nRaw entry: {:#x} Selected RIP: {:#x}\nContext RSP: {:#x} RDI: {:#x} FS planned: {:#x}\nStack: {:?} Guard: {:?}\nStack alignment: RSP modulo 16 = {}\nTLS: {:?}\nTCB: experimental self-pointer block; FS not activated\nProvider registrations: {} (no native installations)\nExternal references: {}\nEntry-critical conservative relocation count: {}\nException boundary: {:?}; native adapter NOT installed\nConstructors/init: {:?} (not invoked)\nProcparam: {:?}\nTiming injected: {}\nStatus: {:?} EntryReady: {}\n",
        guest.image().plan().headers().source().identity(),
        guest.image().state(),
        guest.bootstrap().raw_entry,
        guest.context().rip,
        guest.context().rsp,
        guest.context().gpr[5],
        guest.context().fs_base,
        guest.stack().stack,
        guest.stack().guard,
        guest.context().rsp % 16,
        guest.bootstrap().tls,
        guest.registry().len(),
        guest.image().plan().references().len(),
        guest.image().staging_snapshot().pending_relocations,
        guest.recovery(),
        guest.bootstrap().dynamic_init,
        guest.bootstrap().procparam,
        guest.has_timing(),
        guest.state(),
        guest.entry_ready()
    );
    for r in guest.relro() {
        writeln!(text, "RELRO: {r:?}").unwrap();
    }
    for b in guest.blockers().iter().take(32) {
        writeln!(text, "ENTRY BLOCKER: {b:?}").unwrap();
    }
    if guest.blockers().len() > 32 {
        writeln!(
            text,
            "{} blockers omitted; full API report retained",
            guest.blockers().len() - 32
        )
        .unwrap();
    }
    for b in guest.early_blockers() {
        writeln!(text, "EARLY RUNTIME: {b:?}").unwrap();
    }
    for r in guest.image().plan().references().iter().take(16) {
        writeln!(
            text,
            "reference symbol={} nid={:?} plan={:?}; not installed",
            r.symbol, r.nid, r.resolution
        )
        .unwrap();
    }
    if guest.image().plan().references().len() > 16 {
        writeln!(
            text,
            "{} references omitted",
            guest.image().plan().references().len() - 16
        )
        .unwrap();
    }
    for r in guest.image().plan().references().iter().take(2) {
        if let Some((nid, l, m)) = guest.image().plan().reference_identity(r.symbol) {
            let source = guest.image().plan().headers().source();
            writeln!(
                text,
                "startup identity nid={nid:#x} library={:?} module={:?}",
                source.read(&l),
                source.read(&m)
            )
            .unwrap();
        }
    }
    if r.close_entry {
        let closed = entry::close_entry(
            guest,
            r.limits.max_runtime_bytes,
            entry::StartupPolicy::ExperimentalEntryOwnedInit,
        )
        .map_err(|e| format!("Closure: {e:?}"))?;
        let report = closed.report();
        writeln!(text,"M30 CLOSURE: pending_before={} treated={} pending_after={} landings={} providers={} startup_matches={} bridge_validated={} RELRO finalized={} pending={} landing_bytes={} policy={:?}",report.pending_before,report.treated,report.pending_after,report.landings,report.providers,report.startup_matches,report.bridge_validated,report.relro_finalized,report.relro_pending,report.landing_bytes,report.policy).unwrap();
        writeln!(text,"Unresolved OBJECT traps: writes={} symbols={} bytes={}; RELRO inaccessible hole bytes={}",report.object_trap_writes,report.guarded_objects,report.object_trap_bytes,report.relro_hole_bytes).unwrap();
        writeln!(
            text,
            "Unresolved function landings: {}",
            report.unresolved_function_landings
        )
        .unwrap();
        for risk in &report.early_runtime {
            writeln!(text, "EARLY RUNTIME / EXPERIMENTAL: {risk}").unwrap();
        }
        let mut categories = std::collections::BTreeMap::new();
        for b in &report.remaining {
            if let entry::MustClose::PendingWrite { symbol, .. } = b {
                let ty = closed
                    .prepared()
                    .image()
                    .plan()
                    .input()
                    .linkage()
                    .symbols()
                    .entries()
                    .find(|(_, s)| s.fields().index == u64::from(*symbol))
                    .map(|(_, s)| format!("{:?}", s.fields().symbol_type))
                    .unwrap_or("Missing".into());
                *categories.entry(ty).or_insert(0usize) += 1;
            }
        }
        writeln!(text, "Pending write symbol categories: {categories:?}").unwrap();
        for b in report.remaining.iter().take(16) {
            writeln!(text, "MUST CLOSE: {b:?}").unwrap();
        }
        if report.remaining.len() > 16 {
            writeln!(
                text,
                "{} additional blockers retained in API",
                report.remaining.len() - 16
            )
            .unwrap();
        }
        writeln!(
            text,
            "EntryReady after closure: {}\nNO REAL GUEST ARTIFACT CODE EXECUTED",
            closed.entry_ready()
        )
        .unwrap();
        match closed.try_ready() {
            Ok(ready) => {
                writeln!(
                    text,
                    "EntryReadyGuest capability issued; no execution method invoked"
                )
                .unwrap();
                drop(ready);
            }
            Err(blocked) => {
                writeln!(text, "PreparedBlocked retained; no capability issued").unwrap();
                drop(blocked);
            }
        }
    } else {
        guest.release().map_err(|e| e.to_string())?;
    }
    s.release();
    writeln!(text,"Teardown: image={} stack={} tls={} byte mappings={}; release errors={}/{}/{}\nNO GUEST CODE EXECUTED",image_observer.active_reservations(),obs[0].active_reservations(),obs[1].active_reservations(),bytes.active_mappings(),image_observer.release_error(),obs[0].release_error(),obs[1].release_error()).unwrap();
    Ok(text)
}

pub mod first;
