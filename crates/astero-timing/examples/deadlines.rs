use astero_timing::{
    scheduler::{Completion, Config, Label, TimingEngine},
    time::Span,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (engine, clock) = TimingEngine::manual(Config {
        max_pending: 3,
        max_snapshot_entries: 3,
    })?;
    let s = engine.scheduler();
    let delay = Span::from_millis(5)?;
    let a = s.after(delay, Label::new("first")?)?;
    let b = s.after(delay, Label::new("second")?)?;
    let cancelled = s.after(delay, Label::new("cancelled")?)?;
    println!("Cancellation: {:?}", s.cancel(&cancelled)?);
    clock.advance(delay)?;
    for t in [a, b] {
        match t.wait_timeout(std::time::Duration::from_secs(5)) {
            Some(Completion::Fired(e)) => println!(
                "event={} dispatch={} at={}ns lateness={}ns",
                e.sequence,
                e.dispatch_order,
                e.fired_at.as_nanos(),
                e.lateness.as_nanos()
            ),
            other => return Err(format!("Unexpected completion: {other:?}").into()),
        }
    }
    let v = s.snapshot(3)?;
    println!(
        "pending={} fired={} cancelled={}",
        v.pending_total, v.counters.fired, v.counters.cancelled
    );
    engine.shutdown()?;
    println!("Worker joined. Manual host-domain demonstration; no guest execution.");
    Ok(())
}
