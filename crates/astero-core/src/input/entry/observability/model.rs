//! Schema version 1. Addresses are hexadecimal strings; absent evidence is null, never zero.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Health {
    NotReached,
    Present,
    Initialized,
    Active,
    Partial,
    Blocked,
    Failed,
    Unavailable,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subsystem {
    pub state: Health,
    pub detail: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Modules {
    pub loaded: usize,
    pub hle_backed: usize,
    pub artifact_backed: usize,
    pub providers: usize,
    pub load_requests: u64,
    pub failed_loads: u64,
    pub unloads: u64,
    pub last_id: Option<u16>,
    pub loaded_ids: Vec<u16>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Provider {
    pub nid: String,
    pub library: String,
    pub library_bytes_hex: String,
    pub module: String,
    pub module_bytes_hex: String,
    pub calls: u64,
    pub returned: u64,
    pub unknown: u64,
    pub refused: u64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProviderIdentity {
    pub nid: String,
    pub library: String,
    pub library_bytes_hex: String,
    pub module: String,
    pub module_bytes_hex: String,
}
impl From<&Provider> for ProviderIdentity {
    fn from(p: &Provider) -> Self {
        Self {
            nid: p.nid.clone(),
            library: p.library.clone(),
            library_bytes_hex: p.library_bytes_hex.clone(),
            module: p.module.clone(),
            module_bytes_hex: p.module_bytes_hex.clone(),
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Pc {
    pub enabled: bool,
    pub interval_ms: Option<u64>,
    pub total: u64,
    pub guest_samples: u64,
    pub unique_guest_rips: usize,
    pub unique_guest_pages: usize,
    pub dropped: u64,
    pub suspends: u64,
    pub resumes: u64,
    pub last_rip: Option<String>,
    pub last_guest_rip: Option<String>,
    pub last_domain: Option<String>,
    pub guest_rips: Vec<String>,
    pub guest_page_addresses: Vec<String>,
    pub thread_samples: BTreeMap<u32, u64>,
    pub distribution: BTreeMap<String, u64>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Stop {
    pub reason: String,
    pub boundary_rip: String,
    pub boundary_domain: String,
    pub rsp: String,
    pub guest_continuation: Option<String>,
    pub provider: Option<ProviderIdentity>,
    pub object: Option<String>,
    pub exception: u32,
    pub fault_address: Option<String>,
    pub access_kind: Option<u64>,
    pub thread: Option<u32>,
    pub registers: Option<Vec<String>>,
    pub supervisor_intervened: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Teardown {
    pub thread_joined: bool,
    pub workers_joined: usize,
    pub host_fs_restored: bool,
    pub host_gs_preserved: bool,
    pub active_reservations: u64,
    pub release_errors: Vec<u32>,
    pub remaining_waiters: usize,
    pub remaining_sleep_tickets: usize,
    pub audio_stopped: bool,
    pub ajm_stopped: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Snapshot {
    #[serde(default)]
    pub host_presentation: HostPresentation,
    #[serde(default)]
    pub filesystem: Filesystem,

    #[serde(default)]
    pub modules: Modules,
    #[serde(default)]
    pub kernel_resources: KernelResources,
    pub schema_version: u32,
    pub artifact: String,
    pub sha256: Option<String>,
    pub source_bytes: Option<u64>,
    pub entry_rip: Option<String>,
    pub initial_rsp: Option<String>,
    pub load_bias: Option<String>,
    pub runtime_state: String,
    pub elapsed_us: u64,
    pub registered_callable_identities: usize,
    pub data_exports: usize,
    pub threads_created: usize,
    pub threads_peak: usize,
    pub live_threads: usize,
    pub waiting_threads: usize,
    pub running_or_host_threads: usize,
    pub hle_calls: u64,
    pub unique_providers: usize,
    pub returned_providers: usize,
    pub unknown_calls: u64,
    pub refused_calls: u64,
    pub last_provider: Option<ProviderIdentity>,
    pub providers: Vec<Provider>,
    pub mapped_runtime_bytes_excluding_landings: u64,
    pub source_sha256_unchanged: Option<bool>,
    pub native_interval_us: Option<u64>,
    pub mapped_image_bytes: u64,
    pub heap_live: u64,
    pub heap_peak: u64,
    pub allocations_live: usize,
    pub allocations_peak: usize,
    pub stack_tls_mappings: usize,
    pub guest_modules_observed: usize,
    pub declared_dependencies: usize,
    pub mutexes: usize,
    pub rwlocks: usize,
    pub condvars: usize,
    pub waiters: usize,
    pub waits: u64,
    pub wakes: u64,
    pub timeouts: u64,
    pub shutdown_interruptions: u64,
    pub audio_ports: usize,
    pub audio_buffers: u64,
    pub audio_bytes: u64,
    pub audio_pending: usize,
    pub ajm_contexts: usize,
    pub faults: u64,
    pub supervisor_interventions: u64,
    pub pc: Pc,
    pub subsystems: BTreeMap<String, Subsystem>,
    pub latest_event: String,
    pub stop: Option<Stop>,
    pub teardown: Option<Teardown>,
    pub limitations: Vec<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KernelResources {
    pub direct_allocations: usize,
    pub direct_bytes: u64,
    pub mappings: usize,
    pub allocations_total: u64,
    pub mappings_total: u64,
    #[serde(default)]
    pub unmaps_total: u64,
    #[serde(default)]
    pub mapped_bytes: u64,
    pub semaphores: usize,
    pub semaphore_waiters: usize,
    pub waits: u64,
    pub signals: u64,
    pub timeouts: u64,
    pub cancellations: u64,
}
impl Snapshot {
    pub fn preparing(artifact: String) -> Self {
        Self {
            schema_version:1, artifact, runtime_state:"Preparing".into(),
            subsystems:["Loader","Kernel","libc","pthread","C11 sync","AJM","Filesystem","Modules/sysmodules","VideoOut","GPU/AGC","Shader","Audio","UserService","Unclassified"].into_iter().map(|s|(s.into(),Subsystem{state:Health::NotReached,detail:"No runtime activity observed".into()})).collect(),
            latest_event:"Preparing entry authority".into(),
            limitations:vec!["Live owners are sampled independently; not an atomic process checkpoint".into(),"Returned provider means observed return, not semantic or boot completeness".into(),"PC samples are neither instruction nor cycle counts; host/HLE PCs are grouped".into(),"Waiting threads cover synchronization and sleep tickets; running includes host transitions".into(),"Lifecycle and object fields freeze before teardown; teardown fields describe post-join state".into()],
            ..Default::default()
        }
    }
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
pub fn address(n: u64) -> String {
    format!("0x{n:x}")
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Filesystem {
    pub open: usize,
    pub streams: usize,
    pub peak: usize,
    pub opens: u64,
    pub closes: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub seeks: u64,
    pub stats: u64,
    pub failures: u64,
    pub last_guest_path: String,
}

/// Host readiness only; never changes guest VideoOut/GPU health.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostPresentation {
    pub sink: String,
    pub state: String,
    pub received: u64,
    pub presented: u64,
    pub dropped: u64,
    pub refused: u64,
    pub last_id: Option<u64>,
    pub last_presented_id: Option<u64>,
    pub bytes: u64,
    pub width: u32,
    pub height: u32,
    pub format: Option<String>,
}
impl Default for HostPresentation {
    fn default() -> Self {
        Self {
            sink: "none".into(),
            state: "Unavailable".into(),
            received: 0,
            presented: 0,
            dropped: 0,
            refused: 0,
            last_id: None,
            last_presented_id: None,
            bytes: 0,
            width: 0,
            height: 0,
            format: None,
        }
    }
}
impl From<astero_video::presentation::Snapshot> for HostPresentation {
    fn from(s: astero_video::presentation::Snapshot) -> Self {
        Self {
            sink: s.sink.into(),
            state: format!("{:?}", s.state),
            received: s.received,
            presented: s.presented,
            dropped: s.dropped,
            refused: s.refused,
            last_id: s.last_id,
            last_presented_id: s.last_presented_id,
            bytes: s.bytes,
            width: s.width,
            height: s.height,
            format: s.format.map(|f| format!("{f:?}")),
        }
    }
}
