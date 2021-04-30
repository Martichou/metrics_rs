use crate::binding::{host_statistics, mach_host_self, vmmeter};
use crate::cpu::CpuStats;

use mach::mach_port::mach_port_deallocate;
use mach::traps::mach_task_self;
use mach::vm_types::integer_t;
use std::io::Error;

/// Get basic [CpuStats] info the host.
///
/// It only contains row information, to get the delta we need
/// to get the diff between N and N-1.
///
/// On linux it will get them from `/proc/stat`.
///
/// [CpuStats]: ../cpu/struct.CpuStats.html
pub fn get_cpustats() -> Result<CpuStats, Error> {
    let count = 35u32;
    // ALLOCATE A PORT
    let port = unsafe { mach_host_self() };
    let mut stats = std::mem::MaybeUninit::<vmmeter>::uninit();

    // GET CPU STATS INFO & SAVE THE RETURN VALUE OF host_statistics
    let result = unsafe { host_statistics(port, 2, stats.as_mut_ptr() as *mut integer_t, &count) };

    // Everybody seems to deallocate the port when it's for the cpu related stats
    // so let's be dumb and do the same without searching...
    if unsafe { mach_port_deallocate(mach_task_self(), port) } != 0 || result != 0 {
        return Err(Error::last_os_error());
    }

    // ASSUME VM_STATS IS INIT
    let stats = unsafe { stats.assume_init() };
    Ok(stats.into())
}
