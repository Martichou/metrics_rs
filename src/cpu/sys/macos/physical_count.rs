use libc::{c_uint, c_void, sysctl};
use std::io::Error;

/// Return the number of physcial core the system has.
pub fn get_physical_count() -> Result<u32, Error> {
    let mut data: c_uint = 0;
    let mut mib: [i32; 2] = [libc::CTL_HW, libc::HW_NCPU];

    if unsafe {
        sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            &mut data as *mut _ as *mut c_void,
            &mut std::mem::size_of::<c_uint>(),
            std::ptr::null_mut(),
            0,
        )
    } < 0
    {
        return Err(Error::last_os_error());
    }

    Ok(data)
}
