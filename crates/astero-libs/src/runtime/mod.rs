//! Guest language-runtime export contracts: initialization/finalization and unwind/exception ABI.
//! Future children follow those contract families, not unrelated platform services.
//! Libc scalar/string contracts belong in libc; TLS mechanisms in kernel::threading::tls;
//! native faults in kernel::execution and guest signal delivery in kernel::signals.
//! Planned home only; no runtime functionality is implemented.
