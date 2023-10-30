extern crate addr2line;
extern crate memmap2;
extern crate object;

use std::borrow::Cow;
use std::env;
use std::fs::File;
use std::path::{self, PathBuf};

use object::Object;

fn release_fixture_path() -> PathBuf {
    if let Ok(p) = env::var("ADDR2LINE_FIXTURE_PATH") {
        return p.into();
    }

    let mut path = PathBuf::new();
    if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        path.push(dir);
    }
    path.push("fixtures");
    path.push("addr2line-release");
    path
}

fn with_file<F: FnOnce(&object::File)>(target: &path::Path, f: F) {
    let file = File::open(target).unwrap();
    let map = unsafe { memmap2::Mmap::map(&file).unwrap() };
    let file = object::File::parse(&*map).unwrap();
    f(&file)
}

fn dwarf_load<'a>(object: &object::File<'a>) -> gimli::Dwarf<Cow<'a, [u8]>> {
    let load_section = |id: gimli::SectionId| -> Result<Cow<'a, [u8]>, gimli::Error> {
        use object::ObjectSection;

        let data = object
            .section_by_name(id.name())
            .and_then(|section| section.data().ok())
            .unwrap_or(&[][..]);
        Ok(Cow::Borrowed(data))
    };
    gimli::Dwarf::load(&load_section).unwrap()
}

fn dwarf_borrow<'a>(
    dwarf: &'a gimli::Dwarf<Cow<[u8]>>,
) -> gimli::Dwarf<gimli::EndianSlice<'a, gimli::LittleEndian>> {
    let borrow_section: &dyn for<'b> Fn(
        &'b Cow<[u8]>,
    ) -> gimli::EndianSlice<'b, gimli::LittleEndian> =
        &|section| gimli::EndianSlice::new(&*section, gimli::LittleEndian);
    dwarf.borrow(&borrow_section)
}

#[test]
fn parse_base_rc() {
    let target = release_fixture_path();

    with_file(&target, |file| {
        addr2line::ObjectContext::new(file).unwrap();
    });
}

#[test]
fn parse_base_slice() {
    let target = release_fixture_path();

    with_file(&target, |file| {
        let dwarf = dwarf_load(file);
        let dwarf = dwarf_borrow(&dwarf);
        addr2line::Context::from_dwarf(dwarf).unwrap();
    });
}

#[test]
fn parse_lines_rc() {
    let target = release_fixture_path();

    with_file(&target, |file| {
        let context = addr2line::ObjectContext::new(file).unwrap();
        context.parse_lines().unwrap();
    });
}

#[test]
fn parse_lines_slice() {
    let target = release_fixture_path();

    with_file(&target, |file| {
        let dwarf = dwarf_load(file);
        let dwarf = dwarf_borrow(&dwarf);
        let context = addr2line::Context::from_dwarf(dwarf).unwrap();
        context.parse_lines().unwrap();
    });
}

#[test]
fn parse_functions_rc() {
    let target = release_fixture_path();

    with_file(&target, |file| {
        let context = addr2line::ObjectContext::new(file).unwrap();
        context.parse_functions().unwrap();
    });
}

#[test]
fn parse_functions_slice() {
    let target = release_fixture_path();

    with_file(&target, |file| {
        let dwarf = dwarf_load(file);
        let dwarf = dwarf_borrow(&dwarf);
        let context = addr2line::Context::from_dwarf(dwarf).unwrap();
        context.parse_functions().unwrap();
    });
}
