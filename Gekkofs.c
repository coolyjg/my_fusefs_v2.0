#include<stdio.h>
#include<stdlib.h>
#include<string.h>
#include<sys/types.h>
#include<fcntl.h>
#include<sys/stat.h>
// #include<direct.h>
#include<dirent.h>

#define __mode_t __MODE_T_TYPE
#define __MODE_T_TYPE __U32_TYPE
#define __U32_TYPE unsigned int

#ifndef __mode_t_defined
typedef __mode_t mode_t;
# define __mode_t_defined
#endif

static const char raw_path[100] = "/mnt/my_fuse_test/";

void modify_path(char* rpath, const char* path){
    strcpy(rpath, raw_path);
    strcat(rpath, path);
}

int f_write(const char* path, int size, const char* buf){
    char temp_path[100];
    modify_path(temp_path, path);
    puts(temp_path);
    FILE* fp = fopen(temp_path, "w");
    int ret = fputs(buf, fp);
    fclose(fp);
    return ret;
}

int f_read(const char* path, int size, char* buf){
    char temp_path[100];
    modify_path(temp_path, path);
    puts(temp_path);
    FILE* fp = fopen(temp_path, "r");
    fgets(buf, size, fp);
    fclose(fp);
    return size;
}

int f_open2(const char* path, int flags){
    char temp_path[100];
    modify_path(temp_path, path);
    return open(temp_path, flags);
}

int f_open3(const char* path, int flags, mode_t mode){
    char temp_path[100];
    modify_path(temp_path, path);
    return open(temp_path, flags, mode);
}

int f_close(FILE* fp){
    // char temp_path[100];
    // modify_path(temp_path, path);
    return fclose(fp);
}

int f_unlink(const char* path){
    char temp_path[100];
    modify_path(temp_path, path);
    return remove(temp_path);
}

int f_remove(const char* path){
    char temp_path[100];
    modify_path(temp_path, path);
    return remove(temp_path);
}

int f_mkdir(const char* path, mode_t mode){
    char temp_path[100];
    modify_path(temp_path, path);
    return mkdir(path, mode);
}

int f_rmdir(const char* path){
    char temp_path[100];;
    modify_path(temp_path, path);
    return remove(temp_path);
}

struct dirent* f_readdir(const char* path){
    char temp_path[100];
    modify_path(temp_path, path);
    return readdir((DIR*)temp_path);
}

void main(){
    char buf[20] = "hellowolrd";
    int size = strlen(buf);
    printf("size = %d\n", size);
    int size2 = f_write("test_file", size, buf);
    printf("size2 = %d\n", size2);
    char buf2[20];
    f_read("test_file", size, buf2);
    puts(buf2);
    // f_remove("test_file");
}


