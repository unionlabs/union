extern crate addr2line;
extern crate fallible_iterator;
extern crate findshlibs;
extern crate gimli;
extern crate memmap2;
extern crate object;

use addr2line::Context;
use fallible_iterator::FallibleIterator;
use findshlibs::{IterationControl, SharedLibrary, TargetSharedLibrary};
use object::Object;
use std::fs::File;

fn find_debuginfo() -> memmap2::Mmap {
    let path = std::env::current_exe().unwrap();
    let file = File::open(&path).unwrap();
    let map = unsafe { memmap2::Mmap::map(&file).unwrap() };
    let file = &object::File::parse(&*map).unwrap();
    if let Ok(uuid) = file.mach_uuid() {
        for candidate in path.parent().unwrap().read_dir().unwrap() {
            let path = candidate.unwrap().path();
            if !path.to_str().unwrap().ends_with(".dSYM") {
                continue;
            }
            for candidate in path.join("Contents/Resources/DWARF").read_dir().unwrap() {
                let path = candidate.unwrap().path();
                let file = File::open(&path).unwrap();
                let map = unsafe { memmap2::Mmap::map(&file).unwrap() };
                let file = &object::File::parse(&*map).unwrap();
                if file.mach_uuid().unwrap() == uuid {
                    return map;
                }
            }
        }
    }

    return map;
}

#[test]
fn correctness() {
    let map = find_debuginfo();
    let file = &object::File::parse(&*map).unwrap();
    let module_base = file.relative_address_base();
    let ctx = Context::new(file).unwrap();

    let mut bias = None;
    TargetSharedLibrary::each(|lib| {
        bias = Some((lib.virtual_memory_bias().0 as u64).wrapping_sub(module_base));
        IterationControl::Break
    });

    let test = |sym: u64, expected_prefix: &str| {
        let ip = sym.wrapping_sub(bias.unwrap());

        let frames = ctx.find_frames(ip).unwrap();
        let frame = frames.last().unwrap().unwrap();
        let name = frame.function.as_ref().unwrap().demangle().unwrap();
        // Old rust versions generate DWARF with wrong linkage name,
        // so only check the start.
        if !name.starts_with(expected_prefix) {
            panic!("incorrect name '{}', expected {:?}", name, expected_prefix);
        }
    };

    test(test_function as u64, "correctness::test_function");
    test(
        small::test_function as u64,
        "correctness::small::test_function",
    );
    test(auxiliary::foo as u64, "auxiliary::foo");
}

mod small {
    pub fn test_function() {
        println!("y");
    }
}

fn test_function() {
    println!("x");
}

#[test]
fn zero_function() {
    let map = find_debuginfo();
    let file = &object::File::parse(&*map).unwrap();
    let ctx = Context::new(file).unwrap();
    for probe in 0..10 {
        assert!(ctx.find_frames(probe).unwrap().count().unwrap() < 10);
    }
}
