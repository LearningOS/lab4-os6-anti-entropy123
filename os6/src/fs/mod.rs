mod inode;
mod stdio;

use crate::mm::UserBuffer;
use alloc::sync::Arc;
use easy_fs::Inode;
pub use inode::{link_at, open_file, unlink_at, OSInode, OSInodeInner, OpenFlags, ROOT_INODE};
pub use stdio::{Stdin, Stdout};

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
    fn inode(&self) -> Option<Arc<Inode>> {
        None
    }
}
