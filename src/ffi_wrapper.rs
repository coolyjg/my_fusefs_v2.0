extern crate libc;

use libc::c_int;
use libc::size_t;
use libc::c_void;
use libc::c_char;
use libc::c_long;
// use std::ffi::CString;

pub enum PMEMobjpool {}
pub static mut MY_POP_DATA: Option<*mut PMEMobjpool>= None;
pub static mut MY_POP_MD: Option<*mut PMEMobjpool>= None;

#[link(name = "cnt_store")]
#[link(name = "pmemobj")]
extern "C"{
    fn create_content(pop: *mut PMEMobjpool, size: size_t, buf: *mut c_char, id: c_int)->c_void;
    // fn print_content(pop: *mut PMEMobjpool, id: c_int)->c_void;
    // fn print_content_all(pop: *mut PMEMobjpool)->c_void;
    fn my_free_fn(pop: *mut PMEMobjpool, id: c_int)->c_void;
    // fn my_free_all_fn(pop: *mut PMEMobjpool)->c_void;
    fn init(path:*const c_char)->*mut PMEMobjpool;
    fn fin(pop:*mut PMEMobjpool)->c_void;
    fn write_at_content(pop: *mut PMEMobjpool, id: c_int, offset: c_int, buf: *mut c_char, size: size_t)->c_int;
    // fn write_content(pop: *mut PMEMobjpool, id: c_int, buf: *mut c_char, size: size_t)->c_int;
    fn read_at_content(pop: *mut PMEMobjpool, id: c_int, offset: c_int, buf: *mut c_char)->c_int;
    // fn read_content(pop: *mut PMEMobjpool, id: c_int, buf: *mut c_char)->c_int;
}

pub fn my_create_content(pop: *mut PMEMobjpool, size: usize, buf: *const u8, id: u32)->c_void{
    unsafe{
        create_content(pop, size, buf as *mut c_char, id as c_int)
    }
}
// 
// pub fn my_print_content(pop: *mut PMEMobjpool, id: u32)->c_void{
//     unsafe{
//         print_content(pop, id as c_int)
//     }
// }

// pub fn my_print_content_all(pop: *mut PMEMobjpool)->c_void{
//     unsafe{
//         print_content_all(pop)
//     }
// }

pub fn my_free(pop: *mut PMEMobjpool, id: u32)->c_void{
    unsafe{
        my_free_fn(pop, id as c_int)
    }
}

// pub fn my_free_all(pop: *mut PMEMobjpool)->c_void{
//     unsafe{
//         my_free_all_fn(pop)
//     }
// }
// 
pub fn my_init(path:*const u8)->*mut PMEMobjpool{
    unsafe{
        init(path as *const c_char)
    }
}

pub fn my_fin(pop:*mut PMEMobjpool)->c_void{
    unsafe{
        fin(pop)
    }
}

pub fn my_write_at_content(pop: *mut PMEMobjpool, id: u32, offset: usize, buf: *const u8, size: usize)->c_int{
    unsafe{
        write_at_content(pop, id as c_int, offset as c_int, buf as *mut c_char, size as size_t)
    }
}

// pub fn my_write_content(pop: *mut PMEMobjpool, id: u32, buf: *mut c_char, size: usize)->c_int{
//     unsafe{
//         write_content(pop, id as c_int, buf as *mut c_char, size as size_t)
//     }
// }

pub fn my_read_at_content(pop: *mut PMEMobjpool, id: u32, offset: usize, buf: *const u8)->c_int{
    unsafe{
        read_at_content(pop, id as c_int, offset as c_int, buf as *mut c_char)
    }
}

// pub fn my_read_content(pop: *mut PMEMobjpool, id: u32, buf: *mut c_char)->c_int{
//     unsafe{
//         read_content(pop, id as c_int, buf as *mut c_char)
//     }
// }

#[repr(C)]
pub enum Filetype{
    Directory,
    Regular
}


#[link(name = "cnt_store")]
#[link(name = "pmemobj")]
extern "C"{
    fn create_metadata(
        pop: *mut PMEMobjpool, 
        ino: c_int, 
        size: size_t, 
        blocks: c_int, 
        atime: c_long, 
        mtime: c_long, 
        ctime: c_long, 
        crtime: c_long,
        kind: Filetype,
        perm: c_int,
        nlink: c_int,
        uid: c_int,
        gid: c_int,
        rdev: c_int,
        flags: c_int
    )->c_void;
    fn set_metadata_size(pop: *mut PMEMobjpool, ino: c_int, size: size_t)->c_void;
    // fn set_blocks(pop: *mut PMEMobjpool, ino: c_int, blocks: c_int)->c_void;
    fn set_atime(pop: *mut PMEMobjpool, ino: c_int, atime: c_long)->c_void;
    fn set_mtime(pop: *mut PMEMobjpool, ino: c_int, mtime: c_long)->c_void;
    fn set_ctime(pop: *mut PMEMobjpool, ino: c_int, ctime: c_long)->c_void;
    fn set_crtime(pop: *mut PMEMobjpool, ino: c_int, crtime: c_long)->c_void;
    fn set_kind(pop: *mut PMEMobjpool, ino: c_int, kind: Filetype)->c_void;
    fn set_perm(pop: *mut PMEMobjpool, ino: c_int, perm: c_int)->c_void;
    fn get_metadata_size(pop: *mut PMEMobjpool, ino: c_int)->c_int;
    // fn get_metadata_blocks(pop: *mut PMEMobjpool, ino: c_int)->c_int;
    fn get_metadata_atime(pop: *mut PMEMobjpool, ino: c_int)->c_long;
    fn get_metadata_mtime(pop: *mut PMEMobjpool, ino: c_int)->c_long;
    fn get_metadata_ctime(pop: *mut PMEMobjpool, ino: c_int)->c_long;
    fn get_metadata_crtime(pop: *mut PMEMobjpool, ino: c_int)->c_long;
    fn get_metadata_kind(pop: *mut PMEMobjpool, ino: c_int)->Filetype;
    fn get_metadata_perm(pop: *mut PMEMobjpool, ino: c_int)->c_int;
    fn get_metadata_nlink(pop: *mut PMEMobjpool, ino: c_int)->c_int;
    // fn get_metadata(pop: *mut PMEMobjpool, ino: c_int)-> *struct metadata;
    // fn set_nlink(pop: *mut PMEMobjpool, ino: c_int, nlink: c_int)->c_void;
    // fn set_metadata(
        // pop: *mut PMEMobjpool, 
        // ino: c_int, 
        // size: size_t, 
        // blocks: c_int, 
        // atime: c_long, 
        // mtime: c_long, 
        // ctime: c_long, 
        // crtime: c_long,
        // kind: Filetype,
        // perm: c_int,
        // nlink: c_int,
        // uid: c_int,
        // gid: c_int,
        // rdev: c_int,
        // flags: c_int
    // )->c_void;
    fn free_metadata(pop: *mut PMEMobjpool, id: c_int)->c_void;
    fn init_md(path: *const c_char)->*mut PMEMobjpool;
}

pub fn my_create_metadata(
    pop: *mut PMEMobjpool,
    ino: u32,
    size: usize,
    blocks: u32,
    atime: u64,
    mtime: u64,
    ctime: u64,
    crtime: u64,
    kind: Filetype,
    perm: u32,
    nlink: u32,
    uid: u32,
    gid: u32,
    rdev: u32,
    flags: u32
)->c_void{
    unsafe{
        create_metadata(
            pop,
            ino as c_int,
            size as size_t,
            blocks as c_int,
            atime as c_long,
            mtime as c_long,
            ctime as c_long,
            crtime as c_long,
            kind as Filetype, 
            perm as c_int,
            nlink as c_int,
            uid as c_int,
            gid as c_int,
            rdev as c_int,
            flags as c_int
        )
    }
}

pub fn my_set_metadata_size(pop: *mut PMEMobjpool, id: u32, size: usize)->c_void{
    unsafe{
        set_metadata_size(pop, id as c_int, size as size_t)
    }
}

pub fn my_get_size(pop: *mut PMEMobjpool, id: u32)->usize{
    unsafe{
        get_metadata_size(pop, id as c_int) as usize
    }
}

// pub fn my_set_blocks(pop: *mut PMEMobjpool, id: u32, blocks: u32)->c_void{
//     unsafe{
//         set_blocks(pop, id as c_int, blocks as c_int)
//     }
// }

// pub fn my_get_blocks(pop: *mut PMEMobjpool, id: u32)->c_int{
//     unsafe{
//         get_metadata_blocks(pop, id as c_int)
//     }
// }

pub fn my_set_atime(pop: *mut PMEMobjpool, id: u32, atime: u64)->c_void{
    unsafe{
        set_atime(pop, id as c_int, atime as c_long)
    }
}

pub fn my_get_atime(pop: *mut PMEMobjpool, id: u32)->c_long{
    unsafe{
        get_metadata_atime(pop, id as c_int)
    }
}

pub fn my_set_mtime(pop: *mut PMEMobjpool, id: u32, mtime: u64)->c_void{
    unsafe{
        set_mtime(pop, id as c_int, mtime as c_long)
    }
}

pub fn my_get_mtime(pop: *mut PMEMobjpool, id: u32)->c_long{
    unsafe{
        get_metadata_mtime(pop, id as c_int)
    }
}

pub fn my_set_ctime(pop: *mut PMEMobjpool, id: u32, ctime: u64)->c_void{
    unsafe{
        set_ctime(pop, id as c_int, ctime as c_long)
    }
}

pub fn my_get_ctime(pop: *mut PMEMobjpool, id: u32)->c_long{
    unsafe{
        get_metadata_ctime(pop, id as c_int)
    }
}

pub fn my_set_crtime(pop: *mut PMEMobjpool, id: u32, crtime: u64)->c_void{
    unsafe{
        set_crtime(pop, id as c_int, crtime as c_long)
    }
}

pub fn my_get_crtime(pop: *mut PMEMobjpool, id: u32)->c_long{
    unsafe{
        get_metadata_crtime(pop, id as c_int)
    }
}

pub fn my_set_kind(pop: *mut PMEMobjpool, id: u32, kind: Filetype)->c_void{
    unsafe{
        set_kind(pop, id as c_int, kind)
    }
}

pub fn my_get_kind(pop: *mut PMEMobjpool, id: u32)->Filetype{
    unsafe{
        get_metadata_kind(pop, id as c_int)
    }
}

pub fn my_set_perm(pop: *mut PMEMobjpool, id: u32, perm: u32)->c_void{
    unsafe{
        set_perm(pop, id as c_int, perm as c_int)
    }
}

pub fn my_get_perm(pop: *mut PMEMobjpool, id: u32)->c_int{
    unsafe{
        get_metadata_perm(pop, id as c_int)
    }
}

pub fn my_get_nlink(pop: *mut PMEMobjpool, id: u32)->c_int{
    unsafe{
        get_metadata_nlink(pop, id as c_int)
    }
}

// pub fn my_get_metadata(pop: *mut PMEMobjpool, id: u32)-> *struct metadata{
//     unsafe{
//         set_metadata(pop, id as c_int)
//     }
// }

// pub fn my_set_nlink(pop: *mut PMEMobjpool, id: u32, nlink: u32)->c_void{
//     unsafe{
//         set_nlink(pop, id as c_int, nlink as c_int)
//     }
// }

// pub fn my_set_metadata(
//     pop: *mut PMEMobjpool,
//     ino: u32,
//     size: usize,
//     blocks: u32,
//     atime: u64,
//     mtime: u64,
//     ctime: u64,
//     crtime: u64,
//     kind: Filetype,
//     perm: u32,
//     nlink: u32,
//     uid: u32,
//     gid: u32,
//     rdev: u32,
//     flags: u32
// )->c_void{
//     unsafe{
//         set_metadata(
//             pop,
//             ino as c_int,
//             size as size_t,
//             blocks as c_int,
//             atime as c_long,
//             mtime as c_long,
//             ctime as c_long,
//             crtime as c_long,
//             kind,
//             perm as c_int,
//             nlink as c_int,
//             uid as c_int,
//             gid as c_int,
//             rdev as c_int,
//             flags as c_int
//         )
//     }
// }

pub fn my_free_metadata(pop: *mut PMEMobjpool, id: u32)->c_void{
    unsafe{
        free_metadata(pop, id as c_int)
    }
}

pub fn my_init_md(path: *const u8)->*mut PMEMobjpool{
    unsafe{
        init_md(path as *const c_char)
    }
}

