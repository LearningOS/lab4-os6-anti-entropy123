use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::{
    config::MAX_SYSCALL_NUM,
    syscall::pointer::{from_user_cstring, from_user_ptr},
    task::{add_task, fork_task, run_next_task, switch_task, Task, TaskState},
    timer::{self, get_time_ms},
};

use super::SyscallResult;

pub fn sys_yield(task: Arc<Task>) -> ! {
    {
        task.inner_exclusive_access().set_state(TaskState::Ready)
    }
    switch_task(task)
}

pub fn sys_exit(task: Arc<Task>, exit_code: i32) -> ! {
    {
        let mut inner = task.inner_exclusive_access();
        inner.set_state(TaskState::Exited);
        inner.exit_code = exit_code;
    }
    log::info!(
        "{}, ready to exit, exit_code={}, Arc count={}",
        task,
        exit_code,
        Arc::strong_count(&task)
    );
    drop(task);
    run_next_task()
}

#[derive(Debug)]
pub struct TaskInfo {
    pub state: TaskState,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub exec_time: usize,
}

pub fn sys_taskinfo(task: &Weak<Task>, user_info: usize) -> SyscallResult {
    let task = Task::from_weak(&task);
    let syscall_times = {
        let inner = task.inner_exclusive_access();
        inner.syscall_times
    };
    let taskinfo = from_user_ptr(&task, user_info);
    *taskinfo = TaskInfo {
        state: TaskState::Running,
        syscall_times,
        exec_time: get_time_ms() - task.start_time_ms,
    };
    log::debug!(
        "task_{}({}) sys_taskinfo, copyout user_info={:?}",
        task.pid,
        task.name,
        taskinfo
    );
    Ok(0)
}

pub fn sys_gettimeofday(task: &Weak<Task>, timeval_ptr: usize, _tz: usize) -> SyscallResult {
    let task = Task::from_weak(&task);
    let time = from_user_ptr(&task, timeval_ptr);
    timer::set_time_val(time);
    Ok(0)
}

pub fn sys_fork(task: &Weak<Task>) -> SyscallResult {
    let task = Task::from_weak(&task);
    let child = fork_task(&task);
    let child_pid = child.pid.0;
    task.inner_exclusive_access().children.push(child);
    Ok(child_pid as isize)
}

pub fn sys_waitpid(task: &Weak<Task>, target_pid: isize, exit_code: usize) -> SyscallResult {
    let task = Task::from_weak(&task);
    let target_children_pid = {
        let children = { &task.inner_exclusive_access().children };

        let target_children: Vec<&Arc<Task>> = if target_pid == -1 {
            children.iter().collect()
        } else {
            children
                .iter()
                .filter(|t| t.pid.0 == target_pid as usize)
                .collect()
        };
        if target_children.is_empty() {
            return Ok(-1);
        }
        let exited_children: Vec<&Arc<Task>> = target_children
            .iter()
            .map(|t| *t)
            .filter(|t| t.inner_exclusive_access().state == TaskState::Exited)
            .collect();

        if exited_children.is_empty() {
            return Ok(-2);
        }
        (*exited_children.get(0).unwrap()).pid.clone()
    };

    let target_child = {
        let mut inner = task.inner_exclusive_access();
        let (idx, _) = inner
            .children
            .iter()
            .enumerate()
            .find(|(_, t)| t.pid == target_children_pid)
            .expect("should have this pid child");
        inner.children.remove(idx)
    };
    log::info!(
        "{}, have been wait, arc count={}",
        target_child,
        Arc::strong_count(&target_child)
    );

    let exit_code: &mut i32 = from_user_ptr(&task, exit_code);
    *exit_code = {
        let child_inner = target_child.inner_exclusive_access();
        assert!(target_child.pid == target_children_pid);
        assert!(child_inner.state == TaskState::Exited);
        child_inner.exit_code
    };

    Ok(target_children_pid.0 as isize)
}

pub fn sys_getpid(task: &Weak<Task>) -> SyscallResult {
    let task = Task::from_weak(&task);
    Ok(task.pid.0 as isize)
}

pub fn sys_set_priority(task: &Weak<Task>, priority: isize) -> SyscallResult {
    let task = Task::from_weak(&task);
    if priority > 1 {
        task.inner_exclusive_access().priority = priority as u32;
        Ok(priority)
    } else {
        Err(())
    }
}

pub fn sys_exec(task: &Weak<Task>, path: usize) -> SyscallResult {
    let task = Task::from_weak(&task);
    let path = from_user_cstring(&task, path);
    log::info!("sys_exec, {}, target app={}", task, path);
    task.exec(&path)?;
    // drop(path);
    // drop(task);
    // restore(pop_cur_task().unwrap());
    Ok(0)
}

pub fn sys_spawn(task: &Weak<Task>, path: usize) -> SyscallResult {
    let task = Task::from_weak(&task);
    let path = from_user_cstring(&task, path);
    log::info!("sys_spawn, {}, target app={}", task, path);
    let child = Task::spawn(&path)?;
    let child_pid = child.pid.0;
    task.inner_exclusive_access()
        .children
        .push(Arc::clone(&child));
    add_task(child);
    Ok(child_pid as isize)
}
