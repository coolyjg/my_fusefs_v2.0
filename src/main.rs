mod file;
// use std::sync::Arc;
use std::env;
// use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory};
// use std::ffi::OsStr;
use crate::file::MyFuseFs;
use crate::file::RamFs;
use std::ffi::CString;

mod ffi_wrapper;

// pub use crate::file::MY_POP_DATA;
// pub static mut MY_POP_DATA: Option<*mut PMEMobjpool>= None;
// use crate::ffi_wrapper::PMEMobjpool;
pub use crate::ffi_wrapper::MY_POP_DATA;
pub use crate::ffi_wrapper::MY_POP_MD;
fn main() {
    // env_logger::init();
    // let mountpoint = env::args_os().nth(1).unwrap();
    // let pool_addr = env::args_os().nth(2).unwrap();
    // MY_POP_DATA = ffi_wrapper::my_init(pool_addr);
    let args: Vec<String> = env::args().collect();
    let mountpoint = &args[1];
    let pool_addr: CString = CString::new("data").unwrap();
    let md_pool_addr: CString = CString::new("metadata").unwrap();
    // let pool_layout: CString = CString::new("content_store").unwrap();
    // let pop = ffi::my_init(pool_layout.as_ptr() as *const u8);
    // unsafe{
    //     MY_POP_DATA =match ffi_wrapper::my_init(pool_addr.as_ptr()){
    //         val => Some(val),
    //     // _ => None,
    //     }
    // };
    unsafe{
        MY_POP_DATA = Some(ffi_wrapper::my_init(pool_addr.as_ptr() as *const u8));
        MY_POP_MD = Some(ffi_wrapper::my_init_md(md_pool_addr.as_ptr() as *const u8));
    }
    // let options = ["-o", "ro", "-o", "fsname=hello"]
    //     .iter()
    //     .map(|o| o.as_ref())
    //     .collect::<Vec<&OsStr>>();
    // fuse::mount(*(MyFuseFs::new()).clone(), &mountpoint, &options).expect("failed to mount");
    let fs = RamFs::new();
    fuse::mount(
        MyFuseFs::new(fs),
        &mountpoint,
        &[]
    ).expect("fail to mount");
}