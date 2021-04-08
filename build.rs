extern crate cc;

fn main(){
    cc::Build::new()
        .file("src/cnt_store.c")
        .compile("libcnt_store.a")
}
