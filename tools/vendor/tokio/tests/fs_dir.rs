#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]

use tokio::fs;
use tokio_test::{assert_err, assert_ok};

use std::sync::{Arc, Mutex};
use tempfile::tempdir;

#[tokio::test]
async fn create_dir() {
    let base_dir = tempdir().unwrap();
    let new_dir = base_dir.path().join("foo");
    let new_dir_2 = new_dir.clone();

    assert_ok!(fs::create_dir(new_dir).await);

    assert!(new_dir_2.is_dir());
}

#[tokio::test]
async fn create_all() {
    let base_dir = tempdir().unwrap();
    let new_dir = base_dir.path().join("foo").join("bar");
    let new_dir_2 = new_dir.clone();

    assert_ok!(fs::create_dir_all(new_dir).await);
    assert!(new_dir_2.is_dir());
}

#[tokio::test]
async fn build_dir() {
    let base_dir = tempdir().unwrap();
    let new_dir = base_dir.path().join("foo").join("bar");
    let new_dir_2 = new_dir.clone();

    assert_ok!(fs::DirBuilder::new().recursive(true).create(new_dir).await);

    assert!(new_dir_2.is_dir());
    assert_err!(
        fs::DirBuilder::new()
            .recursive(false)
            .create(new_dir_2)
            .await
    );
}

#[tokio::test]
async fn remove() {
    let base_dir = tempdir().unwrap();
    let new_dir = base_dir.path().join("foo");
    let new_dir_2 = new_dir.clone();

    std::fs::create_dir(new_dir.clone()).unwrap();

    assert_ok!(fs::remove_dir(new_dir).await);
    assert!(!new_dir_2.exists());
}

#[tokio::test]
async fn read_inherent() {
    let base_dir = tempdir().unwrap();

    let p = base_dir.path();
    std::fs::create_dir(p.join("aa")).unwrap();
    std::fs::create_dir(p.join("bb")).unwrap();
    std::fs::create_dir(p.join("cc")).unwrap();

    let files = Arc::new(Mutex::new(Vec::new()));

    let f = files.clone();
    let p = p.to_path_buf();

    let mut entries = fs::read_dir(p).await.unwrap();

    while let Some(e) = assert_ok!(entries.next_entry().await) {
        let s = e.file_name().to_str().unwrap().to_string();
        f.lock().unwrap().push(s);
    }

    let mut files = files.lock().unwrap();
    files.sort(); // because the order is not guaranteed
    assert_eq!(
        *files,
        vec!["aa".to_string(), "bb".to_string(), "cc".to_string()]
    );
}
