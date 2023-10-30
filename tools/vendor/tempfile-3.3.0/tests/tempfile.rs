#![deny(rust_2018_idioms)]

use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::mpsc::{sync_channel, TryRecvError};
use std::thread;

#[test]
fn test_basic() {
    let mut tmpfile = tempfile::tempfile().unwrap();
    write!(tmpfile, "abcde").unwrap();
    tmpfile.seek(SeekFrom::Start(0)).unwrap();
    let mut buf = String::new();
    tmpfile.read_to_string(&mut buf).unwrap();
    assert_eq!("abcde", buf);
}

#[test]
fn test_cleanup() {
    let tmpdir = tempfile::tempdir().unwrap();
    {
        let mut tmpfile = tempfile::tempfile_in(&tmpdir).unwrap();
        write!(tmpfile, "abcde").unwrap();
    }
    let num_files = fs::read_dir(&tmpdir).unwrap().count();
    assert!(num_files == 0);
}

// Only run this test on Linux. MacOS doesn't like us creating so many files, apparently.
#[cfg(target_os = "linux")]
#[test]
fn test_pathological_cleaner() {
    let tmpdir = tempfile::tempdir().unwrap();
    let (tx, rx) = sync_channel(0);
    let cleaner_thread = thread::spawn(move || {
        let tmp_path = rx.recv().unwrap();
        while rx.try_recv() == Err(TryRecvError::Empty) {
            let files = fs::read_dir(&tmp_path).unwrap();
            for f in files {
                // skip errors
                if f.is_err() {
                    continue;
                }
                let f = f.unwrap();
                let _ = fs::remove_file(f.path());
            }
        }
    });

    // block until cleaner_thread makes progress
    tx.send(tmpdir.path().to_owned()).unwrap();
    // need 40-400 iterations to encounter race with cleaner on original system
    for _ in 0..10000 {
        let mut tmpfile = tempfile::tempfile_in(&tmpdir).unwrap();
        write!(tmpfile, "abcde").unwrap();
        tmpfile.seek(SeekFrom::Start(0)).unwrap();
        let mut buf = String::new();
        tmpfile.read_to_string(&mut buf).unwrap();
        assert_eq!("abcde", buf);
    }

    // close the channel to make cleaner_thread exit
    drop(tx);
    cleaner_thread.join().expect("The cleaner thread failed");
}
