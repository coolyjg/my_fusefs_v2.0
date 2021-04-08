use std::ffi::OsStr;
use std::sync::Arc;
use std::sync::Weak;
use libc::{ENOENT, ENOTEMPTY};
use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory, ReplyEmpty, ReplyWrite};
use std::collections::BTreeMap;
use std::result;
use std::sync::RwLock;
use std::sync::RwLockWriteGuard;
use std::any::Any;
use time::*;
use std::str::from_utf8;


use crate::ffi_wrapper::{
    my_read_at_content, my_write_at_content, my_fin, my_create_content, my_free,
    my_create_metadata, my_free_metadata, 
    my_set_metadata_size, my_set_atime, my_set_mtime,
    my_set_ctime, my_set_crtime, my_set_kind, my_set_perm, 
    my_get_size, my_get_atime, my_get_mtime,
    my_get_ctime, my_get_crtime, my_get_kind, my_get_perm, my_get_nlink,
    Filetype
};

use crate::ffi_wrapper::MY_POP_DATA;
use crate::ffi_wrapper::MY_POP_MD;

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

pub struct MyFuseFs{
    pub inodes: BTreeMap<usize, Arc<LockedINode>>,
    pub fs: Arc<RamFs>,
}

pub struct RamFs{
    root: Arc<LockedINode>,
}

pub struct FuseInode{
    parent: Weak<LockedINode>,
    this: Weak<LockedINode>,
    children: BTreeMap<String, Arc<LockedINode>>,
    content: Vec<u8>,
    extra: FileAttr,
    fs: Weak<RamFs>,
}


pub struct LockedINode(RwLock<FuseInode>);

impl RamFs{
    pub fn new() -> Arc<Self>{
        let t = time::now().to_timespec();
        let root = Arc::new(LockedINode(RwLock::new(FuseInode{
            parent: Weak::default(),
            this: Weak::default(),
            children: BTreeMap::new(),
            content: Vec::new(),
            extra: FileAttr{
                ino: new_inode_id() as u64,
                size: 0,
                blocks: 0,
                atime: t,
                mtime: t,
                ctime: t,
                crtime: t,
                kind: FileType::Directory,
                perm: 0o777,
                nlink: 1,
                uid: 0,
                gid: 0,
                rdev: 0,
                flags: 0,
            },
            fs: Weak::default(),
        })));
        let fs = Arc::new(RamFs {root} );
        let t = t.sec as u64;
        let type_md = Filetype::Directory;
        unsafe{
            my_create_metadata(
                MY_POP_MD.unwrap(), fs.root.0.read().unwrap().extra.ino as u32, 0,
                0, t, t, t, t, type_md, 0o777, 1, 0, 0, 0, 0
            );
        }
        let mut root = fs.root.0.write().unwrap();
        root.parent = Arc::downgrade(&fs.root);
        root.this = Arc::downgrade(&fs.root);
        root.fs = Arc::downgrade(&fs);
        root.extra.ino = 
            Arc::into_raw(root.this.upgrade().unwrap()) as *const FuseInode as u64;
        drop(root);
        fs
    }
}

impl MyFuseFs{
    pub fn new(fs: Arc<RamFs>) ->Self{
        let mut inodes = BTreeMap::new();
        inodes.insert(1, Arc::clone(&fs.root));
        MyFuseFs{
            fs,
            inodes,
        }
    }
    fn get_inode(&self, ino:u64)->Result<&Arc<LockedINode>>{
        self.inodes.get(&(ino as usize)).ok_or(FsError::EntryNotFound)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum FsError {
    NotFile,       // E_ISDIR
    IsDir,         // E_ISDIR, used only in link
    NotDir,        // E_NOTDIR
    EntryNotFound, // E_NOENT
    EntryExist,    // E_EXIST
    NotSameFs,     // E_XDEV
    DirNotEmpty,   // E_NOTEMPTY
}

fn lock_multiple<'a>(locks: &[&'a RwLock<FuseInode>]) -> Vec<RwLockWriteGuard<'a, FuseInode>> {
    let mut order: Vec<usize> = (0..locks.len()).collect();
    let mut guards = BTreeMap::new();
    order.sort_by_key(|&i| locks[i].read().unwrap().extra.ino);
    for i in order {
        guards.insert(i, locks[i].write());
    }
    let mut ret :Vec<RwLockWriteGuard<'a, FuseInode>> = Vec::new();
    for i in 0..locks.len() {
        ret.push(guards.remove(&i).unwrap().expect("panic occurs in function lock_multiple!"));
    }
    ret
}

fn new_inode_id() -> usize {
    use std::sync::atomic::*;
    static ID: AtomicUsize = AtomicUsize::new(1);
    ID.fetch_add(1, Ordering::SeqCst)
}

pub type Result<T> = result::Result<T, FsError>;
impl LockedINode{
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.as_any_ref().downcast_ref::<T>()
    }
    pub fn as_any_ref(&self) -> &dyn Any {
        self
    }
    pub fn read_at(&self, offset: usize, buf: &mut [u8])->Result<usize>{
        let file = self.0.read().unwrap();
        if file.extra.kind == FileType::Directory{
            return Err(FsError::IsDir);
        }
        // let start = file.content.len().min(offset);
        // let end = file.content.len().min(offset+buf.len());
        // let src = &file.content[start..end];
        // buf[0..src.len()].copy_from_slice(src);
        // Ok(src.len())
        let ino = file.extra.ino as u32;
        // println!("searching for id(rust): {}", ino);
        let len = unsafe{
            my_read_at_content(MY_POP_DATA.unwrap(), ino, offset, buf.as_ptr())
        };
        Ok(len as usize)
    }
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize> {
        let file = self.0.write().unwrap();
        if file.extra.kind == FileType::Directory {
            return Err(FsError::IsDir);
        }
        // {let content = &mut file.content;
        // if offset + buf.len() > content.len(){
        //     content.resize(offset as usize + buf.len() as usize, 0);
        //     file.extra.size = offset as u64+buf.len() as u64;
        // }}
        // let content = &mut file.content;
        // let target = &mut content[offset..offset + buf.len()];
        // target.copy_from_slice(buf);
        // Ok(buf.len())
        // let size = buf.len();
        let ino = file.extra.ino as u32;
        // let buf = buf.to_string();
        let s = from_utf8(buf).expect("invalid utf-8 sequence");
        // println!("s= {}",s);
        let len = unsafe{
            my_write_at_content(MY_POP_DATA.unwrap(), ino, offset, s.as_ptr(), s.len())
        };
        unsafe{
            my_set_metadata_size(MY_POP_MD.unwrap(), ino, len as usize);
        }
        // println!("new size = {}", len);
        Ok(len as usize)
    }
    pub fn resize(&self, len: usize)->Result<()>{
        let mut file = self.0.write().unwrap();
        if file.extra.kind == FileType::RegularFile{
            file.content.resize(len, 0);
            Ok(())
        }
        else {
            Err(FsError::NotFile)
        }
    }
    pub fn create(&self, name: &str, type_:FileType, _mode:u32)->Result<Arc<LockedINode>>{
        let mut file = self.0.write().unwrap();
        if file.extra.kind ==FileType::Directory{
            if file.children.contains_key(name) {
                return Err(FsError::EntryExist);
            }
            let t = time::now().to_timespec();
            let temp_file = Arc::new(LockedINode(RwLock::new(FuseInode{
                parent: Weak::clone(&file.this),
                this: Weak::default(),
                children: BTreeMap::new(),
                content: Vec::new(),
                extra: FileAttr{
                    ino: new_inode_id() as u64,
                    size: 0,
                    blocks: 0,
                    atime: t,
                    mtime: t,
                    ctime: t,
                    crtime: t,
                    kind: type_,
                    perm: 0o777,
                    nlink: 1,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    flags: 0,
                },
                fs: Weak::clone(&file.fs),
            })));
            let t = t.sec as u64;
            let type_md = match type_{
                FileType::RegularFile => Filetype::Regular,
                _ => Filetype::Directory,
            };
            unsafe{
                my_create_metadata(
                    MY_POP_MD.unwrap(), temp_file.0.read().unwrap().extra.ino as u32, 0,
                    0, t, t, t, t, type_md, 0o777, 1, 0, 0, 0, 0
                );
            }
            temp_file.0.write().unwrap().this = Arc::downgrade(&temp_file);
            file.children.insert(String::from(name), Arc::clone(&temp_file));
            Ok(temp_file)
        }
        else{
            Err(FsError::NotDir)
        }
    }
    pub fn unlink(&self, name: &str) -> Result<()>{
        let mut file = self.0.write().unwrap();
        if file.extra.kind != FileType::Directory{
            return Err(FsError::NotDir);
        }
        let other = file.children.get(name).ok_or(FsError::EntryNotFound)?;
        if other.0.read().unwrap().children.len()>0{
            return Err(FsError::DirNotEmpty);
        }
        let ino = other.0.read().unwrap().extra.clone().ino;
        unsafe{
            my_free(MY_POP_DATA.unwrap(), ino as u32);
            my_free_metadata(MY_POP_MD.unwrap(), ino as u32);
        }
        other.0.write().unwrap().extra.nlink-= 1;
        file.children.remove(name);
        Ok(())
    }
    pub fn link(&self, name: &str, other: &Arc<LockedINode>) -> Result<()> {
        let other = other
            .downcast_ref::<LockedINode>()
            .ok_or(FsError::NotSameFs)?;
        let mut locks = lock_multiple(&[&self.0, &other.0]).into_iter();
        let mut file = locks.next().unwrap();
        let mut other_l = locks.next().unwrap();
        if file.extra.kind != FileType::Directory {
            return Err(FsError::NotDir);
        }
        if other_l.extra.kind == FileType::Directory {
            return Err(FsError::IsDir);
        }
        if file.children.contains_key(name) {
            return Err(FsError::EntryExist);
        }
        file.children
            .insert(String::from(name), other_l.this.upgrade().unwrap());
        other_l.extra.nlink += 1;
        Ok(())
    }
    fn move_(&self, old_name: &str, target: &Arc<LockedINode>, new_name: &str) -> Result<()> {
        let elem = self.find(old_name)?;
        target.link(new_name, &elem)?;
        if let Err(err) = self.unlink(old_name) {
            target.unlink(new_name)?;
            return Err(err);
        }
        Ok(())
    }
    fn find(&self, name: &str) -> Result<Arc<LockedINode>> {
        let file = self.0.read().unwrap();
        if file.extra.kind != FileType::Directory {
            return Err(FsError::NotDir);
        }
        match name {
            "." => Ok(file.this.upgrade().ok_or(FsError::EntryNotFound)?),
            ".." => Ok(file.parent.upgrade().ok_or(FsError::EntryNotFound)?),
            name => {
                let s = file.children.get(name).ok_or(FsError::EntryNotFound)?;
                Ok(Arc::clone(s) as Arc<LockedINode>)
            }
        }
    }
    fn get_entry(&self, id: usize) -> Result<String> {
        let file = self.0.read().unwrap();
        if file.extra.kind != FileType::Directory {
            return Err(FsError::NotDir);
        }
        match id {
            0 => Ok(String::from(".")),
            1 => Ok(String::from("..")),
            i => {
                if let Some(s) = file.children.keys().nth(i - 2) {
                    Ok(s.to_string())
                } else {
                    Err(FsError::EntryNotFound)
                }
            }
        }
    }
    fn get_md(&self, id: usize) -> Result<FileAttr>{
        let file = self.0.read().unwrap();
        let mut md = file.extra.clone();
        unsafe{
            md.size = my_get_size(MY_POP_MD.unwrap(), id as u32) as u64;
            md.atime.sec = my_get_atime(MY_POP_MD.unwrap(), id as u32);
            md.mtime.sec = my_get_mtime(MY_POP_MD.unwrap(), id as u32);
            md.ctime.sec = my_get_ctime(MY_POP_MD.unwrap(), id as u32);
            md.crtime.sec = my_get_crtime(MY_POP_MD.unwrap(), id as u32);
            md.kind = match my_get_kind(MY_POP_MD.unwrap(), id as u32){
                Filetype::Regular => FileType::RegularFile,
                _ => FileType::Directory,
            };
            md.perm = my_get_perm(MY_POP_MD.unwrap(), id as u32) as u16;
            md.nlink = my_get_nlink(MY_POP_MD.unwrap(), id as u32) as u32;
        }
        Ok(md)
    }
}

impl Filesystem for MyFuseFs{
    fn destroy(&mut self, _req: &Request) {
        self.inodes.clear();
        unsafe{
            my_fin(MY_POP_DATA.unwrap());
            my_fin(MY_POP_MD.unwrap());
        };
    }
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry){
        let inode = match self.get_inode(parent){
            Ok(val) => val,
            Err(_err) =>{
                reply.error(ENOENT);
                return ;
            }
        };
        let target = inode.0.read().unwrap();
        let child =match target.children.get(name.to_str().unwrap()){
            Some(val) =>val,
            None =>{
                reply.error(ENOENT);
                return ;
            }
        };
        let attr = child.0.read().unwrap().extra.clone();
        reply.entry(&TTL, &attr, 0);
    }
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr){
        let inode = self.get_inode(ino).unwrap();
        let attr = inode.get_md(ino as usize).unwrap();
        reply.attr(&TTL, &attr);
    }
    fn setattr(
        &mut self, 
        _req: &Request, 
        ino: u64, 
        mode: Option<u32>, 
        uid: Option<u32>, 
        gid: Option<u32>, 
        size: Option<u64>, 
        atime: Option<Timespec>, 
        mtime: Option<Timespec>, 
        _fh: Option<u64>, 
        crtime: Option<Timespec>, 
        chgtime: Option<Timespec>, 
        _bkuptime: Option<Timespec>, 
        flags: Option<u32>, 
        reply: ReplyAttr
    ){
        let inode = self.get_inode(ino).unwrap();
        let tp = match inode.0.read().unwrap().extra.kind{
            FileType::RegularFile => Filetype::Regular,
            _ => Filetype::Directory,
        }; 
        unsafe{
            my_set_kind(MY_POP_MD.unwrap(), ino as u32, tp);
        }
        if let Some(size) = size{
            inode.resize(size as usize).expect("resize error");
            unsafe{
                my_set_metadata_size(MY_POP_MD.unwrap(), ino as u32, size as usize);
            }
        }
        let mut info = inode.0.write().unwrap();
        if let Some(mode) = mode{
            info.extra.perm = mode as u16;
            unsafe{
                my_set_perm(MY_POP_MD.unwrap(), ino as u32, mode);
            }
        }
        if let Some(uid) = uid{
            info.extra.uid = uid;
        }
        if let Some(gid) = gid{
            info.extra.gid = gid;
        }
        if let Some(atime) = atime{
            info.extra.atime = atime;
            unsafe{
                my_set_atime(MY_POP_MD.unwrap(), ino as u32, atime.sec as u64);
            }
        }
        if let Some(mtime) = mtime{
            info.extra.mtime = mtime;
            unsafe{
                my_set_mtime(MY_POP_MD.unwrap(), ino as u32, mtime.sec as u64);
            }
        }
        if let Some(crtime) = crtime{
            info.extra.crtime = crtime;
            unsafe{
                my_set_crtime(MY_POP_MD.unwrap(), ino as u32, crtime.sec as u64);
            }
        }
        if let Some(ctime) = chgtime{
            info.extra.ctime = ctime;
            unsafe{
                my_set_ctime(MY_POP_MD.unwrap(), ino as u32, ctime.sec as u64);
            }
        }
        if let Some(flags) = flags{
            info.extra.flags = flags;
        }
        let attr = info.extra.clone();
        // let attr = at.clone();
        // unsafe{
        //     my_set_metadata(
        //         MY_POP_MD.unwrap(), ino as u32, size.unwrap() as usize, 0, atime.unwrap().sec as u64, 
        //         mtime.unwrap().sec as u64, chgtime.unwrap().sec as u64, crtime.unwrap().sec as u64,
        //         tp, mode.unwrap(), nk, uid.unwrap(), gid.unwrap(), 0, flags.unwrap()
        //     );
        // }
        reply.attr(&TTL, &attr);
    }
    fn mknod(
        &mut self, 
        _req: &Request, 
        parent: u64, 
        name: &OsStr, 
        mode: u32, 
        _rdev: u32, 
        reply: ReplyEntry
    ){
        println!("response by my_fusefs mknod");
        let name = name.to_str().unwrap();
        let inode = self.get_inode(parent).unwrap();
        let target = inode.create(name, FileType::RegularFile, mode).unwrap();
        let attr = target.0.read().unwrap().extra.clone();
        self.inodes.insert(attr.ino as usize, target);
        let buf = "".to_string();
        unsafe{
            my_create_content(MY_POP_DATA.unwrap(), 0, buf.as_ptr(), attr.ino as u32);
            // my_create_metadata(
            //     MY_POP_MD.unwrap(), attr.ino, attr.size, attr.blocks,
            //     attr.atime, attr.mtime, attr.ctime, attr.crtime,
            //     attr.kind, attr.perm, attr.nlink, attr.uid,
            //     attr.gid, attr.rdev, attr.flags
            // );
        }
        reply.entry(&TTL, &attr, 0);
    }
    fn mkdir(
        &mut self, 
        _req: &Request, 
        parent: u64, 
        name: &OsStr, 
        mode: u32, 
        reply: ReplyEntry
    ){
        let name = name.to_str().unwrap();
        let inode = self.get_inode(parent).unwrap();
        let target = inode.create(name, FileType::Directory, mode).unwrap();
        let attr = target.0.read().unwrap().extra.clone();
        self.inodes.insert(attr.ino as usize, target);
        reply.entry(&TTL, &attr, 0);
    }
    fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let name = name.to_str().unwrap();
        let parent = self.get_inode(parent).unwrap();
        match parent.unlink(name) {
            Ok(()) => reply.ok(),
            Err(_err) => {
                reply.error(ENOTEMPTY);
                return ;
            }
        }
    }
    fn rmdir(&mut self, req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty){
        self.unlink(req, parent, name, reply);
    }
    fn rename(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        reply: ReplyEmpty,
    ){
        let name = name.to_str().unwrap();
        let newname = newname.to_str().unwrap();
        let parent = self.get_inode(parent).unwrap();
        let newparent  = self.get_inode(newparent).unwrap();
        parent.move_(name, newparent, newname).expect("move error");
        reply.ok();
    }
    fn link(&mut self, 
        _req: &Request<'_>,
        ino: u64, 
        newparent: u64, 
        newname: &OsStr, 
        reply: ReplyEntry
    ) {
        let newname = newname.to_str().unwrap();
        let inode = self.get_inode(ino).unwrap();
        let newparent = self.get_inode(newparent).unwrap();
        newparent.link(newname, inode).expect("link error");
        let attr = inode.0.read().unwrap().extra.clone();
        reply.entry(&TTL, &attr, 0);
    }
    fn read(
        &mut self, 
        _req: &Request<'_>, 
        ino: u64,
        _fh: u64, 
        offset: i64, 
        size: u32, 
        reply: ReplyData
    ) {
        let inode = self.get_inode(ino).unwrap();
        let mut data = Vec::<u8>::new();
        data.resize(size as usize, 0);
        inode.read_at(offset as usize, data.as_mut_slice()).expect("read_at error");
        reply.data(data.as_slice());
    }
    fn write(
        &mut self,
        _req: &Request, 
        ino: u64, 
        _fh: u64, 
        offset: i64, 
        data: &[u8], 
        _flags: u32, 
        reply: ReplyWrite
    ){
        let inode = self.get_inode(ino).unwrap();
        let len = inode.write_at(offset as usize, data).unwrap();
        let mut info = inode.0.write().unwrap();
        info.extra.size =len as u64;
        println!("top level new size = {}", len);
        reply.written(len as u32);
    }
    fn readdir(
        &mut self,
        _req: &Request, 
        ino: u64, 
        _fh: u64, 
        offset: i64, 
        mut reply: ReplyDirectory
    ){
        let inode = self.get_inode(ino).unwrap();
        for i in offset as usize..{
            let name = match inode.get_entry(i){
                Ok(name) => name,
                Err(FsError::EntryNotFound) => break,
                e @ _ =>e.unwrap(),
            };
            let inode = inode.find(name.as_str()).unwrap();
            let info = inode.0.read().unwrap().extra.clone();
            let kind = info.kind;
            let full = reply.add(info.ino as u64, i as i64 + 1, kind, name);
            if full{
                break;
            }
        }
        reply.ok();
    }
}


