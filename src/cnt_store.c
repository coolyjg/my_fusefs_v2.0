#include <math.h>
#include <stdlib.h>
#include <stdio.h>
#include <assert.h>
#include <sys/stat.h>
#include <string.h>
#include <time.h>
#include <libpmemobj.h>

#define MAX_CONTENT_LEN 50

POBJ_LAYOUT_BEGIN(content_pool); 
    POBJ_LAYOUT_TOID(content_pool, struct content);
POBJ_LAYOUT_END(content_pool);

// static PMEMobjpool *pop;

struct content{
    int ino; //inode id
    char cont[MAX_CONTENT_LEN];
    size_t size; //length of the content
};

//find the content struct by the inode number
TOID(struct content) find_content(PMEMobjpool* pop, const int id){
    TOID(struct content) ret;
    // printf("searching for id: %d\n", id);
    POBJ_FOREACH_TYPE(pop, ret){
        if(D_RO(ret)->ino == id){
            return ret;
        }
    }
    return TOID_NULL(struct content);
}

void create_content(PMEMobjpool* pop, size_t size, char* buf, const int id){
    TOID(struct content) cnt = find_content(pop, id);
    if(!TOID_IS_NULL(cnt)){
        POBJ_FREE(&cnt);
    }
    POBJ_ZNEW(pop, &cnt, struct content);
    struct content* cnt_dir = D_RW(cnt);
    cnt_dir->ino = id;
    cnt_dir->size = size;
    pmemobj_memcpy_persist(pop, cnt_dir->cont, buf, size);
    pmemobj_persist(pop, cnt_dir, sizeof(*cnt_dir));
    // return 0;
}

int write_at_content(PMEMobjpool* pop, const int id, const int offset, char* buf, const int size){
    TOID(struct content) cnt = find_content(pop, id);
    if(TOID_IS_NULL(cnt)){
        printf("not found, now create!\n");
        // return create_content(pop, size, buf, id);
        create_content(pop, size, buf, id);
    }
    cnt = find_content(pop, id);
    // printf("got content: %s\n", buf);
    int new_size = offset+size > (int)D_RO(cnt)->size? offset+size: (int)D_RO(cnt)->size;
    // printf("new size(C) = %d\n", new_size);
    D_RW(cnt)->size = new_size;
    pmemobj_persist(pop, &D_RW(cnt)->size, sizeof(int));
    pmemobj_memcpy_persist(pop, D_RW(cnt)->cont+offset, buf, size);
    pmemobj_persist(pop, D_RW(cnt), sizeof(*D_RW(cnt)));
    return new_size;
}

int write_content(PMEMobjpool * pop, const int id, char* buf, const int size){
    return write_at_content(pop, id, 0, buf, size);
}

int read_at_content(PMEMobjpool *pop, const int id, const int offset, char* buf){
    TOID(struct content) cnt = find_content(pop, id);
    if(TOID_IS_NULL(cnt)){
        printf("not found\n");
        return -2;
    }
    // else {
    //     printf("find content\n");
    // }
    if(offset> (int)D_RW(cnt)->size){
        return -3;
    }
    sprintf(buf, "%s", D_RW(cnt)->cont+offset);
    // memcpy(buf, D_RW(cnt)->cont+offset, D_RW(cnt)->size-offset);
    return 0;
}

int read_content(PMEMobjpool* pop, const int id, char* buf){
    return read_at_content(pop, id, 0, buf);
}


void print_content(PMEMobjpool* pop, const int id){
    TOID(struct content) ret = find_content(pop, id);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return;
    }
    printf("ino: %d\ncontent: %s\n", D_RW(ret)->ino, D_RW(ret)->cont);
}

void print_content_all(PMEMobjpool* pop){
    TOID(struct content) cnt;
    POBJ_FOREACH_TYPE(pop, cnt){
        struct content* cnt_dir = D_RW(cnt);
        printf("ino: %d\ncontent: %s\n", cnt_dir->ino, cnt_dir->cont);
    }
}

void my_free_fn(PMEMobjpool* pop, const int id){
    TOID(struct content) ret = find_content(pop, id);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return ;
    }
    POBJ_FREE(&ret);
}

void my_free_all_fn(PMEMobjpool* pop){
    TOID(struct content) cnt;
    POBJ_FOREACH_TYPE(pop, cnt){
        my_free_fn(pop, D_RW(cnt)->ino);
    }
}

PMEMobjpool* init(const char *path){
    POBJ_LAYOUT_BEGIN(content_pool); 
        POBJ_LAYOUT_TOID(content_pool, struct content);
    POBJ_LAYOUT_END(content_pool);
    static PMEMobjpool* pop = NULL;
    if((pop = pmemobj_create(path, POBJ_LAYOUT_NAME(content_pool), PMEMOBJ_MIN_POOL,0666))==NULL){
        if((pop = pmemobj_open(path, POBJ_LAYOUT_NAME(content_pool)))==NULL){
            printf("fail to open\n");
            return NULL;
        }
    }
    return pop;    
}

void fin(PMEMobjpool* pop){
    pmemobj_close(pop);
}

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

void set_metadata_size(PMEMobjpool* pop, int ino, size_t size){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return;
    }
    struct metadata* ret_dir = D_RW(ret);
    ret_dir->size = size;
    pmemobj_persist(pop, ret_dir, sizeof(*ret_dir));
}

size_t get_metadata_size(PMEMobjpool* pop, int ino){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found size\n");
        return -1;
    }
    struct metadata* ret_dir = D_RW(ret);
    return ret_dir->size;
}

void set_blocks(PMEMobjpool* pop, int ino, int blocks){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return;
    }
    struct metadata* ret_dir = D_RW(ret);
    ret_dir->blocks = blocks;
    pmemobj_persist(pop, ret_dir, sizeof(*ret_dir));
}

int get_metadata_blocks(PMEMobjpool* pop, int ino){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found blocks\n");
        return -1;
    }
    struct metadata* ret_dir = D_RW(ret);
    return ret_dir->blocks;
}

void set_atime(PMEMobjpool* pop, int ino, time_t atime){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return;
    }
    struct metadata* ret_dir = D_RW(ret);
    ret_dir->atime = atime;
    pmemobj_persist(pop, ret_dir, sizeof(*ret_dir));
}

time_t get_metadata_atime(PMEMobjpool* pop, int ino){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found atime\n");
        return -1;
    }
    struct metadata* ret_dir = D_RW(ret);
    return ret_dir->atime;
}

void set_mtime(PMEMobjpool* pop, int ino, time_t mtime){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return;
    }
    struct metadata* ret_dir = D_RW(ret);
    ret_dir->mtime = mtime;
    pmemobj_persist(pop, ret_dir, sizeof(*ret_dir));
}

time_t get_metadata_mtime(PMEMobjpool* pop, int ino){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found mtime\n");
        return -1;
    }
    struct metadata* ret_dir = D_RW(ret);
    return ret_dir->mtime;
}

void set_ctime(PMEMobjpool* pop, int ino, time_t ctime){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return;
    }
    struct metadata* ret_dir = D_RW(ret);
    ret_dir->ctime = ctime;
    pmemobj_persist(pop, ret_dir, sizeof(*ret_dir));
}

time_t get_metadata_ctime(PMEMobjpool* pop, int ino){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found ctime\n");
        return -1;
    }
    struct metadata* ret_dir = D_RW(ret);
    return ret_dir->ctime;
}

void set_crtime(PMEMobjpool* pop, int ino, time_t crtime){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return;
    }
    struct metadata* ret_dir = D_RW(ret);
    ret_dir->crtime = crtime;
    pmemobj_persist(pop, ret_dir, sizeof(*ret_dir));
}

time_t get_metadata_crtime(PMEMobjpool* pop, int ino){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found crtime\n");
        return -1;
    }
    struct metadata* ret_dir = D_RW(ret);
    return ret_dir->crtime;
}

void set_kind(PMEMobjpool* pop, int ino, enum Filetype kind){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return;
    }
    struct metadata* ret_dir = D_RW(ret);
    ret_dir->kind = kind;
    pmemobj_persist(pop, ret_dir, sizeof(*ret_dir));
}

enum Filetype get_metadata_kind(PMEMobjpool* pop, int ino){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found kind\n");
        return -1;
    }
    struct metadata* ret_dir = D_RW(ret);
    return ret_dir->kind;
}

void set_perm(PMEMobjpool* pop, int ino, int perm){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return;
    }
    struct metadata* ret_dir = D_RW(ret);
    ret_dir->perm = perm;
    pmemobj_persist(pop, ret_dir, sizeof(*ret_dir));
}

int get_metadata_perm(PMEMobjpool* pop, int ino){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found perm\n");
        return -1;
    }
    struct metadata* ret_dir = D_RW(ret);
    return ret_dir->perm;
}

void set_nlink(PMEMobjpool* pop, int ino, int nlink){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return;
    }
    struct metadata* ret_dir = D_RW(ret);
    ret_dir->nlink = nlink;
    pmemobj_persist(pop, ret_dir, sizeof(*ret_dir));
}

int get_metadata_nlink(PMEMobjpool* pop, int ino){
    TOID(struct metadata) ret = find_metadata(pop, ino);
    if(TOID_IS_NULL(ret)){
        printf("not found nlink\n");
        return -1;
    }
    struct metadata* ret_dir = D_RW(ret);
    return ret_dir->nlink;
}

struct metadata* get_metadata(PMEMobjpool* pop, const int id){
    TOID(struct metadata) ret = find_metadata(pop, id);
    if(TOID_IS_NULL(ret)){
        printf("not found\n");
        return NULL;
    }
    return D_RW(ret);
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

