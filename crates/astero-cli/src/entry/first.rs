//! Explicit first-entry command, isolated worker process and core-owned execution thread.
use std::ffi::OsString;
pub fn run(args: Vec<OsString>, worker: bool) -> Result<(), String> {
    #[cfg(all(windows, target_arch = "x86_64"))]
    {
        run_windows(args, worker)
    }
    #[cfg(not(all(windows, target_arch = "x86_64")))]
    {
        let _ = (args, worker);
        Err("UnsupportedHost".into())
    }
}
#[cfg(all(windows, target_arch = "x86_64"))]
fn run_windows(args: Vec<OsString>, worker: bool) -> Result<(), String> {
    use astero_core::input::{entry, native, staging};
    use std::{io::Write, sync::Arc};
    let mut a = args.iter().cloned();
    let mut lower = Vec::new();
    let mut wall = None;
    let mut outer = None;
    while let Some(k) = a.next() {
        if k == "--wall-ms" || k == "--containment-ms" {
            let slot = if k == "--wall-ms" {
                &mut wall
            } else {
                &mut outer
            };
            if slot.is_some() {
                return Err("Duplicate execution limit".into());
            }
            *slot = Some(
                a.next()
                    .ok_or("Missing execution limit")?
                    .to_str()
                    .ok_or("Invalid execution limit")?
                    .parse::<u64>()
                    .map_err(|_| "Invalid execution limit")?,
            );
        } else {
            lower.push(k);
        }
    }
    let wall = wall.ok_or("--wall-ms required")?;
    let outer = outer.ok_or("--containment-ms required")?;
    if !(1..=500).contains(&wall) || !(1000..=30000).contains(&outer) {
        return Err("wall-ms must be 1..=500; containment-ms 1000..=30000".into());
    }
    lower.push("--close-entry".into());
    let r = super::parse(lower).map_err(|e| e.to_string())?;
    if !worker {
        println!(
            "REAL GUEST CODE WILL EXECUTE; trusted experimental input, NOT A SECURITY SANDBOX"
        );
        std::io::stdout().flush().map_err(|e| e.to_string())?;
        let child = std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
            .arg("_first-entry-worker")
            .args(args)
            .env("ASTERO_FIRST_ENTRY_WORKER", "1")
            .spawn()
            .map_err(|e| e.to_string())?;
        let result = entry::contain_worker(child, outer)?;
        println!("Process containment: {result:?}");
        return if result == entry::ContainmentExit::Clean {
            Ok(())
        } else {
            Err("Worker failed containment; NOT clean native recovery".into())
        };
    }
    if std::env::var_os("ASTERO_FIRST_ENTRY_WORKER").as_deref() != Some(std::ffi::OsStr::new("1")) {
        return Err("Internal worker requires containment parent".into());
    }
    let report = entry::execute_first_entry(
        move || {
            let plan = crate::load_plan::execute(&r.native.staging.plan)
                .map_err(|e| format!("Plan: {e:?}"))?;
            let (staged, _) = staging::stage(Arc::new(plan), r.native.staging.limits);
            let staged = staged.map_err(|e| e.to_string())?;
            let (image, _) = native::realize(
                &staged,
                native::NativeLimits {
                    max_reserved_bytes: r.native.max_native_bytes,
                    max_committed_bytes: r.native.max_native_bytes,
                },
            );
            let (guest, _) = entry::prepare(
                image.map_err(|e| e.to_string())?,
                r.limits,
                entry::PreparedRegistry::new(vec![], 0).unwrap(),
                None,
            )
            .map_err(|e| e.to_string())?;
            let closed = entry::close_startup(
                guest,
                r.limits.max_runtime_bytes,
                entry::StartupPolicy::ExperimentalEntryOwnedInit,
            )
            .map_err(|e| format!("Closure: {e:?}"))?;
            let ready = closed
                .try_ready()
                .map_err(|_| "PreparedBlocked; no execution".to_string())?;
            println!("EntryReady verified; arming {wall} ms execution lease; entry owns DT_INIT");
            std::io::stdout().flush().map_err(|e| e.to_string())?;
            Ok(ready)
        },
        wall,
    )?;
    println!("{}", render(&report));
    if !report.thread_joined || report.active_reservations != 0 || !report.release_errors.is_empty()
    {
        return Err("Execution teardown incomplete".into());
    }
    println!(
        "REAL GUEST CODE EXECUTED; ASTERO REGAINED CONTROL; guest thread joined; native/runtime reservations=0"
    );
    Ok(())
}

#[cfg(all(windows, target_arch = "x86_64"))]
fn render(r: &astero_core::input::entry::ExecutionReport) -> String {
    use std::fmt::Write;
    let f = &r.first;
    let n = &f.native;
    let mut s = format!(
        "FIRST NATIVE GUEST ENTRY\nSource: {:?}\nInitial RIP={:#x} RSP={:#x} FS={:#x}\nStop: {:?}; elapsed={} us\nCaptured/boundary RIP={:#x} RSP={:#x}; separate last import return address={:#x}\nException={:#x} access-kind={} fault-address={:#x}\nRegisters (full on fault/preemption): {:x?}\nLast import arguments: {:x?}; ordinal={}\nReturn RAX={:#x} XMM0={:x?}\nHost FS restored={} GS preserved={}\nSupervisor: {:?}\n",
        f.source,
        f.initial.rip,
        f.initial.rsp,
        f.initial.fs_base,
        f.stop,
        f.elapsed_micros,
        n.rip,
        n.rsp,
        n.return_address,
        n.exception,
        n.access_kind,
        n.fault_address,
        n.registers,
        n.arguments,
        n.import_ordinal,
        n.value,
        n.xmm0,
        n.host_fs_restored,
        n.host_gs_preserved,
        n.supervision
    );
    writeln!(
        s,
        "Startup residency: {:?}; heap: {:?}",
        f.data_export, f.heap
    )
    .unwrap();
    writeln!(
        s,
        "Provider failure: {:?}; registry failure: {:?}",
        f.provider_failure, f.registry_failure
    )
    .unwrap();
    for c in &f.startup_calls {
        writeln!(
            s,
            "Startup NID={:#x} library={:?} module={:?} args={:x?} returned={:#x}",
            c.key.nid, c.key.library, c.key.module, c.arguments, c.returned
        )
        .unwrap();
    }
    writeln!(s, "Synchronization: {:?}", f.synchronization).unwrap();
    writeln!(s,"Unresolved key: {:?}\nGuarded object: {:?}; associated relocation count={}\nRetained callbacks={} (not invoked)\nThread joined={} native/runtime reservations={} release errors={:?}",f.unresolved,f.guarded_object,f.guarded_relocations.len(),f.callback_count,r.thread_joined,r.active_reservations,r.release_errors).unwrap();
    s
}
