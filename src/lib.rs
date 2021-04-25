//! `sys_metrics` is a crate used to get a system's information.
//!
//! It attempt to provide information about:
//!
//!  * CPU
//!  * Disks
//!  * Host
//!  * Memory
//!
//! ## Quick start
//! ```
//! use sys_metrics::{cpu::*, disks::*, host::*, memory::*};
//!
//! dbg!(get_host_info());
//! dbg!(get_uuid());
//! dbg!(get_cpufreq());
//! dbg!(get_cpustats());
//! dbg!(get_partitions_physical());
//! dbg!(get_iostats());
//! dbg!(get_iostats_physical());
//! dbg!(get_users());
//! dbg!(get_cpu_logical_count());
//! dbg!(get_memory());
//! dbg!(get_swap());
//! ```

/// CPU information
pub mod cpu;
/// Disks information
pub mod disks;
/// Host system information
pub mod host;
/// Memory information
pub mod memory;

use libc::c_char;
use std::ffi::CStr;
#[cfg(target_os = "linux")]
use std::fs;
use std::io::Error;

#[cfg(target_os = "macos")]
lazy_static::lazy_static! {
    static ref PAGE_SIZE: u64 = {
        unsafe {
            libc::sysconf(libc::_SC_PAGESIZE) as u64
        }
    };
}

/// Function used if you want to divide ticked value by the host's JIFFIE (USER_HZ) value. (like CpuStats or IoStats)
pub fn clock_ticks() -> Result<u64, Error> {
    let result = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };

    if result > 0 {
        Ok(result as u64)
    } else {
        Err(Error::last_os_error())
    }
}

lazy_static::lazy_static! {
    /// Time units in JIFFIE (USER_HZ)
    pub static ref CLOCK_TICKS: u64 = clock_ticks().expect("Unable to determine CPU number of ticks per second");
}

/// Read from path to content, trim it and return the String
#[cfg(target_os = "linux")]
pub(crate) fn read_and_trim(path: &str) -> Result<String, Error> {
    let content = fs::read_to_string(path)?;
    Ok(content.trim().to_owned())
}

/// Convert c_char (string in C) to a str in Rust
#[inline]
pub(crate) fn to_str<'a>(s: *const c_char) -> &'a str {
    unsafe {
        let res = CStr::from_ptr(s).to_bytes();
        std::str::from_utf8_unchecked(res)
    }
}
