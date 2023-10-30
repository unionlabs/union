use super::AutoCfg;
use std::env;
use std::path::Path;

impl AutoCfg {
    fn core_std(&self, path: &str) -> String {
        let krate = if self.no_std { "core" } else { "std" };
        format!("{}::{}", krate, path)
    }

    fn assert_std(&self, probe_result: bool) {
        assert_eq!(!self.no_std, probe_result);
    }

    fn assert_min(&self, major: usize, minor: usize, probe_result: bool) {
        assert_eq!(self.probe_rustc_version(major, minor), probe_result);
    }

    fn for_test() -> Result<Self, super::error::Error> {
        match env::var_os("TESTS_TARGET_DIR") {
            Some(d) => Self::with_dir(d),
            None => Self::with_dir("target"),
        }
    }
}

#[test]
fn autocfg_version() {
    let ac = AutoCfg::for_test().unwrap();
    println!("version: {:?}", ac.rustc_version);
    assert!(ac.probe_rustc_version(1, 0));
}

#[test]
fn version_cmp() {
    use super::version::Version;
    let v123 = Version::new(1, 2, 3);

    assert!(Version::new(1, 0, 0) < v123);
    assert!(Version::new(1, 2, 2) < v123);
    assert!(Version::new(1, 2, 3) == v123);
    assert!(Version::new(1, 2, 4) > v123);
    assert!(Version::new(1, 10, 0) > v123);
    assert!(Version::new(2, 0, 0) > v123);
}

#[test]
fn probe_add() {
    let ac = AutoCfg::for_test().unwrap();
    let add = ac.core_std("ops::Add");
    let add_rhs = add.clone() + "<i32>";
    let add_rhs_output = add.clone() + "<i32, Output = i32>";
    let dyn_add_rhs_output = "dyn ".to_string() + &*add_rhs_output;
    assert!(ac.probe_path(&add));
    assert!(ac.probe_trait(&add));
    assert!(ac.probe_trait(&add_rhs));
    assert!(ac.probe_trait(&add_rhs_output));
    ac.assert_min(1, 27, ac.probe_type(&dyn_add_rhs_output));
}

#[test]
fn probe_as_ref() {
    let ac = AutoCfg::for_test().unwrap();
    let as_ref = ac.core_std("convert::AsRef");
    let as_ref_str = as_ref.clone() + "<str>";
    let dyn_as_ref_str = "dyn ".to_string() + &*as_ref_str;
    assert!(ac.probe_path(&as_ref));
    assert!(ac.probe_trait(&as_ref_str));
    assert!(ac.probe_type(&as_ref_str));
    ac.assert_min(1, 27, ac.probe_type(&dyn_as_ref_str));
}

#[test]
fn probe_i128() {
    let ac = AutoCfg::for_test().unwrap();
    let i128_path = ac.core_std("i128");
    ac.assert_min(1, 26, ac.probe_path(&i128_path));
    ac.assert_min(1, 26, ac.probe_type("i128"));
}

#[test]
fn probe_sum() {
    let ac = AutoCfg::for_test().unwrap();
    let sum = ac.core_std("iter::Sum");
    let sum_i32 = sum.clone() + "<i32>";
    let dyn_sum_i32 = "dyn ".to_string() + &*sum_i32;
    ac.assert_min(1, 12, ac.probe_path(&sum));
    ac.assert_min(1, 12, ac.probe_trait(&sum));
    ac.assert_min(1, 12, ac.probe_trait(&sum_i32));
    ac.assert_min(1, 12, ac.probe_type(&sum_i32));
    ac.assert_min(1, 27, ac.probe_type(&dyn_sum_i32));
}

#[test]
fn probe_std() {
    let ac = AutoCfg::for_test().unwrap();
    ac.assert_std(ac.probe_sysroot_crate("std"));
}

#[test]
fn probe_alloc() {
    let ac = AutoCfg::for_test().unwrap();
    ac.assert_min(1, 36, ac.probe_sysroot_crate("alloc"));
}

#[test]
fn probe_bad_sysroot_crate() {
    let ac = AutoCfg::for_test().unwrap();
    assert!(!ac.probe_sysroot_crate("doesnt_exist"));
}

#[test]
fn probe_no_std() {
    let ac = AutoCfg::for_test().unwrap();
    assert!(ac.probe_type("i32"));
    assert!(ac.probe_type("[i32]"));
    ac.assert_std(ac.probe_type("Vec<i32>"));
}

#[test]
fn probe_expression() {
    let ac = AutoCfg::for_test().unwrap();
    assert!(ac.probe_expression(r#""test".trim_left()"#));
    ac.assert_min(1, 30, ac.probe_expression(r#""test".trim_start()"#));
    ac.assert_std(ac.probe_expression("[1, 2, 3].to_vec()"));
}

#[test]
fn probe_constant() {
    let ac = AutoCfg::for_test().unwrap();
    assert!(ac.probe_constant("1 + 2 + 3"));
    ac.assert_min(1, 33, ac.probe_constant("{ let x = 1 + 2 + 3; x * x }"));
    ac.assert_min(1, 39, ac.probe_constant(r#""test".len()"#));
}

#[test]
fn dir_does_not_contain_target() {
    assert!(!super::dir_contains_target(
        &Some("x86_64-unknown-linux-gnu".into()),
        Path::new("/project/target/debug/build/project-ea75983148559682/out"),
        None,
    ));
}

#[test]
fn dir_does_contain_target() {
    assert!(super::dir_contains_target(
        &Some("x86_64-unknown-linux-gnu".into()),
        Path::new(
            "/project/target/x86_64-unknown-linux-gnu/debug/build/project-0147aca016480b9d/out"
        ),
        None,
    ));
}

#[test]
fn dir_does_not_contain_target_with_custom_target_dir() {
    assert!(!super::dir_contains_target(
        &Some("x86_64-unknown-linux-gnu".into()),
        Path::new("/project/custom/debug/build/project-ea75983148559682/out"),
        Some("custom".into()),
    ));
}

#[test]
fn dir_does_contain_target_with_custom_target_dir() {
    assert!(super::dir_contains_target(
        &Some("x86_64-unknown-linux-gnu".into()),
        Path::new(
            "/project/custom/x86_64-unknown-linux-gnu/debug/build/project-0147aca016480b9d/out"
        ),
        Some("custom".into()),
    ));
}
