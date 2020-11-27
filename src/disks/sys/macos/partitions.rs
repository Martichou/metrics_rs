use crate::disk_usage;
use crate::is_physical_filesys;
use crate::models;
use crate::to_str;

use core_foundation_sys::{
    base::{kCFAllocatorDefault, CFRelease},
    dictionary::{CFDictionaryGetValueIfPresent, CFDictionaryRef},
    number::{CFNumberGetValue, CFNumberRef},
    string::{CFStringGetCString, CFStringRef},
};
use io_kit_sys::{
    kIOMasterPortDefault,
    ret::kIOReturnSuccess,
    types::{io_iterator_t, io_registry_entry_t},
    IOServiceMatching, *,
};
use libc::{c_char, statfs};
use models::{Disks, IoStats};
use std::ffi::CStr;
use std::io::Error;
use std::io::ErrorKind;
use std::path::PathBuf;

extern "C" {
    fn getfsstat64(buf: *mut statfs, bufsize: libc::c_int, flags: libc::c_int) -> libc::c_int;
}

/// Return a Vec of [Disks] with their minimal informations.
///
/// Contains `name`, `mount_point` and `total`/`free` space.
///
/// On linux it will get them from `/proc/mounts`.
///
/// On macOS it will use an unsafe call to `getfsstat64`.
///
/// [Disks]: ../struct.Disks.html
pub fn get_partitions_physical() -> Result<Vec<Disks>, Error> {
    let expected_len = unsafe { getfsstat64(std::ptr::null_mut(), 0, 2) };
    let mut mounts: Vec<statfs> = Vec::with_capacity(expected_len as usize);

    let result = unsafe {
        getfsstat64(
            mounts.as_mut_ptr(),
            std::mem::size_of::<statfs>() as libc::c_int * expected_len,
            2,
        )
    };

    if result < 0 {
        return Err(Error::last_os_error());
    }

    unsafe {
        mounts.set_len(result as usize);
    }

    let mut vdisks: Vec<Disks> = Vec::with_capacity(expected_len as usize);
    for stat in mounts {
        if !is_physical_filesys(to_str(stat.f_fstypename.as_ptr())) {
            continue;
        }
        let m_p = PathBuf::from(to_str(stat.f_mntonname.as_ptr()).to_owned());
        let usage: (u64, u64) = match disk_usage(&m_p) {
            Ok(val) => val,
            Err(x) => return Err(x),
        };
        vdisks.push(Disks {
            name: to_str(stat.f_mntfromname.as_ptr()).to_owned(),
            mount_point: m_p.into_os_string().into_string().unwrap(),
            total_space: usage.0 / 100000,
            avail_space: usage.1 / 100000,
        });
    }

    Ok(vdisks)
}
