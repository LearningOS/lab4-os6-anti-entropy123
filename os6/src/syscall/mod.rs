mod fs;
mod mm;
mod pointer;
mod proc;

use alloc::{string::ToString, sync::Weak};

pub use crate::syscall::proc::sys_exit;
use crate::{
    syscall::{
        fs::{sys_read, sys_write},
        mm::{sys_mmap, sys_unmmap},
        proc::{
            sys_exec, sys_fork, sys_getpid, sys_gettimeofday, sys_set_priority, sys_spawn,
            sys_taskinfo, sys_waitpid, sys_yield,
        },
    },
    task::{pop_cur_task, Task},
};

#[derive(Debug)]
enum Syscall {
    Read,
    Write,
    Exit,
    Yield,
    SetPriority,
    GetTimeOfDay,
    GetPid,
    Munmap,
    Fork,
    Exec,
    Mmap,
    WaitPid,
    Spawn,
    TaskInfo,
}

impl Syscall {
    fn from(n: usize) -> Result<Syscall, ()> {
        Ok(match n {
            63 => Self::Read,          // 0x3f
            64 => Self::Write,         // 0x40
            93 => Self::Exit,          // 0x5d
            124 => Self::Yield,        // 0x7c
            140 => Self::SetPriority,  // 0x8c
            169 => Self::GetTimeOfDay, // 0xa9
            172 => Self::GetPid,       // 0xac
            215 => Self::Munmap,       // 0xd7
            220 => Self::Fork,         // 0xdc
            221 => Self::Exec,         // 0xdd
            222 => Self::Mmap,         // 0xde
            260 => Self::WaitPid,      // 0x104
            400 => Self::Spawn,        // 0x190
            410 => Self::TaskInfo,     // 0x19a
            _ => {
                log::warn!("unsupported syscall: {}", n.to_string());
                return Err(());
            }
        })
    }
}

type SyscallResult = Result<isize, ()>;

impl Syscall {
    fn handle(&self, task: &Weak<Task>, arg1: usize, arg2: usize, arg3: usize) {
        let ret: SyscallResult = match self {
            Syscall::Write => sys_write(task, arg1, arg2, arg3),
            Syscall::Exit => sys_exit(Task::from_weak(&task), arg1 as i32),
            Syscall::GetTimeOfDay => sys_gettimeofday(task, arg1, arg2),
            Syscall::Yield => sys_yield(Task::from_weak(&task)),
            Syscall::TaskInfo => sys_taskinfo(task, arg1),
            Syscall::Mmap => sys_mmap(task, arg1, arg2, arg3),
            Syscall::Munmap => sys_unmmap(task, arg1, arg2),
            Syscall::Fork => sys_fork(task),
            Syscall::WaitPid => sys_waitpid(task, arg1 as isize, arg2),
            Syscall::GetPid => sys_getpid(task),
            Syscall::Read => sys_read(task.upgrade().unwrap(), arg1, arg2, arg3),
            Syscall::SetPriority => sys_set_priority(task, arg1 as isize),
            Syscall::Exec => sys_exec(task, arg1),
            Syscall::Spawn => sys_spawn(task, arg1),
            // _ => todo!("unsupported syscall handle function, syscall={:?}", self),
        };
        let ret = ret.unwrap_or(-1);
        let task = Task::from_weak(&task);
        let a0 = {
            let inner = task.inner_exclusive_access();
            let trap_ctx = inner.trap_context();
            trap_ctx.set_reg_a(0, ret as usize);
            trap_ctx.reg_a(0)
        };
        log::info!(
            "task_{} syscall ret={:x}, task.trap_ctx.x[10]={:x}",
            task.pid,
            ret,
            a0
        );
    }
}

pub fn syscall_handler(weak_task: &Weak<Task>) {
    let (syscall_num, a0, a1, a2) = {
        let task = Task::from_weak(&weak_task);
        let mut inner = task.inner_exclusive_access();
        let trap_ctx = inner.trap_context();
        let syscall_num = trap_ctx.reg_a(7);
        let ret = (
            syscall_num,
            trap_ctx.reg_a(0),
            trap_ctx.reg_a(1),
            trap_ctx.reg_a(2),
        );
        inner.syscall_times[syscall_num] += 1;
        ret
    };

    let syscall =
        Syscall::from(syscall_num).unwrap_or_else(|_| sys_exit(pop_cur_task().unwrap(), 1));

    {
        let task = Task::from_weak(&weak_task);
        log::info!(
            "{} syscall_handler, num={}, name={:?}",
            task,
            syscall_num,
            syscall
        );
    }
    // log::info!("syscall_times={:?}", ctx.syscall_times);
    syscall.handle(weak_task, a0, a1, a2)
}
