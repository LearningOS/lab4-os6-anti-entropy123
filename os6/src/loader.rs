use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::fs::{open_file, File, OpenFlags, ROOT_INODE};

pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

fn get_appid_by_name(name: &str) -> Result<usize, ()> {
    (0..get_num_app()).find(|&i| APP_NAMES[i] == name).ok_or(())
}

pub fn _get_app_elf(name: &str) -> Result<&'static [u8], ()> {
    extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    let app_id = match get_appid_by_name(name) {
        Ok(id) => id,
        Err(_) => {
            log::error!("wrong app name? name={}", name);
            return Err(());
        }
    };
    unsafe {
        Ok(core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        ))
    }
}

lazy_static! {
    static ref APP_NAMES: Vec<&'static str> = {
        let num_app = get_num_app();
        extern "C" {
            fn _app_names();
        }
        let mut start = _app_names as usize as *const u8;
        let mut v = Vec::new();
        unsafe {
            for _ in 0..num_app {
                let mut end = start;
                while end.read_volatile() != b'\0' {
                    end = end.add(1);
                }
                let slice = core::slice::from_raw_parts(start, end as usize - start as usize);
                let str = core::str::from_utf8(slice).unwrap();
                v.push(str);
                start = end.add(1);
            }
        }
        v
    };
}

pub fn list_apps() {
    println!("/**** APPS ****");
    for app in APP_NAMES.iter() {
        println!("{}", app);
    }
    println!("**************/");
}

pub fn list_efs_apps() {
    println!("/**** APPS ****");
    for app in ROOT_INODE.ls() {
        println!("{}", app);
    }
    println!("**************/");
}

pub fn efs_get_app_elf(name: &str) -> Result<Vec<u8>, ()> {
    let all_data = match open_file(name, OpenFlags::RDONLY) {
        Some(app_inode) => app_inode.read_all(),
        None => panic!("wrong app name? {}", name),
    };
    Ok(all_data)
}
