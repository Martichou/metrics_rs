use crate::models;

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
use libc::{c_char, c_void};
use models::IoStats;
use std::ffi::CStr;
use std::io::Error;
use std::io::ErrorKind;

/// Get basic [IoStats] info for each disks/partitions.
///
/// It only contains the `device_name` and the number of bytes `read`/`wrtn`.
///
/// On linux it will get them from `/proc/diskstats`.
///
/// On macOS it will use unsafes call to multiple OSX specific functions.
///
/// [IoStats]: ../struct.IoStats.html
pub fn get_iostats() -> Result<Vec<IoStats>, Error> {
    let mut viostats: Vec<IoStats> = Vec::new();

    unsafe {
        let mut disk_list = std::mem::MaybeUninit::<io_iterator_t>::uninit();
        if IOServiceGetMatchingServices(
            kIOMasterPortDefault,
            IOServiceMatching(b"IOMedia\0".as_ptr() as *const c_char),
            disk_list.as_mut_ptr(),
        ) != kIOReturnSuccess
        {
            return Err(Error::last_os_error());
        }
        let disk_list = disk_list.assume_init();

        #[allow(unused_assignments)]
        let mut disk: io_registry_entry_t = 0;
        let mut parent: io_registry_entry_t = 0;
        let mut parent_dict: CFDictionaryRef;
        let mut props_dict: CFDictionaryRef;
        let mut stats_dict: CFDictionaryRef;

        disk = IOIteratorNext(disk_list);
        while disk != 0 {
            parent_dict = std::ptr::null();
            props_dict = std::ptr::null();
            stats_dict = std::ptr::null();

            // Maybe pass the plane as a mut_ptr
            if IORegistryEntryGetParentEntry(disk, b"IOService\0".as_ptr() as *mut i8, &mut parent)
                != kIOReturnSuccess
            {
                IOObjectRelease(disk);
                return Err(Error::last_os_error());
            }

            // Maybe pass the className as a mut_ptr
            if IOObjectConformsTo(parent, b"IOBlockStorageDriver\0".as_ptr() as *mut i8) != 0 {
                // The parent_dict convertion was a try error, might fail
                if IORegistryEntryCreateCFProperties(
                    disk,
                    &mut parent_dict as *const _ as *mut _,
                    kCFAllocatorDefault,
                    0,
                ) != kIOReturnSuccess
                {
                    IOObjectRelease(disk);
                    IOObjectRelease(parent);
                    return Err(Error::last_os_error());
                }

                if IORegistryEntryCreateCFProperties(
                    parent,
                    &mut props_dict as *const _ as *mut _,
                    kCFAllocatorDefault,
                    0,
                ) != kIOReturnSuccess
                {
                    CFRelease(parent_dict as *mut c_void);
                    CFRelease(props_dict as *mut c_void);
                    IOObjectRelease(disk);
                    IOObjectRelease(parent);
                    return Err(Error::last_os_error());
                }

                let mut disk_name_ref = std::mem::MaybeUninit::<CFStringRef>::uninit();
                if CFDictionaryGetValueIfPresent(
                    parent_dict,
                    CFSTR(b"BSD Name\0".as_ptr() as *mut i8) as *mut c_void,
                    disk_name_ref.as_mut_ptr() as *mut *const c_void,
                ) == 0
                {
                    CFRelease(parent_dict as *mut c_void);
                    CFRelease(props_dict as *mut c_void);
                    IOObjectRelease(disk);
                    IOObjectRelease(parent);
                    return Err(Error::new(
                        ErrorKind::Other,
                        "CFDictionaryGetValueIfPresent: BSD Name not found in the parent_dict",
                    ));
                }
                let disk_name_ref = disk_name_ref.assume_init();
                let mut name = [0i8; 64];
                if CFStringGetCString(disk_name_ref, name.as_mut_ptr(), 64, 134217984) == 0 {
                    CFRelease(parent_dict as *mut c_void);
                    CFRelease(props_dict as *mut c_void);
                    IOObjectRelease(disk);
                    IOObjectRelease(parent);
                    return Err(Error::new(ErrorKind::Other, "Cannot get the buffer filled"));
                }

                if CFDictionaryGetValueIfPresent(
                    props_dict,
                    CFSTR(b"Statistics\0".as_ptr() as *mut i8) as *mut c_void,
                    &mut stats_dict as *mut _ as *mut *const c_void,
                ) == 0
                {
                    CFRelease(parent_dict as *mut c_void);
                    CFRelease(props_dict as *mut c_void);
                    IOObjectRelease(disk);
                    IOObjectRelease(parent);
                    return Err(Error::new(
                        ErrorKind::Other,
                        "CFDictionaryGetValueIfPresent: Statistics not found in the props_dict",
                    ));
                }

                let mut write_bytes_nbr = std::mem::MaybeUninit::<CFNumberRef>::uninit();
                let mut read_bytes_nbr = std::mem::MaybeUninit::<CFNumberRef>::uninit();
                let mut read_bytes = 0i64;
                let mut write_bytes = 0i64;

                if CFDictionaryGetValueIfPresent(
                    stats_dict,
                    CFSTR(b"Bytes (Read)\0".as_ptr() as *mut i8) as *mut c_void,
                    write_bytes_nbr.as_mut_ptr() as *mut *const c_void,
                ) == 0
                {
                    CFRelease(parent_dict as *mut c_void);
                    CFRelease(props_dict as *mut c_void);
                    IOObjectRelease(disk);
                    IOObjectRelease(parent);
                    return Err(Error::new(
                        ErrorKind::Other,
                        "CFDictionaryGetValueIfPresent: Bytes Read not found in the stats_dict",
                    ));
                }
                let number = write_bytes_nbr.assume_init();
                CFNumberGetValue(number, 4, &mut read_bytes as *mut _ as *mut c_void);

                if CFDictionaryGetValueIfPresent(
                    stats_dict,
                    CFSTR(b"Bytes (Write)\0".as_ptr() as *mut i8) as *mut c_void,
                    read_bytes_nbr.as_mut_ptr() as *mut *const c_void,
                ) == 0
                {
                    CFRelease(parent_dict as *mut c_void);
                    CFRelease(props_dict as *mut c_void);
                    IOObjectRelease(disk);
                    IOObjectRelease(parent);
                    return Err(Error::new(
                        ErrorKind::Other,
                        "CFDictionaryGetValueIfPresent: Bytes Write not found in the stats_dict",
                    ));
                }
                let number = read_bytes_nbr.assume_init();
                CFNumberGetValue(number, 4, &mut write_bytes as *mut _ as *mut c_void);

                let name = match CStr::from_ptr(name.as_mut_ptr()).to_str() {
                    Ok(val) => val.to_owned(),
                    Err(_) => String::from("?"),
                };

                viostats.push(IoStats {
                    device_name: name,
                    bytes_read: read_bytes,
                    bytes_wrtn: write_bytes,
                });

                CFRelease(parent_dict as *mut c_void);
                CFRelease(props_dict as *mut c_void);
                IOObjectRelease(disk);
                IOObjectRelease(parent);
            }
            disk = IOIteratorNext(disk_list);
        }

        IOObjectRelease(disk_list);
    }

    Ok(viostats)
}
