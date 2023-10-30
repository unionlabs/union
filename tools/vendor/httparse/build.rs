use std::env;
//use std::ffi::OsString;
//use std::process::Command;

fn main() {
    // We don't currently need to check the Version anymore...
    // But leaving this in place in case we need to in the future.
    /*
    let rustc = env::var_os("RUSTC").unwrap_or(OsString::from("rustc"));
    let output = Command::new(&rustc)
        .arg("--version")
        .output()
        .expect("failed to check 'rustc --version'")
        .stdout;

    let version = String::from_utf8(output)
        .expect("rustc version output should be utf-8");
    */

    enable_new_features(/*&version*/);
}

fn enable_new_features(/*raw_version: &str*/) {
    /*
    let version = match Version::parse(raw_version) {
        Ok(version) => version,
        Err(err) => {
            println!("cargo:warning=failed to parse `rustc --version`: {}", err);
            return;
        }
    };
    */

    enable_simd(/*version*/);
}

fn enable_simd(/*version: Version*/) {
    if env::var_os("CARGO_FEATURE_STD").is_none() {
        println!("cargo:warning=building for no_std disables httparse SIMD");
        return;
    }
    if env::var_os("CARGO_CFG_MIRI").is_some() {
        println!("cargo:warning=building for Miri disables httparse SIMD");
        return;
    }

    let env_disable = "CARGO_CFG_HTTPARSE_DISABLE_SIMD";
    if var_is(env_disable, "1") {
        println!("cargo:warning=detected {} environment variable, disabling SIMD", env_disable);
        return;
    }

    println!("cargo:rustc-cfg=httparse_simd");

    // cfg(target_feature) isn't stable yet, but CARGO_CFG_TARGET_FEATURE has
    // a list... We aren't doing anything unsafe, since the is_x86_feature_detected
    // macro still checks in the actual lib, BUT!
    //
    // By peeking at the list here, we can change up slightly how we do feature
    // detection in the lib. If our features aren't in the feature list, we
    // stick with a cached runtime detection strategy.
    //
    // But if the features *are* in the list, we benefit from removing our cache,
    // since the compiler will eliminate several branches with its internal
    // cfg(target_feature) usage.


    let env_runtime_only = "CARGO_CFG_HTTPARSE_DISABLE_SIMD_COMPILETIME";
    if var_is(env_runtime_only, "1") {
        println!("cargo:warning=detected {} environment variable, using runtime SIMD detection only", env_runtime_only);
        return;
    }
    let feature_list = match env::var_os("CARGO_CFG_TARGET_FEATURE") {
        Some(var) => match var.into_string() {
            Ok(s) => s,
            Err(_) => {
                println!("cargo:warning=CARGO_CFG_TARGET_FEATURE was not valid utf-8");
                return;
            },
        },
        None => {
            println!("cargo:warning=CARGO_CFG_TARGET_FEATURE was not set");
            return
        },
    };

    let mut saw_sse42 = false;
    let mut saw_avx2 = false;

    for feature in feature_list.split(',') {
        let feature = feature.trim();
        if !saw_sse42 && feature == "sse4.2" {
            saw_sse42 = true;
            println!("cargo:rustc-cfg=httparse_simd_target_feature_sse42");
        }

        if !saw_avx2 && feature == "avx2" {
            saw_avx2 = true;
            println!("cargo:rustc-cfg=httparse_simd_target_feature_avx2");
        }
    }
}

/*
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    fn parse(mut s: &str) -> Result<Version, String> {
        if !s.starts_with("rustc ") {
            return Err(format!("unrecognized version string: {}", s));
        }
        s = &s["rustc ".len()..];

        let parts: Vec<&str> = s.split(".").collect();
        if parts.len() < 3 {
            return Err(format!("not enough version parts: {:?}", parts));
        }

        let mut num = String::new();
        for c in parts[0].chars() {
            if !c.is_digit(10) {
                break;
            }
            num.push(c);
        }
        let major = num.parse::<u32>().map_err(|e| e.to_string())?;

        num.clear();
        for c in parts[1].chars() {
            if !c.is_digit(10) {
                break;
            }
            num.push(c);
        }
        let minor = num.parse::<u32>().map_err(|e| e.to_string())?;

        num.clear();
        for c in parts[2].chars() {
            if !c.is_digit(10) {
                break;
            }
            num.push(c);
        }
        let patch = num.parse::<u32>().map_err(|e| e.to_string())?;

        Ok(Version {
            major: major,
            minor: minor,
            patch: patch,
        })
    }
}
*/

fn var_is(key: &str, val: &str) -> bool {
    match env::var(key) {
        Ok(v) => v == val,
        Err(_) => false,
    }
}
