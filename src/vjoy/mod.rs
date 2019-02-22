use libc::{c_int, c_uint, c_uchar, c_long};

// Is this right?
type WORD = usize;

#[repr(C)]
enum VjdStat  /* Declares an enumeration data type */
{
	VJD_STAT_OWN,	// The  vJoy Device is owned by this application.
	VJD_STAT_FREE,	// The  vJoy Device is NOT owned by any application (including this one).
	VJD_STAT_BUSY,	// The  vJoy Device is owned by another application. It cannot be acquired by this application.
	VJD_STAT_MISS,	// The  vJoy Device is missing. It either does not exist or the driver is down.
	VJD_STAT_UNKN	// Unknown
}

#[derive(Debug, Copy, Clone)]
pub enum Stat {
    Own,
    Free,
    Busy,
    Missing,
    Unknown
}

#[link(name = "vJoyInterface")]
extern {
    fn vJoyEnabled() -> bool;
    fn GetvJoyMaxDevices(n: *mut c_int) -> bool;
    fn DriverMatch(dll_ver: *mut WORD, drv_ver: *mut WORD) -> bool;
    fn GetNumberExistingVJD(n: *mut c_int) -> bool;
    fn GetVJDStatus(n: c_uint) -> VjdStat;
    fn AcquireVJD(rID: c_uint) -> bool;
    fn ResetVJD(rID: c_uint) -> bool;
    fn RelinquishVJD(rID: c_uint);
    fn SetBtn(value: bool, rID: c_uint, nBtn: c_uchar) -> bool;
    fn SetAxis(value: c_long, rID: c_uint, axis: c_uint) -> bool;
}

pub fn enabled() -> bool {
    unsafe { vJoyEnabled() }
}

pub fn max_devices() -> Option<i32> {
    let mut n = 0;
    if unsafe { GetvJoyMaxDevices(&mut n) } {
        Some(n)
    } else {
        None
    }
}

pub fn driver_match() -> bool {
    let mut dll_ver = WORD::default();
    let mut drv_ver = WORD::default();

    unsafe {
        DriverMatch(&mut dll_ver, &mut drv_ver)
    }
}

pub fn get_number_existing_devices() -> Option<i32> {
    let mut n = 0;
    
    if unsafe { GetNumberExistingVJD(&mut n) } {
        Some(n)
    } else {
        None
    }
}

pub fn get_vjd_status(rID: u32) -> Stat {
    use VjdStat::*;

    let c_stat = unsafe {
        GetVJDStatus(rID)
    };

    match c_stat {
        VJD_STAT_OWN => Stat::Own,
        VJD_STAT_FREE => Stat::Free,
        VJD_STAT_BUSY => Stat::Busy,
        VJD_STAT_MISS => Stat::Missing,
        VJD_STAT_UNKN => Stat::Unknown
    }
}

pub fn acquire_vjd(rID: u32) -> bool {
    unsafe {
        AcquireVJD(rID)
    }
}

pub fn reset_vjd(rID: u32) -> bool {
    unsafe {
        ResetVJD(rID)
    }
}

pub fn relinquish_vjd(rID: u32) {
    unsafe {
        RelinquishVJD(rID)
    }
}

pub fn set_axis(value: i32, rID: u32, axis: u32) -> bool {
    unsafe {
        SetAxis(value, rID, axis)
    }
}

pub fn set_btn(value: bool, rID: u32, nBtn: u8) -> bool {
    unsafe {
        SetBtn(value, rID, nBtn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enabled() {
        assert!(enabled());
    }

    #[test]
    fn test_driver_match() {
        assert!(driver_match());
    }
}
