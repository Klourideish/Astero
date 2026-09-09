use astero_core::session::inputs::{CandidateDetail, NameState};
/// Byte-faithful display. Canonical report data is never normalized or mutated.
pub fn display_name(detail: &CandidateDetail) -> String {
    match detail.name_state {
        NameState::Absent => "<absent name>".into(),
        NameState::Empty => "<empty name>".into(),
        NameState::Utf8 => match detail
            .name
            .as_deref()
            .and_then(|b| std::str::from_utf8(b).ok())
        {
            Some(text) => format!("UTF-8: \"{}\"", text.escape_default()),
            None => "<inconsistent UTF-8 name observation>".into(),
        },
        NameState::Bytes => format!(
            "<non-UTF8: {}>",
            detail
                .name
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect::<Vec<_>>()
                .join(" ")
        ),
    }
}
