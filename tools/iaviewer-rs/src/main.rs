use std::{
    env::args,
    io::{self, Write},
    iter::FromIterator,
};

use rusty_leveldb::{compressor, CompressorId, LdbIterator, Options, DB};

fn get(db: &mut DB, k: &str) {
    match db.get(k.as_bytes()) {
        Some(v) => {
            if let Ok(s) = String::from_utf8(v.to_vec()) {
                eprintln!("{} => {}", k, s);
            } else {
                eprintln!("{} => {:?}", k, v);
            }
        }
        None => eprintln!("{} => <not found>", k),
    }
}

fn put(db: &mut DB, k: &str, v: &str) {
    db.put(k.as_bytes(), v.as_bytes()).unwrap();
    db.flush().unwrap();
}

fn delete(db: &mut DB, k: &str) {
    db.delete(k.as_bytes()).unwrap();
    db.flush().unwrap();
}

fn iter(db: &mut DB) {
    let mut it = db.new_iter().unwrap();
    let (mut k, mut v) = (Default::default(), Default::default());
    let mut out = io::BufWriter::new(io::stdout());
    while it.advance() {
        (k, v) = it.current().unwrap();
        out.write_all(&k).unwrap();
        out.write_all(b" => ").unwrap();
        out.write_all(&v).unwrap();
        out.write_all(b"\n").unwrap();
    }
}

fn compact(db: &mut DB, from: &str, to: &str) {
    db.compact_range(from.as_bytes(), to.as_bytes()).unwrap();
}

fn main() {
    let args = Vec::from_iter(args());

    if args.len() < 3 {
        panic!(
            "Usage: {} [db] [get|put/set|delete|iter|compact] [key|from] [val|to]",
            args[0]
        );
    }

    let opt = Options {
        reuse_logs: false,
        reuse_manifest: false,
        compressor: compressor::SnappyCompressor::ID,
        ..Default::default()
    };
    let mut db = DB::open(&args[1], opt).unwrap();

    match args[2].as_str() {
        "get" => {
            if args.len() < 3 {
                panic!("Usage: {} get key", args[1]);
            }
            get(&mut db, &args[3]);
        }
        "put" | "set" => {
            if args.len() < 4 {
                panic!("Usage: {} put key val", args[1]);
            }
            put(&mut db, &args[3], &args[4]);
        }
        "delete" => {
            if args.len() < 3 {
                panic!("Usage: {} delete key", args[1]);
            }
            delete(&mut db, &args[3]);
        }
        "iter" => iter(&mut db),
        "compact" => {
            if args.len() < 4 {
                panic!("Usage: {} compact from to", args[1]);
            }
            compact(&mut db, &args[3], &args[4]);
        }
        _ => unimplemented!(),
    }
}
