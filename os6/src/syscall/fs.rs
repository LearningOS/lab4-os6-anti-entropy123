use alloc::sync::{Arc, Weak};

use crate::{mm::UserBuffer, task::Task};

use super::{pointer::translated_byte_buffer, SyscallResult};

pub fn sys_write(task: &Weak<Task>, fd: usize, buf: usize, len: usize) -> SyscallResult {
    let task = Task::from_weak(&task);
    let buffer = UserBuffer::new(translated_byte_buffer(&task, buf, len));
    let inner = task.inner_exclusive_access();
    let file = if let Some(Some(file)) = inner.fd_table.get(fd) {
        Arc::clone(file)
    } else {
        log::error!("{}, sys_write, user pass a bad fd? fd={}", task, fd);
        return Err(());
    };
    drop(inner);
    Ok(file.write(buffer) as isize)
}

pub fn sys_read(task: Arc<Task>, fd: usize, buf: usize, len: usize) -> SyscallResult {
    let buffer = UserBuffer::new(translated_byte_buffer(&task, buf, len));
    let inner = task.inner_exclusive_access();
    let file = if let Some(Some(file)) = inner.fd_table.get(fd) {
        Arc::clone(file)
    } else {
        log::error!("{}, sys_read, user pass a bad fd? fd={}", task, fd);
        return Err(());
    };
    drop(inner);
    Ok(file.read(buffer) as isize)
}
