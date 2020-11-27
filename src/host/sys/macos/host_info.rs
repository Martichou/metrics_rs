#[cfg(target_os = "linux")]
use crate::read_and_trim;
#[cfg(target_os = "macos")]
use crate::to_str;

use crate::cpu;
use crate::memory;
use crate::models;
use crate::sys;

#[cfg(target_os = "macos")]
use core_foundation_sys::{
    base::{kCFAllocatorDefault, CFRelease, CFTypeRef},
    string::{CFStringGetCString, CFStringRef},
};
#[cfg(target_os = "macos")]
use io_kit_sys::*;
#[cfg(target_os = "macos")]
use io_kit_sys::{kIOMasterPortDefault, keys::kIOPlatformUUIDKey, IOServiceMatching};
#[cfg(target_os = "macos")]
use libc::c_char;
#[cfg(target_os = "macos")]
use libc::{c_void, sysctl, timeval};
use models::HostInfo;
use std::io::{Error, ErrorKind};
use std::time::Duration;

/// Get some basic [HostInfo] of the host.
///
/// On linux and macOS it will get the `os_version` and `hostname` from uname.
///
/// For the `uptime`/`loadavg`/`memory` on linux it will get them from sysinfo.
/// But on macOS it will use the crate [get_loadavg] and [get_memory] and a special get_uptime function using an unsafe syscall.
///
/// [get_loadavg]: ../cpu/fn.get_loadavg.html
/// [get_memory]: ../memory/fn.get_memory.html
/// [HostInfo]: ../struct.HostInfo.html
pub fn get_host_info() -> Result<HostInfo, Error> {
    let x = sys::get_uname()?;

    Ok(HostInfo {
        loadavg: cpu::get_loadavg().unwrap(),
        memory: memory::get_memory()?,
        uptime: get_uptime().unwrap().as_secs(),
        os_version: sys::get_os_version_from_uname(&x),
        hostname: sys::get_hostname_from_uname(&x),
    })
}

fn get_uptime() -> Result<Duration, Error> {
    let mut data = std::mem::MaybeUninit::<timeval>::uninit();
    let mib = [1, 21];

    if unsafe {
        sysctl(
            &mib[0] as *const _ as *mut _,
            mib.len() as u32,
            &mut data as *mut _ as *mut c_void,
            &mut std::mem::size_of::<timeval>(),
            std::ptr::null_mut(),
            0,
        )
    } < 0
    {
        return Err(Error::last_os_error());
    }

    let data = unsafe { data.assume_init() };
    Ok(Duration::from_secs(data.tv_sec as u64))
}
