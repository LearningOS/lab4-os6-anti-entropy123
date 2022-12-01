use alloc::{format, string::String, sync::Arc, vec::Vec};

use crate::{
    mm::{address::StepByOne, VirtAddr},
    task::Task,
};

pub fn from_user_ptr<T>(task: &Arc<Task>, user_addr: usize) -> &'static mut T {
    let phy_addr = task.inner_exclusive_access().translate(user_addr);

    let phy_addr = if let Some(addr) = phy_addr {
        addr
    } else {
        panic!(
            "{}, receive bad user addr? user_addr=0x{:x}",
            task, user_addr
        )
    };

    unsafe { &mut *(phy_addr as *mut T) }
}

pub fn from_user_cstring(task: &Arc<Task>, user_addr: usize) -> String {
    // let mut user_addr = user_addr;
    let phy_addr = task.inner_exclusive_access().translate(user_addr);

    let mut phy_addr = if let Some(addr) = phy_addr {
        addr
    } else {
        log::warn!(
            "{}, receive bad user addr? user_addr=0x{:x}",
            task,
            user_addr
        );
        panic!();
    };

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
