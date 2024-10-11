fn main() {
    use std::process::Command;
    let project_root = std::env::current_dir()
        .unwrap()
        .parent() // flatbuffers/tests/rust_usage test
        .unwrap()
        .parent() // flatbuffers/tests
        .unwrap()
        .parent() // flatbuffers/
        .unwrap()
        .to_path_buf();
    let sample_schema = {
        let mut s = project_root.to_path_buf();
        s.push("samples");
        s.push("monster.fbs");
        s
    };

    let flatc = {
        let mut f = project_root.to_path_buf();
        f.push("flatc");
        f
    };

    let out_dir = {
        let mut d = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).to_path_buf();
        d.push("flatbuffers");
        d
    };

    Command::new(&flatc)
        .arg("-o")
        .arg(&out_dir)
        .arg("--rust")
        .arg("--rust-module-root-file")
        .arg(&sample_schema)
        .output()
        .expect("Failed to generate file");

    assert!(out_dir.exists());

    let generated = std::path::Path::new("src/generated");
    #[cfg(target_os = "windows")]
    {
        if generated.exists() {
            std::fs::remove_dir(generated).unwrap();
        }
        std::os::windows::fs::symlink_dir(out_dir, generated).unwrap();
    }
    #[cfg(not(target_os = "windows"))]
    {
        if generated.exists() {
            std::fs::remove_file(generated).unwrap();
        }
        std::os::unix::fs::symlink(out_dir, generated).unwrap();
    }
}
