use alloc::{format, string::String, sync::Arc, vec::Vec};

use crate::{
    mm::{address::StepByOne, VirtAddr},
    task::Task,
};

// pub fn from_user_ptr_to_str(task: &Arc<Task>, buf: usize, len: usize) -> &'static str {
//     let buf = task
//         .inner_exclusive_access()
//         .translate(buf)
//         .expect(&format!(
//             "task_{}, task_name={}, receive bad user buffer addr? user_buf_addr=0x{:x}",
//             task.pid, task.name, buf
//         ));

//     let slice = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };
//     core::str::from_utf8(slice).unwrap()
// }

// pub fn from_user_ptr_to_slice<T>(task: &Arc<Task>, buf: usize, len: usize) -> &'static mut [T]
// where
//     T: Sized,
// {
//     let buf = task
//         .inner_exclusive_access()
//         .translate(buf)
//         .expect(&format!(
//             "task_{}, task_name={}, receive bad user buffer addr? user_buf_addr=0x{:x}",
//             task.pid, task.name, buf
//         ));

//     unsafe { core::slice::from_raw_parts_mut(buf as *mut T, len) }
// }

pub fn from_user_ptr<T>(task: &Arc<Task>, user_addr: usize) -> &'static mut T {
    let phy_addr = task
        .inner_exclusive_access()
        .translate(user_addr)
        .expect(&format!(
            "task_{}, task_name={}, receive bad user addr? user_addr=0x{:x}",
            task.pid, task.name, user_addr
        ));

    unsafe { &mut *(phy_addr as *mut T) }
}

pub fn from_user_cstring(task: &Arc<Task>, user_addr: usize) -> String {
    // let mut user_addr = user_addr;
    let mut phy_addr = task
        .inner_exclusive_access()
        .translate(user_addr)
        .expect(&format!(
            "task_{}, task_name={}, receive bad user addr? user_addr=0x{:x}",
            task.pid, task.name, user_addr
        ));

    let mut s = String::new();
    loop {
        unsafe {
            let val = *(phy_addr as *const u8);
            if val == 0 {
                break;
            }
            phy_addr += 1;
            s.push(val as char);
        }
    }
    s
}

/// translate a pointer to a mutable u8 Vec through page table
pub fn translated_byte_buffer(task: &Arc<Task>, ptr: usize, len: usize) -> Vec<&'static mut [u8]> {
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = task
            .inner_exclusive_access()
            .addr_space
            .translate(vpn)
            .unwrap()
            .ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}
