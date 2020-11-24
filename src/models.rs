#[cfg(target_os = "macos")]
use mach::vm_types::natural_t;
use serde::Serialize;

/// Struct containing a disk' information.
#[derive(Debug, Serialize)]
pub struct Disks {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub avail_space: u64,
}

/// Struct containing a disk_io (bytes read/wrtn) information.
#[derive(Debug, Serialize)]
pub struct IoStats {
    pub device_name: String,
    pub bytes_read: i64,
    pub bytes_wrtn: i64,
}

/// Struct containing a cpu's loadavg information.
#[derive(Debug, Serialize)]
pub struct LoadAvg {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

/// Struct containing the memory (ram/swap) information.
#[derive(Debug, Serialize)]
pub struct Memory {
    pub total_virt: u64,
    pub avail_virt: u64,
    pub total_swap: u64,
    pub avail_swap: u64,
}

/// Struct containing the principal host's information.
#[derive(Debug, Serialize)]
pub struct HostInfo {
    pub loadavg: LoadAvg,
    pub memory: Memory,
    pub os_version: String,
    pub hostname: String,
    pub uptime: u64,
}

#[doc(hidden)]
#[cfg(target_os = "macos")]
#[repr(C)]
pub struct vm_statistics64 {
    pub free_count: natural_t,
    pub active_count: natural_t,
    pub inactive_count: natural_t,
    pub wire_count: natural_t,
    pub zero_fill_count: u64,
    pub reactivations: u64,
    pub pageins: u64,
    pub pageouts: u64,
    pub faults: u64,
    pub cow_faults: u64,
    pub lookups: u64,
    pub hits: u64,
    pub purges: u64,
    pub purgeable_count: natural_t,
    pub speculative_count: natural_t,
    pub decompressions: u64,
    pub compressions: u64,
    pub swapins: u64,
    pub swapouts: u64,
    pub compressor_page_count: natural_t,
    pub throttled_count: natural_t,
    pub external_page_count: natural_t,
    pub internal_page_count: natural_t,
    pub total_uncompressed_pages_in_compressor: u64,
}
