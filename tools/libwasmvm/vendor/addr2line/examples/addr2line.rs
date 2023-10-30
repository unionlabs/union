extern crate addr2line;
extern crate clap;
extern crate fallible_iterator;
extern crate gimli;
extern crate memmap2;
extern crate object;
extern crate typed_arena;

use std::borrow::Cow;
use std::fs::File;
use std::io::{BufRead, Lines, StdinLock, Write};
use std::path::Path;

use clap::{Arg, Command, Values};
use fallible_iterator::FallibleIterator;
use object::{Object, ObjectSection, SymbolMap, SymbolMapName};
use typed_arena::Arena;

use addr2line::{Context, Location};

fn parse_uint_from_hex_string(string: &str) -> Option<u64> {
    if string.len() > 2 && string.starts_with("0x") {
        u64::from_str_radix(&string[2..], 16).ok()
    } else {
        u64::from_str_radix(string, 16).ok()
    }
}

enum Addrs<'a> {
    Args(Values<'a>),
    Stdin(Lines<StdinLock<'a>>),
}

impl<'a> Iterator for Addrs<'a> {
    type Item = Option<u64>;

    fn next(&mut self) -> Option<Option<u64>> {
        let text = match *self {
            Addrs::Args(ref mut vals) => vals.next().map(Cow::from),
            Addrs::Stdin(ref mut lines) => lines.next().map(Result::unwrap).map(Cow::from),
        };
        text.as_ref()
            .map(Cow::as_ref)
            .map(parse_uint_from_hex_string)
    }
}

fn print_loc(loc: Option<&Location>, basenames: bool, llvm: bool) {
    if let Some(ref loc) = loc {
        if let Some(ref file) = loc.file.as_ref() {
            let path = if basenames {
                Path::new(Path::new(file).file_name().unwrap())
            } else {
                Path::new(file)
            };
            print!("{}:", path.display());
        } else {
            print!("??:");
        }
        if llvm {
            print!("{}:{}", loc.line.unwrap_or(0), loc.column.unwrap_or(0));
        } else if let Some(line) = loc.line {
            print!("{}", line);
        } else {
            print!("?");
        }
        println!();
    } else if llvm {
        println!("??:0:0");
    } else {
        println!("??:0");
    }
}

fn print_function(name: Option<&str>, language: Option<gimli::DwLang>, demangle: bool) {
    if let Some(name) = name {
        if demangle {
            print!("{}", addr2line::demangle_auto(Cow::from(name), language));
        } else {
            print!("{}", name);
        }
    } else {
        print!("??");
    }
}

fn load_file_section<'input, 'arena, Endian: gimli::Endianity>(
    id: gimli::SectionId,
    file: &object::File<'input>,
    endian: Endian,
    arena_data: &'arena Arena<Cow<'input, [u8]>>,
) -> Result<gimli::EndianSlice<'arena, Endian>, ()> {
    // TODO: Unify with dwarfdump.rs in gimli.
    let name = id.name();
    match file.section_by_name(name) {
        Some(section) => match section.uncompressed_data().unwrap() {
            Cow::Borrowed(b) => Ok(gimli::EndianSlice::new(b, endian)),
            Cow::Owned(b) => Ok(gimli::EndianSlice::new(arena_data.alloc(b.into()), endian)),
        },
        None => Ok(gimli::EndianSlice::new(&[][..], endian)),
    }
}

fn find_name_from_symbols<'a>(
    symbols: &'a SymbolMap<SymbolMapName>,
    probe: u64,
) -> Option<&'a str> {
    symbols.get(probe).map(|x| x.name())
}

fn main() {
    let matches = Command::new("addr2line")
        .version("0.1")
        .about("A fast addr2line Rust port")
        .args(&[
            Arg::new("exe")
                .short('e')
                .long("exe")
                .value_name("filename")
                .help(
                    "Specify the name of the executable for which addresses should be translated.",
                )
                .required(true),
            Arg::new("sup")
                .long("sup")
                .value_name("filename")
                .help("Path to supplementary object file."),
            Arg::new("functions")
                .short('f')
                .long("functions")
                .help("Display function names as well as file and line number information."),
            Arg::new("pretty").short('p').long("pretty-print").help(
                "Make the output more human friendly: each location are printed on one line.",
            ),
            Arg::new("inlines").short('i').long("inlines").help(
                "If the address belongs to a function that was inlined, the source information for \
                all enclosing scopes back to the first non-inlined function will also be printed.",
            ),
            Arg::new("addresses").short('a').long("addresses").help(
                "Display the address before the function name, file and line number information.",
            ),
            Arg::new("basenames")
                .short('s')
                .long("basenames")
                .help("Display only the base of each file name."),
            Arg::new("demangle").short('C').long("demangle").help(
                "Demangle function names. \
                Specifying a specific demangling style (like GNU addr2line) is not supported. \
                (TODO)"
            ),
            Arg::new("llvm")
                .long("llvm")
                .help("Display output in the same format as llvm-symbolizer."),
            Arg::new("addrs")
                .takes_value(true)
                .multiple_occurrences(true)
                .help("Addresses to use instead of reading from stdin."),
        ])
        .get_matches();

    let arena_data = Arena::new();

    let do_functions = matches.is_present("functions");
    let do_inlines = matches.is_present("inlines");
    let pretty = matches.is_present("pretty");
    let print_addrs = matches.is_present("addresses");
    let basenames = matches.is_present("basenames");
    let demangle = matches.is_present("demangle");
    let llvm = matches.is_present("llvm");
    let path = matches.value_of("exe").unwrap();

    let file = File::open(path).unwrap();
    let map = unsafe { memmap2::Mmap::map(&file).unwrap() };
    let object = &object::File::parse(&*map).unwrap();

    let endian = if object.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };

    let mut load_section = |id: gimli::SectionId| -> Result<_, _> {
        load_file_section(id, object, endian, &arena_data)
    };

    let sup_map;
    let sup_object = if let Some(sup_path) = matches.value_of("sup") {
        let sup_file = File::open(sup_path).unwrap();
        sup_map = unsafe { memmap2::Mmap::map(&sup_file).unwrap() };
        Some(object::File::parse(&*sup_map).unwrap())
    } else {
        None
    };

    let symbols = object.symbol_map();
    let mut dwarf = gimli::Dwarf::load(&mut load_section).unwrap();
    if let Some(ref sup_object) = sup_object {
        let mut load_sup_section = |id: gimli::SectionId| -> Result<_, _> {
            load_file_section(id, sup_object, endian, &arena_data)
        };
        dwarf.load_sup(&mut load_sup_section).unwrap();
    }

    let ctx = Context::from_dwarf(dwarf).unwrap();

    let stdin = std::io::stdin();
    let addrs = matches
        .values_of("addrs")
        .map(Addrs::Args)
        .unwrap_or_else(|| Addrs::Stdin(stdin.lock().lines()));

    for probe in addrs {
        if print_addrs {
            let addr = probe.unwrap_or(0);
            if llvm {
                print!("0x{:x}", addr);
            } else {
                print!("0x{:016x}", addr);
            }
            if pretty {
                print!(": ");
            } else {
                println!();
            }
        }

        if do_functions || do_inlines {
            let mut printed_anything = false;
            if let Some(probe) = probe {
                let mut frames = ctx.find_frames(probe).unwrap().enumerate();
                while let Some((i, frame)) = frames.next().unwrap() {
                    if pretty && i != 0 {
                        print!(" (inlined by) ");
                    }

                    if do_functions {
                        if let Some(func) = frame.function {
                            print_function(
                                func.raw_name().ok().as_ref().map(AsRef::as_ref),
                                func.language,
                                demangle,
                            );
                        } else {
                            let name = find_name_from_symbols(&symbols, probe);
                            print_function(name, None, demangle);
                        }

                        if pretty {
                            print!(" at ");
                        } else {
                            println!();
                        }
                    }

                    print_loc(frame.location.as_ref(), basenames, llvm);

                    printed_anything = true;

                    if !do_inlines {
                        break;
                    }
                }
            }

            if !printed_anything {
                if do_functions {
                    let name = probe.and_then(|probe| find_name_from_symbols(&symbols, probe));
                    print_function(name, None, demangle);

                    if pretty {
                        print!(" at ");
                    } else {
                        println!();
                    }
                }

                print_loc(None, basenames, llvm);
            }
        } else {
            let loc = probe.and_then(|probe| ctx.find_location(probe).unwrap());
            print_loc(loc.as_ref(), basenames, llvm);
        }

        if llvm {
            println!();
        }
        std::io::stdout().flush().unwrap();
    }
}
