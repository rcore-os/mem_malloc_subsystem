use std::env;
use std::process::Command;
use std::path::Path;


fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
 
    Command::new("cc").args(&["tests/test.c", "-O3","-c", "-fPIC", "-o"])
        .arg(&format!("{}/test.o", out_dir))
        .status().unwrap();
 
    Command::new("ar").args(&["crus", "libtest.a", "test.o"])
        .current_dir(&Path::new(&out_dir))
        .status().unwrap();
 
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=test");


    Command::new("cc").args(&["src/tlsf_c/tlsf.c", "-O3","-c", "-fPIC", "-o"])
        .arg(&format!("{}/tlsf.o", out_dir))
        .status().unwrap();
 
    Command::new("ar").args(&["crus", "libtlsf.a", "tlsf.o"])
        .current_dir(&Path::new(&out_dir))
        .status().unwrap();
 
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=tlsf");


    Command::new("cc").args(&["mimalloc_test_1/mitest.c", "-O3","-c", "-fPIC", "-o"])
        .arg(&format!("{}/mitest.o", out_dir))
        .status().unwrap();
 
    Command::new("ar").args(&["crus", "libmitest.a", "mitest.o"])
        .current_dir(&Path::new(&out_dir))
        .status().unwrap();
 
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=mitest");

    
}