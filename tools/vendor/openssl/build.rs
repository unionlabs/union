#![allow(clippy::inconsistent_digit_grouping, clippy::unusual_byte_groupings)]

use std::env;

fn main() {
    if env::var("DEP_OPENSSL_LIBRESSL").is_ok() {
        println!("cargo:rustc-cfg=libressl");
    }

    if let Ok(v) = env::var("DEP_OPENSSL_LIBRESSL_VERSION") {
        println!("cargo:rustc-cfg=libressl{}", v);
    }

    if let Ok(vars) = env::var("DEP_OPENSSL_CONF") {
        for var in vars.split(',') {
            println!("cargo:rustc-cfg=osslconf=\"{}\"", var);
        }
    }

    if let Ok(version) = env::var("DEP_OPENSSL_VERSION_NUMBER") {
        let version = u64::from_str_radix(&version, 16).unwrap();

        if version >= 0x1_00_01_00_0 {
            println!("cargo:rustc-cfg=ossl101");
        }
        if version >= 0x1_00_02_00_0 {
            println!("cargo:rustc-cfg=ossl102");
        }
        if version >= 0x1_01_00_00_0 {
            println!("cargo:rustc-cfg=ossl110");
        }
        if version >= 0x1_01_00_07_0 {
            println!("cargo:rustc-cfg=ossl110g");
        }
        if version >= 0x1_01_00_08_0 {
            println!("cargo:rustc-cfg=ossl110h");
        }
        if version >= 0x1_01_01_00_0 {
            println!("cargo:rustc-cfg=ossl111");
        }
        if version >= 0x3_00_00_00_0 {
            println!("cargo:rustc-cfg=ossl300");
        }
    }

    if let Ok(version) = env::var("DEP_OPENSSL_LIBRESSL_VERSION_NUMBER") {
        let version = u64::from_str_radix(&version, 16).unwrap();

        if version >= 0x2_05_01_00_0 {
            println!("cargo:rustc-cfg=libressl251");
        }

        if version >= 0x2_06_01_00_0 {
            println!("cargo:rustc-cfg=libressl261");
        }

        if version >= 0x2_07_00_00_0 {
            println!("cargo:rustc-cfg=libressl270");
        }

        if version >= 0x2_07_01_00_0 {
            println!("cargo:rustc-cfg=libressl271");
        }

        if version >= 0x2_07_03_00_0 {
            println!("cargo:rustc-cfg=libressl273");
        }

        if version >= 0x2_08_00_00_0 {
            println!("cargo:rustc-cfg=libressl280");
        }

        if version >= 0x2_09_01_00_0 {
            println!("cargo:rustc-cfg=libressl291");
        }

        if version >= 0x3_02_01_00_0 {
            println!("cargo:rustc-cfg=libressl321");
        }

        if version >= 0x3_03_02_00_0 {
            println!("cargo:rustc-cfg=libressl332");
        }

        if version >= 0x3_04_00_00_0 {
            println!("cargo:rustc-cfg=libressl340");
        }

        if version >= 0x3_05_00_00_0 {
            println!("cargo:rustc-cfg=libressl350");
        }
    }
}
