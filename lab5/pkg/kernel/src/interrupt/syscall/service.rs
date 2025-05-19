use core::alloc::Layout;

use crate::proc;
use crate::proc::*;
use crate::utils::*;

use super::SyscallArgs;

pub fn spawn_process(args: &SyscallArgs) -> usize {
    // 从参数获取应用程序名称
    let app_name = unsafe {
        let ptr = args.arg0 as *const u8;
        let len = args.arg1;
        let slice = core::slice::from_raw_parts(ptr, len);
        core::str::from_utf8_unchecked(slice)
    };

    // 通过名称创建进程
    let res = proc::spawn(app_name);
    match res {
        Some(pid) => pid.0 as usize,
        None => 0,
    }
}

pub fn sys_write(args: &SyscallArgs) -> usize {
    // 获取文件描述符和缓冲区
    let fd = args.arg0 as u8;
    let buf = unsafe {
        let ptr = args.arg1 as *const u8;
        let len = args.arg2;
        core::slice::from_raw_parts(ptr, len)
    };

    proc::write(fd, buf) as usize
}

pub fn sys_read(args: &SyscallArgs) -> usize {
    // 获取文件描述符和缓冲区
    let fd = args.arg0 as u8;
    let buf = unsafe {
        let ptr = args.arg1 as *mut u8;
        let len = args.arg2;
        core::slice::from_raw_parts_mut(ptr, len)
    };

    // 调用读取函数并返回读取的字节数
    proc::read(fd, buf) as usize
}

pub fn exit_process(args: &SyscallArgs, _context: &mut ProcessContext) {
    proc::exit(args.arg0 as isize, _context)
}

pub fn list_process() {
    // 列出所有进程
    proc::print_process_list();
}

pub fn sys_get_current_pid() -> usize {
    // 获取当前进程ID
    proc::get_current_pid() as usize
}

pub fn sys_wait_pid(args: &SyscallArgs) -> usize {
    match proc::wait_pid(args.arg0 as u16) {
        Some(code) => code as usize,
        None => 1919810,
    }
}

pub fn list_apps() {
    proc::list_app();
}

pub fn sys_allocate(args: &SyscallArgs) -> usize {
    let layout = unsafe { (args.arg0 as *const Layout).as_ref().unwrap() };

    if layout.size() == 0 {
        return 0;
    }

    let ret = crate::memory::user::USER_ALLOCATOR
        .lock()
        .allocate_first_fit(*layout);

    match ret {
        Ok(ptr) => ptr.as_ptr() as usize,
        Err(_) => 0,
    }
}

pub fn sys_deallocate(args: &SyscallArgs) {
    let layout = unsafe { (args.arg1 as *const Layout).as_ref().unwrap() };

    if args.arg0 == 0 || layout.size() == 0 {
        return;
    }

    let ptr = args.arg0 as *mut u8;

    unsafe {
        crate::memory::user::USER_ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), *layout);
    }
}
pub fn sys_fork(context: &mut ProcessContext) {
    proc::fork(context);
}
