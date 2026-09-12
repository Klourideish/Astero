use super::*;
use astero_hle::dispatch::prepared::ProviderKey;
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn provider(k: &ProviderKey) -> Provider {
    Provider {
        nid: address(k.nid),
        library_bytes_hex: k.library.iter().map(|b| format!("{b:02x}")).collect(),
        module_bytes_hex: k.module.iter().map(|b| format!("{b:02x}")).collect(),
        library: String::from_utf8_lossy(&k.library).into_owned(),
        module: String::from_utf8_lossy(&k.module).into_owned(),
        ..Default::default()
    }
}
pub(super) fn subsystem(k: &ProviderKey) -> &'static str {
    if k.module == b"libkernel"
        && k.library == b"libkernel"
        && (astero_libs::pthread::exports::EXPORTS
            .iter()
            .any(|x| x.nid == k.nid)
            || astero_libs::pthread::thread::exports::EXPORTS
                .iter()
                .any(|x| x.nid == k.nid))
    {
        return "pthread";
    }
    if k.module == b"libc"
        && k.library == b"libc"
        && astero_libs::libc::c11::EXPORTS
            .iter()
            .any(|(_, n)| *n == k.nid)
    {
        return "C11 sync";
    }
    match k.module.as_slice() {
        b"libkernel" => "Kernel",
        b"libc" => "libc",
        b"libSceAjm" => "AJM",
        b"libSceAudioOut" => "Audio",
        b"libSceVideoOut" => "VideoOut",
        b"libSceAgcDriver" => "GPU/AGC",
        b"libSceSysmodule" => "Modules/sysmodules",
        b"libSceUserService" => "UserService",
        _ => "Unclassified",
    }
}
pub(super) fn runtime(r: &Runtime, s: &mut Snapshot) {
    use astero_kernel::{synchronization::owned::Kind, threading::thread::lifecycle::State};
    let sync = r.synchronization.snapshot();
    let sema = r.semaphores.snapshot();
    let resources = r.memory_resources.snapshot();
    s.kernel_resources = KernelResources {
        direct_allocations: resources.allocations.len(),
        direct_bytes: resources.allocations.iter().map(|r| r.size).sum(),
        mappings: resources.mappings.len(),
        allocations_total: resources.allocated_total,
        mappings_total: resources.mapped_total,
        semaphores: sema.objects.len(),
        semaphore_waiters: sema.waiting_threads.len(),
        waits: sema.waits,
        signals: sema.signals,
        timeouts: sema.timeouts,
        cancellations: sema.cancellations,
    };
    let timing = r.guest_timing.snapshot();
    let threads = r.table.snapshot();
    let waiting: BTreeSet<_> = sync
        .waiting_threads
        .iter()
        .map(|t| t.0)
        .chain(timing.sleeping.iter().map(|(t, _)| t.0))
        .chain(sema.waiting_threads.iter().map(|t| t.0))
        .collect();
    s.threads_created = threads.len();
    s.live_threads = threads
        .iter()
        .filter(|t| matches!(t.state, State::Running | State::Created))
        .count();
    s.threads_peak = r.table.peak_threads();
    s.waiting_threads = waiting.len();
    s.running_or_host_threads = s.live_threads.saturating_sub(s.waiting_threads);
    let memory = r.memory_observation();
    s.mapped_runtime_bytes_excluding_landings = memory.0;
    s.stack_tls_mappings = memory.1;
    s.mutexes = sync
        .objects
        .iter()
        .filter(|o| o.kind == Kind::Mutex)
        .count();
    s.rwlocks = sync
        .objects
        .iter()
        .filter(|o| o.kind == Kind::Rwlock)
        .count();
    s.condvars = sync.objects.iter().filter(|o| o.kind == Kind::Cond).count();
    s.waiters = sync.waiters;
    s.waits = sync.waits;
    s.wakes = sync.wakes;
    s.timeouts = sync.timeouts;
    s.shutdown_interruptions = sync.shutdown_interruptions;
    let heap = r
        .foundation
        .heap
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .snapshot();
    s.heap_live = heap.live_bytes;
    s.heap_peak = heap.peak_bytes;
    s.allocations_live = heap.live;
    s.allocations_peak = heap.peak_live;
    providers(r.metrics.snapshot(), s);
    let audio = r.audio.snapshot();
    s.audio_ports = audio
        .objects
        .iter()
        .filter(|o| {
            matches!(
                o.kind,
                astero_audio::output::service::Kind::Port
                    | astero_audio::output::service::Kind::Legacy
            )
        })
        .count();
    s.audio_buffers = audio.submitted;
    s.audio_bytes = audio.bytes;
    s.audio_pending = audio.objects.iter().map(|o| o.pending).sum();
    if audio.initialized {
        let h = s.subsystems.get_mut("Audio").unwrap();
        if h.state != Health::Blocked {
            h.state = if audio.submitted > 0 {
                Health::Active
            } else {
                Health::Initialized
            };
            h.detail = format!(
                "{}; ports={}, buffers={}",
                audio.backend, s.audio_ports, s.audio_buffers
            );
        }
    }
    let ajm = r.ajm.snapshot();
    s.ajm_contexts = ajm.contexts;
    if ajm.contexts > 0 {
        let h = s.subsystems.get_mut("AJM").unwrap();
        if h.state != Health::Blocked {
            h.state = Health::Partial;
            h.detail = format!(
                "contexts={}, modules={}, instances={}; decode unavailable",
                ajm.contexts, ajm.modules, ajm.instances
            );
        }
    }
}
pub(super) fn pc(p: &Sampler, s: &mut Pc) {
    let snap = p.snapshot();
    s.enabled = true;
    s.interval_ms = Some(p.interval_ms());
    s.total = snap.total;
    s.dropped = snap.dropped;
    s.suspends = snap.suspends;
    s.resumes = snap.resumes;
    let guest: Vec<_> = snap
        .samples
        .iter()
        .filter(|x| x.domain == Domain::GuestImage)
        .collect();
    s.guest_samples = guest.len() as u64;
    s.last_guest_rip = guest.last().map(|x| address(x.rip));
    s.unique_guest_rips = guest.iter().map(|x| x.rip).collect::<BTreeSet<_>>().len();
    s.unique_guest_pages = guest
        .iter()
        .map(|x| x.rip / 4096)
        .collect::<BTreeSet<_>>()
        .len();
    s.guest_rips = guest
        .iter()
        .map(|x| address(x.rip))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    s.guest_page_addresses = guest
        .iter()
        .map(|x| address(x.rip / 4096 * 4096))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    s.thread_samples.clear();
    for x in &snap.samples {
        *s.thread_samples.entry(x.thread).or_default() += 1;
    }
    s.distribution.clear();
    for x in &snap.samples {
        *s.distribution
            .entry(format!("{:?}/source={:?}", x.domain, x.source))
            .or_default() += 1;
    }
    s.last_rip = snap.samples.last().map(|x| address(x.rip));
    s.last_domain = snap.samples.last().map(|x| format!("{:?}", x.domain));
}

fn providers(counts: astero_hle::calls::metrics::Snapshot, s: &mut Snapshot) {
    let mut merged: BTreeMap<(u64, Vec<u8>, Vec<u8>), Provider> = BTreeMap::new();
    for c in &counts.entries {
        if let Some(k) = &c.key {
            let entry = merged
                .entry((k.nid, k.library.clone(), k.module.clone()))
                .or_insert_with(|| provider(k));
            entry.calls += c.calls;
            entry.returned += c.returned;
            entry.unknown += c.unknown;
            entry.refused += c.refused;
            let name = subsystem(k);
            let h = s
                .subsystems
                .entry(name.into())
                .or_insert_with(|| Subsystem {
                    state: Health::NotReached,
                    detail: String::new(),
                });
            if c.unknown > 0 || c.refused > 0 {
                h.state = Health::Blocked;
                h.detail = format!("Unresolved/refused {}", address(k.nid));
            } else if c.returned > 0 && h.state != Health::Blocked {
                h.state = Health::Active;
                h.detail = "Provider return observed; subsystem completeness not implied".into();
            } else if h.state == Health::NotReached {
                h.state = Health::Present;
                h.detail = "Provider entered; return not yet observed".into();
            }
        }
    }
    s.providers = merged.into_values().collect();
    s.hle_calls = counts.entries.iter().map(|e| e.calls).sum::<u64>() + counts.invalid_ordinals;
    s.unique_providers = s.providers.len();
    s.returned_providers = s.providers.iter().filter(|p| p.returned > 0).count();
    s.unknown_calls =
        counts.entries.iter().map(|e| e.unknown).sum::<u64>() + counts.invalid_ordinals;
    s.refused_calls = counts.entries.iter().map(|e| e.refused).sum();
    s.last_provider = counts
        .last_ordinal
        .and_then(|i| counts.entries.iter().find(|e| e.ordinal == i))
        .and_then(|c| c.key.as_ref())
        .map(|k| ProviderIdentity::from(&provider(k)));
    if let Some(p) = &s.last_provider {
        s.latest_event = format!("Thread {:?}: {} / {}", counts.last_thread, p.module, p.nid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn subsystem_mapping_never_assumes_rows_and_counts_exact_keys() {
        let key = ProviderKey {
            nid: 1,
            library: b"new library".to_vec(),
            module: b"new module".to_vec(),
        };
        let metrics = astero_hle::calls::metrics::Metrics::new(vec![Some(key)]).unwrap();
        metrics.begin(0, 1);
        metrics.complete(
            0,
            Err(astero_hle::dispatch::prepared::RegistryError::Missing),
        );
        let mut s = Snapshot::preparing("fixture".into());
        s.subsystems.clear();
        providers(metrics.snapshot(), &mut s);
        assert_eq!(s.subsystems["Unclassified"].state, Health::Blocked);
        assert_eq!(s.unknown_calls, 1);
        assert_eq!(s.unique_providers, 1);
        assert_eq!(s.returned_providers, 0);
    }
    #[test]
    fn returned_and_unknown_are_distinct_health_states() {
        let key = ProviderKey {
            nid: 1,
            library: b"libc".to_vec(),
            module: b"libc".to_vec(),
        };
        let m = astero_hle::calls::metrics::Metrics::new(vec![Some(key)]).unwrap();
        let mut s = Snapshot::preparing("x".into());
        m.begin(0, 1);
        providers(m.snapshot(), &mut s);
        assert_eq!(s.subsystems["libc"].state, Health::Present);
        m.complete(0, Ok(astero_hle::dispatch::prepared::CallResult::Returned));
        providers(m.snapshot(), &mut s);
        assert_eq!(s.subsystems["libc"].state, Health::Active);
        m.begin(0, 1);
        m.complete(
            0,
            Err(astero_hle::dispatch::prepared::RegistryError::Missing),
        );
        providers(m.snapshot(), &mut s);
        assert_eq!(s.subsystems["libc"].state, Health::Blocked);
    }
}
