use alloc::sync::Weak;

use crate::{
    mm::{MapPermission, VirtAddr},
    task::Task,
};

use super::SyscallResult;

pub fn sys_mmap(task: &Weak<Task>, start: usize, len: usize, port: usize) -> SyscallResult {
    let task = Task::from_weak(&task);
    log::info!(
        "task_{}({}) sys_mmap, receive args start=0x{:x}, end=0x{:x}, len=0x{:x}, port=0x{:x}",
        task.pid,
        task.name,
        start,
        start + len,
        len,
        port
    );
    if port & !0x7 != 0 {
        log::info!(
            "task_{}({}) sys_mmap failed, receive bad port? port=0x{:x}",
            task.pid,
            task.name,
            port
        );
        return Err(());
    }
    let perm = MapPermission::U
        | match port {
            7 => MapPermission::X | MapPermission::W | MapPermission::R,
            4 => MapPermission::X | MapPermission::R,
            3 => MapPermission::W | MapPermission::R,
            2 => MapPermission::W,
            1 => MapPermission::R,
            _ => {
                log::info!(
                    "task_{}({}) sys_mmap failed, receive meaningless port? port=0x{:x}",
                    task.pid,
                    task.name,
                    port
                );
                return Err(());
            }
        };

    let end = VirtAddr::from(start + len);
    let start = VirtAddr::from(start);
    if start.page_offset() != 0 {
        return Err(());
    };
    let mut inner = task.inner_exclusive_access();
    inner
        .addr_space
        .insert_framed_area(start, end, perm)
        .map(|_| 0)
}

pub fn sys_unmmap(task: &Weak<Task>, start: usize, len: usize) -> SyscallResult {
    let task = Task::from_weak(&task);
    log::info!(
        "task_{}({}) sys_unmmap, receive args start=0x{:x}, len=0x{:x}",
        task.pid,
        task.name,
        start,
        len
    );
    let end = VirtAddr::from(start + len);
    let start = VirtAddr::from(start);
    if start.page_offset() != 0 {
        return Err(());
    }
    let mut inner = task.inner_exclusive_access();
    inner
        .addr_space
        .unmap_area(task.pid.clone(), start, end)
        .map(|_| 0)
}
