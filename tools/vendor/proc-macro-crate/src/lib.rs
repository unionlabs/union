/*!

[![](https://docs.rs/proc-macro-crate/badge.svg)](https://docs.rs/proc-macro-crate/) [![](https://img.shields.io/crates/v/proc-macro-crate.svg)](https://crates.io/crates/proc-macro-crate) [![](https://img.shields.io/crates/d/proc-macro-crate.png)](https://crates.io/crates/proc-macro-crate) [![Build Status](https://travis-ci.org/bkchr/proc-macro-crate.png?branch=master)](https://travis-ci.org/bkchr/proc-macro-crate)

Providing support for `$crate` in procedural macros.

* [Introduction](#introduction)
* [Example](#example)
* [License](#license)

## Introduction

In `macro_rules!` `$crate` is used to get the path of the crate where a macro is declared in. In
procedural macros there is currently no easy way to get this path. A common hack is to import the
desired crate with a know name and use this. However, with rust edition 2018 and dropping
`extern crate` declarations from `lib.rs`, people start to rename crates in `Cargo.toml` directly.
However, this breaks importing the crate, as the proc-macro developer does not know the renamed
name of the crate that should be imported.

This crate provides a way to get the name of a crate, even if it renamed in `Cargo.toml`. For this
purpose a single function `crate_name` is provided. This function needs to be called in the context
of a proc-macro with the name of the desired crate. `CARGO_MANIFEST_DIR` will be used to find the
current active `Cargo.toml` and this `Cargo.toml` is searched for the desired crate. The returned
name of `crate_name` is either the given original rename (crate not renamed) or the renamed name.

## Example

```
use quote::quote;
use syn::Ident;
use proc_macro2::Span;
use proc_macro_crate::crate_name;

fn import_my_crate() {
    let name = crate_name("my-crate").expect("my-crate is present in `Cargo.toml`");
    let ident = Ident::new(&name, Span::call_site());
    quote!( extern crate #ident as my_crate_known_name );
}

# fn main() {}
```

## License

Licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)

 * [MIT license](http://opensource.org/licenses/MIT)

at your option.
*/

use std::{
    collections::HashMap,
    env,
    fmt::Display,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use toml::{self, value::Table};

type CargoToml = HashMap<String, toml::Value>;

/// Find the crate name for the given `orig_name` in the current `Cargo.toml`.
///
/// `orig_name` should be the original name of the searched crate.
///
/// The current `Cargo.toml` is determined by taking `CARGO_MANIFEST_DIR/Cargo.toml`.
///
/// # Returns
///
/// - `Ok(orig_name)` if the crate was found, but not renamed in the `Cargo.toml`.
/// - `Ok(RENAMED)` if the crate was found, but is renamed in the `Cargo.toml`. `RENAMED` will be
/// the renamed name.
/// - `Err` if an error occurred.
///
/// The returned crate name is sanitized in such a way that it is a valid rust identifier. Thus,
/// it is ready to be used in `extern crate` as identifier.
pub fn crate_name(orig_name: &str) -> Result<String, String> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "Could not find `CARGO_MANIFEST_DIR` env variable.")?;

    let cargo_toml_path = PathBuf::from(manifest_dir).join("Cargo.toml");

    if !cargo_toml_path.exists() {
        return Err(format!("`{}` does not exist.", cargo_toml_path.display()));
    }

    let cargo_toml = open_cargo_toml(&cargo_toml_path)?;

    extract_crate_name(orig_name, cargo_toml, &cargo_toml_path).map(sanitize_crate_name)
}

/// Make sure that the given crate name is a valid rust identifier.
fn sanitize_crate_name(name: String) -> String {
    name.replace("-", "_")
}

/// Open the given `Cargo.toml` and parse it into a hashmap.
fn open_cargo_toml(path: &Path) -> Result<CargoToml, String> {
    let mut content = String::new();
    File::open(path)
        .map_err(|e| format!("Could not open `{}`: {:?}", path.display(), e))?
        .read_to_string(&mut content)
        .map_err(|e| format!("Could not read `{}` to string: {:?}", path.display(), e))?;
    toml::from_str(&content).map_err(|e| format!("{:?}", e))
}

/// Create the not found error.
fn create_not_found_err(orig_name: &str, path: &dyn Display) -> Result<String, String> {
    Err(format!(
        "Could not find `{}` in `dependencies` or `dev-dependencies` in `{}`!",
        orig_name, path
    ))
}

/// Extract the crate name for the given `orig_name` from the given `Cargo.toml` by checking the
/// `dependencies` and `dev-dependencies`.
///
/// Returns `Ok(orig_name)` if the crate is not renamed in the `Cargo.toml` or otherwise
/// the renamed identifier.
fn extract_crate_name(
    orig_name: &str,
    mut cargo_toml: CargoToml,
    cargo_toml_path: &Path,
) -> Result<String, String> {
    if let Some(name) = ["dependencies", "dev-dependencies"]
        .iter()
        .find_map(|k| search_crate_at_key(k, orig_name, &mut cargo_toml))
    {
        return Ok(name);
    }

    // Start searching `target.xy.dependencies`
    if let Some(name) = cargo_toml
        .remove("target")
        .and_then(|t| t.try_into::<Table>().ok())
        .and_then(|t| {
            t.values()
                .filter_map(|v| v.as_table())
                .filter_map(|t| t.get("dependencies").and_then(|t| t.as_table()))
                .find_map(|t| extract_crate_name_from_deps(orig_name, t.clone()))
        })
    {
        return Ok(name);
    }

    create_not_found_err(orig_name, &cargo_toml_path.display())
}

/// Search the `orig_name` crate at the given `key` in `cargo_toml`.
fn search_crate_at_key(key: &str, orig_name: &str, cargo_toml: &mut CargoToml) -> Option<String> {
    cargo_toml
        .remove(key)
        .and_then(|v| v.try_into::<Table>().ok())
        .and_then(|t| extract_crate_name_from_deps(orig_name, t))
}

/// Extract the crate name from the given dependencies.
///
/// Returns `Some(orig_name)` if the crate is not renamed in the `Cargo.toml` or otherwise
/// the renamed identifier.
fn extract_crate_name_from_deps(orig_name: &str, deps: Table) -> Option<String> {
    for (key, value) in deps.into_iter() {
        let renamed = value
            .try_into::<Table>()
            .ok()
            .and_then(|t| t.get("package").cloned())
            .map(|t| t.as_str() == Some(orig_name))
            .unwrap_or(false);

        if key == orig_name || renamed {
            return Some(key.clone());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! create_test {
        (
            $name:ident,
            $cargo_toml:expr,
            $result:expr,
        ) => {
            #[test]
            fn $name() {
                let cargo_toml = toml::from_str($cargo_toml).expect("Parses `Cargo.toml`");
                let path = PathBuf::from("test-path");

                assert_eq!($result, extract_crate_name("my_crate", cargo_toml, &path));
            }
        };
    }

    create_test! {
        deps_with_crate,
        r#"
            [dependencies]
            my_crate = "0.1"
        "#,
        Ok("my_crate".into()),
    }

    create_test! {
        dev_deps_with_crate,
        r#"
            [dev-dependencies]
            my_crate = "0.1"
        "#,
        Ok("my_crate".into()),
    }

    create_test! {
        deps_with_crate_renamed,
        r#"
            [dependencies]
            cool = { package = "my_crate", version = "0.1" }
        "#,
        Ok("cool".into()),
    }

    create_test! {
        deps_with_crate_renamed_second,
        r#"
            [dependencies.cool]
            package = "my_crate"
            version = "0.1"
        "#,
        Ok("cool".into()),
    }

    create_test! {
        deps_empty,
        r#"
            [dependencies]
        "#,
        create_not_found_err("my_crate", &"test-path"),
    }

    create_test! {
        crate_not_found,
        r#"
            [dependencies]
            serde = "1.0"
        "#,
        create_not_found_err("my_crate", &"test-path"),
    }

    create_test! {
        target_dependency,
        r#"
            [target.'cfg(target_os="android")'.dependencies]
            my_crate = "0.1"
        "#,
        Ok("my_crate".into()),
    }

    create_test! {
        target_dependency2,
        r#"
            [target.x86_64-pc-windows-gnu.dependencies]
            my_crate = "0.1"
        "#,
        Ok("my_crate".into()),
    }
}
