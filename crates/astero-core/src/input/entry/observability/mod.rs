//! Read-only frontend observation. No execution authority and no retained guest resources after detach.
mod collect;
mod model;
use super::{ExecutionReport, workers::Runtime};
use astero_kernel::execution::host::sampling::{Domain, Sampler};
pub use model::*;
use std::sync::{Arc, Mutex, Weak};
struct State {
    report: Snapshot,
    runtime: Weak<Runtime>,
    started: Option<std::time::Instant>,
    sampler: Option<Arc<Sampler>>,
}
pub struct Observer {
    state: Mutex<State>,
    interval: Option<u64>,
}
impl Observer {
    pub fn new(artifact: String, interval: Option<u64>) -> Result<Arc<Self>, String> {
        if artifact.len() > 4096 || interval.is_some_and(|n| !(5..=1000).contains(&n)) {
            return Err("artifact label or sample interval out of bounds (5..=1000 ms)".into());
        }
        Ok(Arc::new(Self {
            state: Mutex::new(State {
                report: Snapshot::preparing(artifact),
                runtime: Weak::new(),
                started: None,
                sampler: None,
            }),
            interval,
        }))
    }
    pub fn sample_interval(&self) -> Option<u64> {
        self.interval
    }
    pub(crate) fn attach(
        &self,
        runtime: Option<&Arc<Runtime>>,
        report: Snapshot,
        sampler: Option<Arc<Sampler>>,
    ) {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        s.report = report;
        s.runtime = runtime.map_or_else(Weak::new, Arc::downgrade);
        s.sampler = sampler;
        s.started = Some(std::time::Instant::now());
    }
    fn refresh(s: &mut State) {
        if let Some(r) = s.runtime.upgrade() {
            collect::runtime(&r, &mut s.report);
        }
        if let Some(t) = s.started {
            s.report.elapsed_us = t.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        }
        if let Some(p) = &s.sampler
            && s.report.pc.enabled
        {
            collect::pc(p, &mut s.report.pc);
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if s.report.teardown.is_none() {
            Self::refresh(&mut s);
        }
        s.report.clone()
    }
    /// The lock serializes the final Weak upgrade with teardown: no frontend-held runtime Arc can escape.
    pub(crate) fn detach(&self) {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        Self::refresh(&mut s);
        s.runtime = Weak::new();
        s.started = None;
        s.report.runtime_state = "Stopping".into();
    }
    pub(crate) fn finish(&self, r: &ExecutionReport) {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        let f = &r.first;
        let n = &f.native;
        if let Some(p) = s.sampler.clone()
            && s.report.pc.enabled
        {
            collect::pc(&p, &mut s.report.pc);
        }
        let domain = if matches!(
            f.stop,
            super::FirstEntryStop::UnresolvedFunction
                | super::FirstEntryStop::ProviderRefused
                | super::FirstEntryStop::ProviderStopped
                | super::FirstEntryStop::StartupAllowanceExhausted
        ) {
            "RuntimeBridge".into()
        } else {
            s.sampler.as_ref().map_or_else(
                || "UnsampledBoundary".into(),
                |p| format!("{:?}", p.classify(n.rip).0),
            )
        };
        s.report.native_interval_us = n
            .supervision
            .as_ref()
            .map(|x| x.elapsed_micros.min(u128::from(u64::MAX)) as u64);
        s.report.runtime_state = "Stopped".into();
        s.report.elapsed_us = f.elapsed_micros.min(u128::from(u64::MAX)) as u64;
        s.report.faults = u64::from(n.exception != 0)
            + f.worker_exits
                .iter()
                .filter(|(_, n)| n.exception != 0)
                .count() as u64;
        s.report.supervisor_interventions =
            u64::from(n.supervision.as_ref().is_some_and(|x| x.redirected))
                + f.worker_exits
                    .iter()
                    .filter(|(_, n)| n.supervision.as_ref().is_some_and(|x| x.redirected))
                    .count() as u64;
        s.report.stop = Some(Stop {
            reason: format!("{:?}", f.stop),
            boundary_rip: address(n.rip),
            boundary_domain: domain,
            rsp: address(n.rsp),
            guest_continuation: (n.return_address != 0).then(|| address(n.return_address)),
            provider: f
                .unresolved
                .as_ref()
                .map(|k| ProviderIdentity::from(&collect::provider(k)))
                .or_else(|| {
                    (n.reason == 2)
                        .then(|| s.report.last_provider.clone())
                        .flatten()
                }),
            object: f.guarded_object.as_ref().map(|o| format!("{o:?}")),
            exception: n.exception,
            fault_address: (n.exception != 0).then(|| address(n.fault_address)),
            access_kind: (n.exception == 0xc0000005 && n.access_kind != u64::MAX)
                .then_some(n.access_kind),
            thread: n.supervision.as_ref().map(|x| x.thread_id),
            registers: (n.exception != 0 || n.supervision.as_ref().is_some_and(|s| s.redirected))
                .then(|| n.registers.iter().map(|n| address(*n)).collect()),
            supervisor_intervened: n.supervision.as_ref().is_some_and(|x| x.redirected),
        });
        s.report.latest_event = format!("Stopped: {:?}", f.stop);
        s.report.teardown = Some(Teardown {
            thread_joined: r.thread_joined,
            workers_joined: f.worker_exits.len(),
            host_fs_restored: n.host_fs_restored
                && f.worker_exits.iter().all(|(_, x)| x.host_fs_restored),
            host_gs_preserved: n.host_gs_preserved
                && f.worker_exits.iter().all(|(_, x)| x.host_gs_preserved),
            active_reservations: r.active_reservations,
            release_errors: r.release_errors.clone(),
            remaining_waiters: f.synchronization.as_ref().map_or(0, |x| x.waiters),
            remaining_sleep_tickets: f.guest_timing.as_ref().map_or(0, |x| x.pending),
            audio_stopped: f.audio.as_ref().is_some_and(|x| x.stopped),
            ajm_stopped: f.ajm.as_ref().is_some_and(|x| x.stopped),
        });
        if let Some(x) = &f.synchronization {
            s.report.shutdown_interruptions = x.shutdown_interruptions;
        }
    }
    pub(crate) fn source_verified(&self, hash: &str) {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        s.report.source_sha256_unchanged = s.report.sha256.as_deref().map(|before| before == hash);
    }
    pub(crate) fn failed(&self, error: &str) {
        self.detach();
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        s.report.runtime_state = "Failed".into();
        s.report.latest_event = error.chars().take(1024).collect();
    }
}
