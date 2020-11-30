use crate::PAGE_SIZE;

use crate::models;

use mach::{
    kern_return::kern_return_t,
    mach_types::{host_name_port_t, host_t},
    message::mach_msg_type_number_t,
    vm_types::integer_t,
};
use models::vm_statistics64;
use models::Memory;
use std::io::Error;

extern "C" {
    fn mach_host_self() -> host_name_port_t;

    fn host_statistics64(
        host_priv: host_t,
        flavor: integer_t,
        host_info_out: *mut integer_t,
        host_info_outCnt: *const mach_msg_type_number_t,
    ) -> kern_return_t;
}

/// Return the [Memory] struct.
///
/// Only contains the virtual/swap memory total/available.
///
/// On linux it will get them from the sysinfo.
///
/// On macOS it will use unsafe syscall due to specific OSX implementation.
///
/// [Memory]: ../struct.Memory.html
pub fn get_memory() -> Result<Memory, Error> {
    let count = 38;
    // ALLOCATE A PORT
    let port = unsafe { mach_host_self() };
    let mut vm_stats = std::mem::MaybeUninit::<vm_statistics64>::uninit();
    // GET HOST INFO ABOUT MEMORY & CHECK THE RETURN VALUE OF host_statistics64
    if unsafe { host_statistics64(port, 4, vm_stats.as_mut_ptr() as *mut integer_t, &count) } != 0 {
        return Err(Error::last_os_error());
    }
    // Is the port_deallocate really usefull ?
    // If I try to stay like vm_stat (https://opensource.apple.com/source/system_cmds/system_cmds-498.2/vm_stat.tproj/vm_stat.c)
    // there is not a single deallocation for the port whatsoever.
    //
    // if mach_port_deallocate(mach_task_self(), port) != 0 {
    //     return Err(Error::last_os_error());
    // }

    // ASSUME VM_STATS IS INIT
    let vm_stats = unsafe { vm_stats.assume_init() };
    // AVAILABLE VIRT MEMORY
    let virt_avail = (vm_stats.active_count + vm_stats.free_count) as u64 * (*PAGE_SIZE);

    // TOTAL VIRTUAL MEMORY
    let mut name: [i32; 2] = [6, 24];
    let mut virt_total = 0u64;
    let mut length = std::mem::size_of::<u64>();
    if unsafe {
        libc::sysctl(
            name.as_mut_ptr(),
            2,
            &mut virt_total as *mut u64 as *mut libc::c_void,
            &mut length,
            std::ptr::null_mut(),
            0,
        )
    } != 0
    {
        return Err(Error::last_os_error());
    }

    // SWAP MEMORY
    let mut name: [i32; 2] = [2, 5];
    let mut swap_info = std::mem::MaybeUninit::<libc::xsw_usage>::uninit();
    let mut length = std::mem::size_of::<libc::xsw_usage>();
    if unsafe {
        libc::sysctl(
            name.as_mut_ptr(),
            2,
            swap_info.as_mut_ptr() as *mut libc::c_void,
            &mut length,
            std::ptr::null_mut(),
            0,
        )
    } != 0
    {
        return Err(Error::last_os_error());
    }
    let swap_info = unsafe { swap_info.assume_init() };

    Ok(Memory {
        total_virt: virt_total,
        total_swap: swap_info.xsu_total,
        avail_virt: virt_avail,
        avail_swap: swap_info.xsu_avail,
    })
}
