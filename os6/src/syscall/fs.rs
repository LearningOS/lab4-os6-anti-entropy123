use alloc::sync::{Arc, Weak};

use crate::{
    fs::{link_at, open_file, unlink_at, OpenFlags, ROOT_INODE},
    mm::UserBuffer,
    task::Task,
};

use super::{
    pointer::{from_user_cstring, from_user_ptr, translated_byte_buffer},
    SyscallResult,
};

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

#[repr(C)]
#[derive(Debug)]
pub struct Stat {
    /// 文件所在磁盘驱动器号，该实验中写死为 0 即可
    pub dev: u64,
    /// inode 文件所在 inode 编号
    pub ino: u64,
    /// 文件类型
    pub mode: StatMode,
    /// 硬链接数量，初始为1
    pub nlink: u32,
    /// 无需考虑，为了兼容性设计
    pad: [u64; 7],
}

bitflags! {
    pub struct StatMode: u32 {
        const NULL  = 0;
        /// directory
        const DIR   = 0o040000;
        /// ordinary regular file
        const FILE  = 0o100000;
    }
}

pub fn sys_fstat(task: &Weak<Task>, fd: i32, st_user: usize) -> SyscallResult {
    let task = Task::from_weak(&task);
    let stat = from_user_ptr(&task, st_user);
    let (ino, mode, nlink) = {
        /*
         * 这里需要考虑两种情况:
         * 1. fd 超过目前 fd_table 的长度, 可能是因为还没有打开目标文件
         * 2. fd 在 fd_table 里, 但是为 None, 可能是因为目标文件已经被关闭了.
         */
        let target_file = {
            let fd_table = &task.inner_exclusive_access().fd_table;
            let len = fd_table.len() as i32;
            if len <= fd {
                log::warn!(
                    "{}, sys_fstat, fd={} fd_table len={}, didn't have target file?",
                    task,
                    fd,
                    len
                );
                return Err(());
            }
            if let Some(file) = fd_table.get(fd as usize).unwrap() {
                Arc::clone(file)
            } else {
                log::warn!(
                    "{}, sys_fstat, fd={}, has file slot but is None, file has been close?",
                    task,
                    fd,
                );
                return Err(());
            }
        };

        if let Some(inode) = target_file.inode() {
            (
                inode.inode_id as u64,
                if inode.is_dir() {
                    StatMode::DIR
                } else {
                    StatMode::FILE
                },
                inode.link_cnt(),
            )
        } else {
            log::warn!("{}, sys_fstat, wrong fd? fd={}", task, fd);
            return Err(());
        }
    };
    log::info!(
        "{}, sys_fstat, fstat finish, ino={}, mode={:?}, nlink={}",
        task,
        ino,
        mode,
        nlink
    );
    *stat = Stat {
        dev: 0,
        ino,
        mode,
        nlink,
        pad: [0; 7],
    };
    Ok(0)
}

pub fn sys_link_at(
    task: &Weak<Task>,
    olddirfd: i32,
    oldpath: usize,
    newdirfd: i32,
    newpath: usize,
    flags: u32,
) -> SyscallResult {
    (drop(olddirfd), drop(newdirfd), drop(flags));
    let task = Task::from_weak(task);
    let (oldpath, newpath) = (
        from_user_cstring(&task, oldpath),
        from_user_cstring(&task, newpath),
    );

    link_at(&newpath, &oldpath).map(|_| 0)
}

pub fn sys_unlink_at(task: &Weak<Task>, dirfd: i32, path: usize, flags: u32) -> SyscallResult {
    (drop(dirfd), drop(flags));
    let task = Task::from_weak(task);
    let path = from_user_cstring(&task, path);
    unlink_at(&path).map(|_| 0)
}

pub fn sys_open_at(
    task: &Weak<Task>,
    _at_fdcwd: usize,
    path: usize,
    flags: u32,
    _mode: u32,
) -> SyscallResult {
    let task = Task::from_weak(task);
    let path = from_user_cstring(&task, path);
    let file = open_file(&path, OpenFlags::from_bits(flags).ok_or(())?);
    match file {
        Some(file) => {
            let mut inner = task.inner_exclusive_access();
            inner.fd_table.push(Some(file));
            Ok(inner.fd_table.len() as isize - 1)
        }
        None => {
            log::warn!("{}, sys_open_at, wrong path? path={}", task, path);
            Err(())
        }
    }
}

pub fn sys_close(task: &Weak<Task>, fd: usize) -> SyscallResult {
    let task = Task::from_weak(&task);
    let mut inner = task.inner_exclusive_access();
    match inner.fd_table.remove(fd) {
        Some(_) => Ok(0),
        None => Err(()),
    }
}
