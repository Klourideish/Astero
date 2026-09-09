//! Shared synthetic composition adapter; no byte parsing or candidate semantics in CLI.
pub fn session(max_symbols: u64) -> Result<astero_core::session::Session, String> {
    astero_core::session::inputs::synthetic::session(max_symbols)
        .map_err(|e| format!("Synthetic composition: {e:?}"))
}
