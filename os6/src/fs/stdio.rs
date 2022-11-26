use crate::{mm::UserBuffer, sbi::console_getchar, task::{pop_cur_task, switch_task}};

use super::File;

pub struct Stdin;
pub struct Stdout;

impl File for Stdin {
    fn readable(&self) -> bool { true }
    fn writable(&self) -> bool { false }
    fn read(&self, mut user_buf: UserBuffer) -> usize {
        assert_eq!(user_buf.len(), 1);
        // busy loop
        let c: usize;
        loop {
            c = console_getchar();
            if c == 0 {
                switch_task(pop_cur_task().unwrap());
            } else {
                break;
            }
        }
        let ch = c as u8;
        unsafe { user_buf.buffers[0].as_mut_ptr().write_volatile(ch); }
        1
    }
    fn write(&self, _user_buf: UserBuffer) -> usize {
        panic!("Cannot write to stdin!");
    }
}

impl File for Stdout {
    fn readable(&self) -> bool { false }
    fn writable(&self) -> bool { true }
    fn read(&self, _user_buf: UserBuffer) -> usize{
        panic!("Cannot read from stdout!");
    }
    fn write(&self, user_buf: UserBuffer) -> usize {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        user_buf.len()
    }
}