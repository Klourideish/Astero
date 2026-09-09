use super::{Selection, State};
use std::fmt::Write;

pub fn render(selection: &Selection) -> String {
    let request = selection.request();
    let mut text = format!(
        "Astero acquisition only\nSelected path: {:?}\nLimits: max-bytes={} max-read-calls={}\n",
        request.path, request.limits.max_bytes, request.limits.max_read_calls
    );
    match selection.state() {
        State::Ready => text.push_str("Status: Ready\n"),
        State::Acquired(source) => {
            writeln!(
                text,
                "Status: Acquired\nSource identity: {:?}\nObserved bytes: {}",
                source.identity(),
                source.len()
            )
            .expect("String write");
        }
        State::Failed(error) => {
            writeln!(text, "Status: Failed\n{error}").expect("String write");
        }
    }
    text.push_str("No guest loaded. No parsing or linkage performed.\n");
    text
}
