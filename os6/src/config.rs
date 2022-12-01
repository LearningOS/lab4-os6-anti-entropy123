//! Constants used in rCore

// base
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;
pub const CLOCK_FREQ: usize = 12500000;
pub const MMIO: &[(usize, usize)] = &[
    (0x10001000, 0x1000),
];

// kernel space config
pub const KERNEL_STACK_PAGE_NUM: usize = 15;
pub const KERNEL_STACK_SIZE: usize = PAGE_SIZE * KERNEL_STACK_PAGE_NUM;
pub const KERNEL_HEAP_SIZE: usize = PAGE_SIZE * 4096;
pub const MEMORY_END: usize = 0x88000000;

// syscall/user config
pub const MAX_SYSCALL_NUM: usize = 500;
#[allow(dead_code)]
pub const BIG_STRIDE: usize = 500000;

// user space config
pub const USER_STACK_PAGE_NUM: usize = 20;
pub const USER_STACK_SIZE: usize = 4096 * USER_STACK_PAGE_NUM;
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;


