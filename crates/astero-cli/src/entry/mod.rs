//! Explicit native-image/runtime preparation. Never enters guest code.
use crate::{
    acquisition::{ArgumentError, limit},
    native,
};
use std::ffi::OsString;
pub struct Request {
    pub native: native::Request,
    pub limits: astero_core::input::entry::RuntimeLimits,
}
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Request, ArgumentError> {
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
        "ENTRY READINESS\nArtifact: {:?}\nNative mapping: {:?}\nRaw entry: {:#x} Selected RIP: {:#x}\nContext RSP: {:#x} RDI: {:#x} FS planned: {:#x}\nStack: {:?} Guard: {:?}\nStack alignment: RSP modulo 16 = {}\nTLS: {:?}\nTCB: experimental self-pointer block; FS not activated\nProvider registrations: {} (no native installations)\nExternal references: {}\nEntry-critical conservative relocation count: {}\nException boundary: {:?}; native adapter NOT installed\nConstructors/init: {:?} (not invoked)\nProcparam: {:?}\nTiming injected: {}\nStatus: {:?} EntryReady: {}\n",
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
    guest.release().map_err(|e| e.to_string())?;
    s.release();
    writeln!(text,"Teardown: image={} stack={} tls={} byte mappings={}; release errors={}/{}/{}\nNO GUEST CODE EXECUTED",image_observer.active_reservations(),obs[0].active_reservations(),obs[1].active_reservations(),bytes.active_mappings(),image_observer.release_error(),obs[0].release_error(),obs[1].release_error()).unwrap();
    Ok(text)
}
