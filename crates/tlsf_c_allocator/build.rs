use std::env;
use std::process::Command;
use std::path::Path;


fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    //Command::new("cc")
    Command::new("riscv64-linux-gnu-gcc")
        .args(&["src/tlsf.c", "-O3","-c", "-fPIC", "-o"])
        .arg(&format!("{}/tlsf.o", out_dir))
        .status().unwrap();
 
    //Command::new("ar")
    Command::new("riscv64-linux-gnu-ar")
        .args(&["crus", "libtlsf.a", "tlsf.o"])
        .current_dir(&Path::new(&out_dir))
        .status().unwrap();
 
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=tlsf");    
}

//riscv64-linux-gnu-gcc -DAX_CONFIG_DEFAULT -DAX_CONFIG_ALLOC -DAX_CONFIG_PAGING -static -no-pie -fno-builtin -ffreestanding -nostdinc -Wall -Iulib/c_libax/include -Iulib/c_libax/../libax -O3 -march=rv64gc -mabi=lp64d -mcmodel=medany -c -o apps/c/memtest/memtest.o apps/c/memtest/memtest.c
//riscv64-linux-gnu-ar rc ulib/c_libax/build_riscv64/libc.a ulib/c_libax/build_riscv64/stat.o ulib/c_libax/build_riscv64/time.o ulib/c_libax/build_riscv64/stdlib.o ulib/c_libax/build_riscv64/fcntl.o ulib/c_libax/build_riscv64/string.o ulib/c_libax/build_riscv64/stdio.o ulib/c_libax/build_riscv64/mmap.o ulib/c_libax/build_riscv64/unistd.o ulib/c_libax/build_riscv64/assert.o