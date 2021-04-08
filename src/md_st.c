#include <math.h>
#include <stdlib.h>
#include <stdio.h>
#include <assert.h>
#include <sys/stat.h>
#include <string.h>
#include <time.h>
#include <libpmemobj.h>

POBJ_LAYOUT_BEGIN(metadata_pool); 
    POBJ_LAYOUT_TOID(metadata_pool, struct metadata);
POBJ_LAYOUT_END(metadata_pool);

enum Filetype{
    Directory,
    Regular
};

struct metadata{
    int ino;
    size_t size;
    int blocks;
    time_t atime;
    time_t mtime;
    time_t ctime;
    time_t crtime;
    enum Filetype kind;
    int perm;
    int nlink;
    int uid;
    int gid;
    int rdev;
    int flags;
};

TOID(struct metadata) find_metadata(PMEMobjpool* pop, const int id){
    TOID(struct metadata) ret;
    POBJ_FOREACH_TYPE(pop, ret){
        if(D_RO(ret)->ino == id){
            return ret;
        }
    }
    return TOID_NULL(struct metadata);
}

struct metadata* get_metadata(PMEMobjpool* pop, const int id){
    TOID(struct metadata) ret = find_metadata(pop, id);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return NULL;
    }
    return D_RW(ret);
}




void create_metadata(
    PMEMobjpool *pop,
    int ino,
    size_t size,
    int blocks,
    time_t atime,
    time_t mtime,
    time_t ctime,
    time_t crtime,
    enum Filetype kind,
    int perm,
    int nlink,
    int uid,
    int gid,
    int rdev,
    int flags
){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(!TOID_IS_NULL(ret)){
        POBJ_FREE(&ret);
    }
    POBJ_ZNEW(pop, &ret, struct metadata);
    struct metadata* ret_dir = D_RW(ret);
    ret_dir ->ino = ino;
    ret_dir ->size = size;
    ret_dir ->blocks = blocks;
    ret_dir ->atime = atime;
    ret_dir ->mtime = mtime;
    ret_dir ->ctime = ctime;
    ret_dir ->crtime = crtime;
    ret_dir ->kind = kind;
    ret_dir ->perm = perm;
    ret_dir ->nlink = nlink;
    ret_dir ->uid = uid;
    ret_dir ->gid = gid;
    ret_dir ->rdev = rdev;
    ret_dir ->flags = flags;
    pmemobj_persist(pop, ret_dir, sizeof(*ret_dir));
}

void set_metadata_size(
    PMEMobjpool* pop,
    int ino,
    size_t size
){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return;
    }
    struct metadata* ret_dir = D_RW(ret);
    ret_dir->size = size;
    pmemobj_persist(pop, ret_dir, sizeof(*ret_dir));
}



void set_metadata(
    PMEMobjpool* pop,
    int ino,
    size_t size,
    int blocks,
    time_t atime,
    time_t mtime,
    time_t ctime,
    time_t crtime,
    enum Filetype kind,
    int perm,
    int nlink,
    int uid,
    int gid,
    int rdev,
    int flags
){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found ,now create!\n");
        create_metadata(pop, ino, size, blocks, atime, mtime, ctime, crtime, kind, perm, nlink, uid, gid, rdev, flags);
    }
    ret = find_metadata(pop, ino);
    struct metadata* ret_dir = D_RW(ret);
    ret_dir ->ino = ino;
    ret_dir ->size = size;
    ret_dir ->blocks = blocks;
    ret_dir ->atime = atime;
    ret_dir ->mtime = mtime;
    ret_dir ->ctime = ctime;
    ret_dir ->crtime = crtime;
    ret_dir ->kind = kind;
    ret_dir ->perm = perm;
    ret_dir ->nlink = nlink;
    ret_dir ->uid = uid;
    ret_dir ->gid = gid;
    ret_dir ->rdev = rdev;
    ret_dir ->flags = flags;
    pmemobj_persist(pop, ret_dir, sizeof(*ret_dir));
}

void free_metadata(PMEMobjpool *pop, const int id){
    TOID(struct metadata) ret = find_metadata(pop, id);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return ;
    }
    POBJ_FREE(&ret);
}

PMEMobjpool* init_md(const char* path){
    POBJ_LAYOUT_BEGIN(metadata_pool);
        POBJ_LAYOUT_TOID(metadata_pool, struct metadata);
    POBJ_LAYOUT_END(metadata_pool);
    static PMEMobjpool* pop = NULL;
    if((pop = pmemobj_create(path, POBJ_LAYOUT_NAME(metadata_pool), PMEMOBJ_MIN_POOL, 0666))==NULL){
        if((pop = pmemobj_open(path, POBJ_LAYOUT_NAME(metadata_pool)))==NULL){
            printf("fail to open metadata_pool\n");
            return NULL;
        }
    }
    return pop;
}






