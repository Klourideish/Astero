//! Terminal presentation only. Core snapshots are also the machine-readable report.
use astero_core::input::entry::observability::{Observer, Snapshot};
use std::{
    ffi::OsString,
    io::{IsTerminal, Write},
    path::PathBuf,
    sync::{Arc, Condvar, Mutex},
};
#[derive(Default)]
pub struct Options {
    pub verbose: bool,
    pub trace: bool,
    pub no_dashboard: bool,
    pub sample: Option<u64>,
    pub log: Option<PathBuf>,
    pub report: Option<PathBuf>,
}
impl Options {
    pub fn take(
        &mut self,
        k: &OsString,
        args: &mut impl Iterator<Item = OsString>,
    ) -> Result<bool, String> {
        match k.to_str() {
            Some("--verbose") => self.verbose = true,
            Some("--trace") => self.trace = true,
            Some("--no-dashboard") => self.no_dashboard = true,
            Some("--pc-sample-ms") => {
                if self.sample.is_some() {
                    return Err("Duplicate pc-sample-ms".into());
                }
                let n = args
                    .next()
                    .ok_or("Missing pc-sample-ms")?
                    .to_str()
                    .ok_or("Invalid pc-sample-ms")?
                    .parse::<u64>()
                    .map_err(|_| "Invalid pc-sample-ms")?;
                if !(5..=1000).contains(&n) {
                    return Err("pc-sample-ms must be 5..=1000".into());
                }
                self.sample = Some(n);
            }
            Some("--log-file") | Some("--report-json") => {
                let slot = if k == "--log-file" {
                    &mut self.log
                } else {
                    &mut self.report
                };
                if slot.is_some() {
                    return Err("Duplicate output path".into());
                }
                *slot = Some(args.next().ok_or("Missing output path")?.into());
            }
            _ => return Ok(false),
        };
        Ok(true)
    }
    pub fn interactive(&self, tty: bool, term: Option<&str>) -> bool {
        tty && term != Some("dumb") && !self.no_dashboard && !self.trace
    }
    pub fn files(&self) -> Result<(Option<std::fs::File>, Option<std::fs::File>), String> {
        fn open(p: &Option<PathBuf>) -> Result<Option<std::fs::File>, String> {
            p.as_ref()
                .map(|p| {
                    std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(p)
                        .map_err(|e| {
                            format!("Output {}: {e}; output files must be new", p.display())
                        })
                })
                .transpose()
        }
        Ok((open(&self.log)?, open(&self.report)?))
    }
}
fn clean(s: &str, max: usize) -> String {
    s.chars().filter(|c| !c.is_control()).take(max).collect()
}
fn shown(s: &Option<String>) -> &str {
    s.as_deref().unwrap_or("not observed")
}
pub fn render(s: &Snapshot, verbose: bool) -> String {
    use std::fmt::Write;
    let mut out = format!(
        "ASTERO | RUNTIME DASHBOARD | {}\nArtifact: {}\nSHA256: {} | entry {} | bias {}\nElapsed: {:.3} ms | image {} bytes | modules {}\n",
        s.runtime_state,
        clean(&s.artifact, 100),
        s.sha256
            .as_deref()
            .map_or("pending", |s| &s[..s.len().min(16)]),
        shown(&s.entry_rip),
        shown(&s.load_bias),
        s.elapsed_us as f64 / 1000.0,
        s.mapped_image_bytes,
        s.guest_modules_observed
    );
    writeln!(
        out,
        "Threads: created {} (incl. main), peak {}, live {}, waiting {}, running/host {}",
        s.threads_created,
        s.threads_peak,
        s.live_threads,
        s.waiting_threads,
        s.running_or_host_threads
    )
    .unwrap();
    writeln!(
        out,
        "PC: {} [{}] | samples {}, guest {}, unique RIPs {}, 4KiB pages {}",
        shown(&s.pc.last_rip),
        s.pc.last_domain.as_deref().unwrap_or("sampling off"),
        s.pc.total,
        s.pc.guest_samples,
        s.pc.unique_guest_rips,
        s.pc.unique_guest_pages
    )
    .unwrap();
    writeln!(
        out,
        "HLE: registered {} + {} data | calls {} | used {} | returned {} | unknown {} | refused {}",
        s.registered_callable_identities,
        s.data_exports,
        s.hle_calls,
        s.unique_providers,
        s.returned_providers,
        s.unknown_calls,
        s.refused_calls
    )
    .unwrap();
    writeln!(
        out,
        "Memory: heap live/peak {}/{} B | allocations {}/{} peak | stack/TLS {}",
        s.heap_live, s.heap_peak, s.allocations_live, s.allocations_peak, s.stack_tls_mappings
    )
    .unwrap();
    writeln!(
        out,
        "Sync: mutex {} rwlock {} cond {} | waiters {} | waits/wakes/timeouts {}/{}/{}",
        s.mutexes, s.rwlocks, s.condvars, s.waiters, s.waits, s.wakes, s.timeouts
    )
    .unwrap();
    writeln!(
        out,
        "Faults {} | supervisor interventions {} | audio ports {} buffers {} pending {}",
        s.faults, s.supervisor_interventions, s.audio_ports, s.audio_buffers, s.audio_pending
    )
    .unwrap();
    let k = &s.kernel_resources;
    writeln!(out,"Kernel: direct {} / {} B | mappings {} | sema {} waiters {} | waits/signals/timeouts {}/{}/{}",k.direct_allocations,k.direct_bytes,k.mappings,k.semaphores,k.semaphore_waiters,k.waits,k.signals,k.timeouts).unwrap();
    for (name, h) in &s.subsystems {
        writeln!(
            out,
            "  {:18} {:11} {}",
            name,
            format!("{:?}", h.state),
            clean(&h.detail, 72)
        )
        .unwrap();
    }
    writeln!(out, "Latest event: {}", clean(&s.latest_event, 140)).unwrap();
    if let Some(stop) = &s.stop {
        writeln!(
            out,
            "STOP: {} | captured RIP {} [{}] | RSP {}",
            stop.reason, stop.boundary_rip, stop.boundary_domain, stop.rsp
        )
        .unwrap();
        writeln!(
            out,
            "Guest import continuation: {} | native thread {:?}",
            shown(&stop.guest_continuation),
            stop.thread
        )
        .unwrap();
        if let Some(p) = &stop.provider {
            writeln!(
                out,
                "Provider: {} {}/{}",
                p.nid,
                clean(&p.library, 128),
                clean(&p.module, 128)
            )
            .unwrap();
        }
        if stop.exception != 0 {
            writeln!(
                out,
                "Exception {:#x} address {} access-kind {:?}",
                stop.exception,
                shown(&stop.fault_address),
                stop.access_kind
            )
            .unwrap();
        }
        if let Some(o) = &stop.object {
            writeln!(out, "Object: {}", clean(o, 180)).unwrap();
        }
    }
    if let Some(t) = &s.teardown {
        writeln!(out,"TEARDOWN: joined={} workers={} FS={} GS={} reservations={} release-errors={:?} waiters={} sleep-tickets={}",t.thread_joined,t.workers_joined,t.host_fs_restored,t.host_gs_preserved,t.active_reservations,t.release_errors,t.remaining_waiters,t.remaining_sleep_tickets).unwrap();
    }
    if verbose {
        for p in s.providers.iter().take(12) {
            writeln!(
                out,
                "  {} {} calls={} returns={} refused={}",
                p.nid,
                clean(&p.library, 48),
                p.calls,
                p.returned,
                p.refused
            )
            .unwrap();
        }
    }
    out
}
fn compact(s: &Snapshot, verbose: bool) -> String {
    use std::fmt::Write;
    let mut out = format!(
        "ASTERO | {} | {:.3} ms\nArtifact: {}\nSHA {} | entry {} | bias {}\n",
        s.runtime_state,
        s.elapsed_us as f64 / 1000.0,
        clean(&s.artifact, 60),
        s.sha256
            .as_deref()
            .map_or("pending", |s| &s[..s.len().min(16)]),
        shown(&s.entry_rip),
        shown(&s.load_bias)
    );
    writeln!(
        out,
        "Threads live/wait/peak: {}/{}/{} | image {} B",
        s.live_threads, s.waiting_threads, s.threads_peak, s.mapped_image_bytes
    )
    .unwrap();
    writeln!(
        out,
        "HLE registered {} (+{} data) | calls {} used {} returned {}",
        s.registered_callable_identities,
        s.data_exports,
        s.hle_calls,
        s.unique_providers,
        s.returned_providers
    )
    .unwrap();
    writeln!(
        out,
        "Unknown {} refused {} | last NID {}",
        s.unknown_calls,
        s.refused_calls,
        s.last_provider.as_ref().map_or("none", |p| p.nid.as_str())
    )
    .unwrap();
    writeln!(
        out,
        "Heap live/peak {}/{} B | allocations {}/{} peak",
        s.heap_live, s.heap_peak, s.allocations_live, s.allocations_peak
    )
    .unwrap();
    writeln!(
        out,
        "Sync m/r/c {}/{}/{} | wait/wake/timeout {}/{}/{}",
        s.mutexes, s.rwlocks, s.condvars, s.waits, s.wakes, s.timeouts
    )
    .unwrap();
    writeln!(
        out,
        "PC samples {} guest {} | unique RIP/pages {}/{}",
        s.pc.total, s.pc.guest_samples, s.pc.unique_guest_rips, s.pc.unique_guest_pages
    )
    .unwrap();
    writeln!(
        out,
        "Guest PC {} | latest [{}] | faults {} supervisor {}",
        shown(&s.pc.last_guest_rip),
        s.pc.last_domain.as_deref().unwrap_or("off"),
        s.faults,
        s.supervisor_interventions
    )
    .unwrap();
    let states: Vec<_> = s
        .subsystems
        .iter()
        .map(|(n, h)| format!("{n}: {:?}", h.state))
        .collect();
    for pair in states.chunks(2) {
        writeln!(out, "{}", pair.join(" | ")).unwrap();
    }
    writeln!(out, "Latest: {}", clean(&s.latest_event, 70)).unwrap();
    if verbose {
        writeln!(
            out,
            "Returned {} distinct providers; full details: --trace/--log-file",
            s.returned_providers
        )
        .unwrap();
    }
    out
}
/// UI cadence only, never a guest timing service. Joined before final output or unwinding.
pub struct Live {
    stop: Arc<(Mutex<bool>, Condvar)>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Live {
    pub fn start(observer: Arc<Observer>, options: &Options) -> Self {
        let enabled = options.interactive(
            std::io::stdout().is_terminal(),
            std::env::var("TERM").ok().as_deref(),
        ) && crossterm::terminal::size().is_ok_and(|(w, h)| w >= 80 && h >= 20);
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let shared = stop.clone();
        let verbose = options.verbose;
        let thread = enabled.then(|| {
            std::thread::spawn(move || {
                struct Screen;
                impl Drop for Screen {
                    fn drop(&mut self) {
                        let mut o = std::io::stdout().lock();
                        let _ = crossterm::execute!(
                            o,
                            crossterm::cursor::Show,
                            crossterm::terminal::LeaveAlternateScreen
                        );
                        let _ = o.flush();
                    }
                }
                let _screen = Screen;
                {
                    let mut o = std::io::stdout().lock();
                    if crossterm::execute!(
                        o,
                        crossterm::terminal::EnterAlternateScreen,
                        crossterm::cursor::Hide
                    )
                    .is_err()
                    {
                        return;
                    }
                    let _ = o.flush();
                }
                let mut previous = String::new();
                loop {
                    let text = compact(&observer.snapshot(), verbose);
                    if text != previous {
                        let mut o = std::io::stdout().lock();
                        let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
                        if crossterm::queue!(
                            o,
                            crossterm::cursor::MoveTo(0, 0),
                            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
                        )
                        .is_err()
                        {
                            break;
                        }
                        for (row, line) in text
                            .lines()
                            .take(height.saturating_sub(1) as usize)
                            .enumerate()
                        {
                            if crossterm::queue!(o, crossterm::cursor::MoveTo(0, row as u16))
                                .is_err()
                            {
                                break;
                            }
                            let clipped: String = line
                                .chars()
                                .map(|c| if c.is_ascii() { c } else { '?' })
                                .take(width.saturating_sub(1) as usize)
                                .collect();
                            if write!(o, "{clipped}").is_err() {
                                break;
                            }
                        }
                        if o.flush().is_err() {
                            break;
                        }
                    }
                    previous = text;
                    let (lock, cv) = &*shared;
                    let guard = lock.lock().unwrap_or_else(|p| p.into_inner());
                    let (guard, _) = cv
                        .wait_timeout_while(guard, std::time::Duration::from_millis(200), |done| {
                            !*done
                        })
                        .unwrap_or_else(|p| p.into_inner());
                    if *guard {
                        break;
                    }
                }
            })
        });
        Self { stop, thread }
    }
}
impl Drop for Live {
    fn drop(&mut self) {
        let (lock, cv) = &*self.stop;
        *lock.lock().unwrap_or_else(|p| p.into_inner()) = true;
        cv.notify_all();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compact_fits_minimum_terminal_without_event_spam() {
        let s = Snapshot::preparing("x".into());
        let text = compact(&s, true);
        assert!(text.lines().count() <= 19);
        assert!(text.lines().all(|l| l.len() < 80));
    }
    #[test]
    fn output_files_never_overwrite_existing_evidence() {
        let path =
            std::env::temp_dir().join(format!("astero-m44-output-{}.txt", std::process::id()));
        std::fs::write(&path, b"existing").unwrap();
        let o = Options {
            report: Some(path.clone()),
            ..Default::default()
        };
        assert!(o.files().is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"existing");
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn tty_modes() {
        let mut o = Options::default();
        assert!(o.interactive(true, None));
        assert!(!o.interactive(false, None));
        assert!(!o.interactive(true, Some("dumb")));
        o.no_dashboard = true;
        assert!(!o.interactive(true, None));
        o.no_dashboard = false;
        o.trace = true;
        assert!(!o.interactive(true, None));
    }
    #[test]
    fn final_screen_keeps_addresses_distinct() {
        let mut s = Snapshot::preparing("artifact\x1b[1m".into());
        s.stop = Some(astero_core::input::entry::observability::Stop {
            reason: "UnresolvedFunction".into(),
            boundary_rip: "0xhost".into(),
            boundary_domain: "RuntimeBridge".into(),
            guest_continuation: Some("0xguest".into()),
            ..Default::default()
        });
        let text = render(&s, false);
        assert!(text.contains("captured RIP 0xhost [RuntimeBridge]"));
        assert!(text.contains("continuation: 0xguest"));
        assert!(!text.contains('\x1b'));
    }
    #[test]
    fn sampler_option_limits() {
        let mut o = Options::default();
        assert!(
            o.take(&"--pc-sample-ms".into(), &mut vec!["4".into()].into_iter())
                .is_err()
        );
        assert!(
            o.take(&"--pc-sample-ms".into(), &mut vec!["20".into()].into_iter())
                .unwrap()
        );
        assert_eq!(o.sample, Some(20));
    }
}
