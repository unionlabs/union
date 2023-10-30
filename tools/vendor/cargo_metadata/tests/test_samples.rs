extern crate cargo_metadata;
extern crate semver;
#[macro_use]
extern crate serde_json;

use camino::Utf8PathBuf;
use cargo_metadata::{CargoOpt, DependencyKind, Metadata, MetadataCommand};

#[test]
fn old_minimal() {
    // Output from oldest supported version (1.24).
    // This intentionally has as many null fields as possible.
    // 1.8 is when metadata was introduced.
    // Older versions not supported because the following are required:
    // - `workspace_members` added in 1.13
    // - `target_directory` added in 1.19
    // - `workspace_root` added in 1.24
    let json = r#"
{
  "packages": [
    {
      "name": "foo",
      "version": "0.1.0",
      "id": "foo 0.1.0 (path+file:///foo)",
      "license": null,
      "license_file": null,
      "description": null,
      "source": null,
      "dependencies": [
        {
          "name": "somedep",
          "source": null,
          "req": "^1.0",
          "kind": null,
          "optional": false,
          "uses_default_features": true,
          "features": [],
          "target": null
        }
      ],
      "targets": [
        {
          "kind": [
            "bin"
          ],
          "crate_types": [
            "bin"
          ],
          "name": "foo",
          "src_path": "/foo/src/main.rs"
        }
      ],
      "features": {},
      "manifest_path": "/foo/Cargo.toml"
    }
  ],
  "workspace_members": [
    "foo 0.1.0 (path+file:///foo)"
  ],
  "resolve": null,
  "target_directory": "/foo/target",
  "version": 1,
  "workspace_root": "/foo"
}
"#;
    let meta: Metadata = serde_json::from_str(json).unwrap();
    assert_eq!(meta.packages.len(), 1);
    let pkg = &meta.packages[0];
    assert_eq!(pkg.name, "foo");
    assert_eq!(pkg.version, semver::Version::parse("0.1.0").unwrap());
    assert_eq!(pkg.authors.len(), 0);
    assert_eq!(pkg.id.to_string(), "foo 0.1.0 (path+file:///foo)");
    assert_eq!(pkg.description, None);
    assert_eq!(pkg.license, None);
    assert_eq!(pkg.license_file, None);
    assert_eq!(pkg.default_run, None);
    assert_eq!(pkg.rust_version, None);
    assert_eq!(pkg.dependencies.len(), 1);
    let dep = &pkg.dependencies[0];
    assert_eq!(dep.name, "somedep");
    assert_eq!(dep.source, None);
    assert_eq!(dep.req, semver::VersionReq::parse("^1.0").unwrap());
    assert_eq!(dep.kind, DependencyKind::Normal);
    assert_eq!(dep.optional, false);
    assert_eq!(dep.uses_default_features, true);
    assert_eq!(dep.features.len(), 0);
    assert!(dep.target.is_none());
    assert_eq!(dep.rename, None);
    assert_eq!(dep.registry, None);
    assert_eq!(pkg.targets.len(), 1);
    let target = &pkg.targets[0];
    assert_eq!(target.name, "foo");
    assert_eq!(target.kind, vec!["bin"]);
    assert_eq!(target.crate_types, vec!["bin"]);
    assert_eq!(target.required_features.len(), 0);
    assert_eq!(target.src_path, "/foo/src/main.rs");
    assert_eq!(target.edition, "2015");
    assert_eq!(target.doctest, true);
    assert_eq!(target.test, true);
    assert_eq!(target.doc, true);
    assert_eq!(pkg.features.len(), 0);
    assert_eq!(pkg.manifest_path, "/foo/Cargo.toml");
    assert_eq!(pkg.categories.len(), 0);
    assert_eq!(pkg.keywords.len(), 0);
    assert_eq!(pkg.readme, None);
    assert_eq!(pkg.repository, None);
    assert_eq!(pkg.homepage, None);
    assert_eq!(pkg.documentation, None);
    assert_eq!(pkg.edition, "2015");
    assert_eq!(pkg.metadata, serde_json::Value::Null);
    assert_eq!(pkg.links, None);
    assert_eq!(pkg.publish, None);
    assert_eq!(meta.workspace_members.len(), 1);
    assert_eq!(
        meta.workspace_members[0].to_string(),
        "foo 0.1.0 (path+file:///foo)"
    );
    assert!(meta.resolve.is_none());
    assert_eq!(meta.workspace_root, "/foo");
    assert_eq!(meta.workspace_metadata, serde_json::Value::Null);
    assert_eq!(meta.target_directory, "/foo/target");
}

macro_rules! sorted {
    ($e:expr) => {{
        let mut v = $e.clone();
        v.sort();
        v
    }};
}

fn cargo_version() -> semver::Version {
    let output = std::process::Command::new("cargo")
        .arg("-V")
        .output()
        .expect("Failed to exec cargo.");
    let out = std::str::from_utf8(&output.stdout)
        .expect("invalid utf8")
        .trim();
    let split: Vec<&str> = out.split_whitespace().collect();
    assert!(split.len() >= 2, "cargo -V output is unexpected: {}", out);
    let mut ver = semver::Version::parse(split[1]).expect("cargo -V semver could not be parsed");
    // Don't care about metadata, it is awkward to compare.
    ver.pre = semver::Prerelease::EMPTY;
    ver.build = semver::BuildMetadata::EMPTY;
    ver
}

#[derive(serde::Deserialize, PartialEq, Eq, Debug)]
struct WorkspaceMetadata {
    testobject: TestObject,
}

#[derive(serde::Deserialize, PartialEq, Eq, Debug)]
struct TestObject {
    myvalue: String,
}

#[test]
fn all_the_fields() {
    // All the fields currently generated as of 1.60. This tries to exercise as
    // much as possible.
    let ver = cargo_version();
    let minimum = semver::Version::parse("1.56.0").unwrap();
    if ver < minimum {
        // edition added in 1.30
        // rename added in 1.31
        // links added in 1.33
        // doctest added in 1.37
        // publish added in 1.39
        // dep_kinds added in 1.41
        // test added in 1.47
        // homepage added in 1.49
        // documentation added in 1.49
        // doc added in 1.50
        // path added in 1.51
        // default_run added in 1.55
        // rust_version added in 1.58
        eprintln!("Skipping all_the_fields test, cargo {} is too old.", ver);
        return;
    }
    let meta = MetadataCommand::new()
        .manifest_path("tests/all/Cargo.toml")
        .exec()
        .unwrap();
    assert_eq!(meta.workspace_root.file_name().unwrap(), "all");
    assert_eq!(
        serde_json::from_value::<WorkspaceMetadata>(meta.workspace_metadata).unwrap(),
        WorkspaceMetadata {
            testobject: TestObject {
                myvalue: "abc".to_string()
            }
        }
    );
    assert_eq!(meta.workspace_members.len(), 1);
    assert!(meta.workspace_members[0].to_string().starts_with("all"));

    assert_eq!(meta.packages.len(), 9);
    let all = meta.packages.iter().find(|p| p.name == "all").unwrap();
    assert_eq!(all.version, semver::Version::parse("0.1.0").unwrap());
    assert_eq!(all.authors, vec!["Jane Doe <user@example.com>"]);
    assert!(all.id.to_string().starts_with("all"));
    assert_eq!(all.description, Some("Package description.".to_string()));
    assert_eq!(all.license, Some("MIT/Apache-2.0".to_string()));
    assert_eq!(all.license_file, Some(Utf8PathBuf::from("LICENSE")));
    assert!(all.license_file().unwrap().ends_with("tests/all/LICENSE"));
    assert_eq!(all.publish, Some(vec![]));
    assert_eq!(all.links, Some("foo".to_string()));
    assert_eq!(all.default_run, Some("otherbin".to_string()));
    if ver >= semver::Version::parse("1.58.0").unwrap() {
        assert_eq!(
            all.rust_version,
            Some(semver::VersionReq::parse("1.56").unwrap())
        );
    }

    assert_eq!(all.dependencies.len(), 8);
    let bitflags = all
        .dependencies
        .iter()
        .find(|d| d.name == "bitflags")
        .unwrap();
    assert_eq!(
        bitflags.source,
        Some("registry+https://github.com/rust-lang/crates.io-index".to_string())
    );
    assert_eq!(bitflags.optional, true);
    assert_eq!(bitflags.req, semver::VersionReq::parse("^1.0").unwrap());

    let path_dep = all
        .dependencies
        .iter()
        .find(|d| d.name == "path-dep")
        .unwrap();
    assert_eq!(path_dep.source, None);
    assert_eq!(path_dep.kind, DependencyKind::Normal);
    assert_eq!(path_dep.req, semver::VersionReq::parse("*").unwrap());
    assert_eq!(
        path_dep.path.as_ref().map(|p| p.ends_with("path-dep")),
        Some(true),
    );

    all.dependencies
        .iter()
        .find(|d| d.name == "namedep")
        .unwrap();

    let featdep = all
        .dependencies
        .iter()
        .find(|d| d.name == "featdep")
        .unwrap();
    assert_eq!(featdep.features, vec!["i128"]);
    assert_eq!(featdep.uses_default_features, false);

    let renamed = all
        .dependencies
        .iter()
        .find(|d| d.name == "oldname")
        .unwrap();
    assert_eq!(renamed.rename, Some("newname".to_string()));

    let devdep = all
        .dependencies
        .iter()
        .find(|d| d.name == "devdep")
        .unwrap();
    assert_eq!(devdep.kind, DependencyKind::Development);

    let bdep = all.dependencies.iter().find(|d| d.name == "bdep").unwrap();
    assert_eq!(bdep.kind, DependencyKind::Build);

    let windep = all
        .dependencies
        .iter()
        .find(|d| d.name == "windep")
        .unwrap();
    assert_eq!(
        windep.target.as_ref().map(|x| x.to_string()),
        Some("cfg(windows)".to_string())
    );

    macro_rules! get_file_name {
        ($v:expr) => {
            all.targets
                .iter()
                .find(|t| t.src_path.file_name().unwrap() == $v)
                .unwrap()
        };
    }
    assert_eq!(all.targets.len(), 8);
    let lib = get_file_name!("lib.rs");
    assert_eq!(lib.name, "all");
    assert_eq!(sorted!(lib.kind), vec!["cdylib", "rlib", "staticlib"]);
    assert_eq!(
        sorted!(lib.crate_types),
        vec!["cdylib", "rlib", "staticlib"]
    );
    assert_eq!(lib.required_features.len(), 0);
    assert_eq!(lib.edition, "2018");
    assert_eq!(lib.doctest, true);
    assert_eq!(lib.test, true);
    assert_eq!(lib.doc, true);

    let main = get_file_name!("main.rs");
    assert_eq!(main.crate_types, vec!["bin"]);
    assert_eq!(main.kind, vec!["bin"]);
    assert_eq!(main.doctest, false);
    assert_eq!(main.test, true);
    assert_eq!(main.doc, true);

    let otherbin = get_file_name!("otherbin.rs");
    assert_eq!(otherbin.edition, "2015");
    assert_eq!(otherbin.doc, false);

    let reqfeat = get_file_name!("reqfeat.rs");
    assert_eq!(reqfeat.required_features, vec!["feat2"]);

    let ex1 = get_file_name!("ex1.rs");
    assert_eq!(ex1.kind, vec!["example"]);
    assert_eq!(ex1.test, false);

    let t1 = get_file_name!("t1.rs");
    assert_eq!(t1.kind, vec!["test"]);

    let b1 = get_file_name!("b1.rs");
    assert_eq!(b1.kind, vec!["bench"]);

    let build = get_file_name!("build.rs");
    assert_eq!(build.kind, vec!["custom-build"]);

    if ver >= semver::Version::parse("1.60.0").unwrap() {
        // 1.60 now reports optional dependencies within the features table
        assert_eq!(all.features.len(), 4);
        assert_eq!(all.features["bitflags"], vec!["dep:bitflags"]);
    } else {
        assert_eq!(all.features.len(), 3);
    }
    assert_eq!(all.features["feat1"].len(), 0);
    assert_eq!(all.features["feat2"].len(), 0);
    assert_eq!(sorted!(all.features["default"]), vec!["bitflags", "feat1"]);

    assert!(all.manifest_path.ends_with("all/Cargo.toml"));
    assert_eq!(all.categories, vec!["command-line-utilities"]);
    assert_eq!(all.keywords, vec!["cli"]);
    assert_eq!(all.readme, Some(Utf8PathBuf::from("README.md")));
    assert_eq!(
        all.repository,
        Some("https://github.com/oli-obk/cargo_metadata/".to_string())
    );
    assert_eq!(
        all.homepage,
        Some("https://github.com/oli-obk/cargo_metadata/".to_string())
    );
    assert_eq!(
        all.documentation,
        Some("https://docs.rs/cargo_metadata/".to_string())
    );
    assert_eq!(all.edition, "2018");
    assert_eq!(
        all.metadata,
        json!({
            "docs": {
                "rs": {
                    "all-features": true,
                    "default-target": "x86_64-unknown-linux-gnu",
                    "rustc-args": ["--example-rustc-arg"]
                }
            }
        })
    );

    let resolve = meta.resolve.as_ref().unwrap();
    assert!(resolve
        .root
        .as_ref()
        .unwrap()
        .to_string()
        .starts_with("all"));

    assert_eq!(resolve.nodes.len(), 9);
    let path_dep = resolve
        .nodes
        .iter()
        .find(|n| n.id.to_string().starts_with("path-dep"))
        .unwrap();
    assert_eq!(path_dep.deps.len(), 0);
    assert_eq!(path_dep.dependencies.len(), 0);
    assert_eq!(path_dep.features.len(), 0);

    let bitflags = resolve
        .nodes
        .iter()
        .find(|n| n.id.to_string().starts_with("bitflags"))
        .unwrap();
    assert_eq!(bitflags.features, vec!["default"]);

    let featdep = resolve
        .nodes
        .iter()
        .find(|n| n.id.to_string().starts_with("featdep"))
        .unwrap();
    assert_eq!(featdep.features, vec!["i128"]);

    let all = resolve
        .nodes
        .iter()
        .find(|n| n.id.to_string().starts_with("all"))
        .unwrap();
    assert_eq!(all.dependencies.len(), 8);
    assert_eq!(all.deps.len(), 8);
    let newname = all.deps.iter().find(|d| d.name == "newname").unwrap();
    assert!(newname.pkg.to_string().starts_with("oldname"));
    // Note the underscore here.
    let path_dep = all.deps.iter().find(|d| d.name == "path_dep").unwrap();
    assert!(path_dep.pkg.to_string().starts_with("path-dep"));
    assert_eq!(path_dep.dep_kinds.len(), 1);
    let kind = &path_dep.dep_kinds[0];
    assert_eq!(kind.kind, DependencyKind::Normal);
    assert!(kind.target.is_none());

    let namedep = all
        .deps
        .iter()
        .find(|d| d.name == "different_name")
        .unwrap();
    assert!(namedep.pkg.to_string().starts_with("namedep"));
    assert_eq!(sorted!(all.features), vec!["bitflags", "default", "feat1"]);

    let bdep = all.deps.iter().find(|d| d.name == "bdep").unwrap();
    assert_eq!(bdep.dep_kinds.len(), 1);
    let kind = &bdep.dep_kinds[0];
    assert_eq!(kind.kind, DependencyKind::Build);
    assert!(kind.target.is_none());

    let devdep = all.deps.iter().find(|d| d.name == "devdep").unwrap();
    assert_eq!(devdep.dep_kinds.len(), 1);
    let kind = &devdep.dep_kinds[0];
    assert_eq!(kind.kind, DependencyKind::Development);
    assert!(kind.target.is_none());

    let windep = all.deps.iter().find(|d| d.name == "windep").unwrap();
    assert_eq!(windep.dep_kinds.len(), 1);
    let kind = &windep.dep_kinds[0];
    assert_eq!(kind.kind, DependencyKind::Normal);
    assert_eq!(
        kind.target.as_ref().map(|x| x.to_string()),
        Some("cfg(windows)".to_string())
    );
}

#[test]
fn alt_registry() {
    // This is difficult to test (would need to set up a custom index).
    // Just manually check the JSON is handled.
    let json = r#"
{
  "packages": [
    {
      "name": "alt",
      "version": "0.1.0",
      "id": "alt 0.1.0 (path+file:///alt)",
      "source": null,
      "dependencies": [
        {
          "name": "alt2",
          "source": "registry+https://example.com",
          "req": "^0.1",
          "kind": null,
          "rename": null,
          "optional": false,
          "uses_default_features": true,
          "features": [],
          "target": null,
          "registry": "https://example.com"
        }
      ],
      "targets": [
        {
          "kind": [
            "lib"
          ],
          "crate_types": [
            "lib"
          ],
          "name": "alt",
          "src_path": "/alt/src/lib.rs",
          "edition": "2018"
        }
      ],
      "features": {},
      "manifest_path": "/alt/Cargo.toml",
      "metadata": null,
      "authors": [],
      "categories": [],
      "keywords": [],
      "readme": null,
      "repository": null,
      "edition": "2018",
      "links": null
    }
  ],
  "workspace_members": [
    "alt 0.1.0 (path+file:///alt)"
  ],
  "resolve": null,
  "target_directory": "/alt/target",
  "version": 1,
  "workspace_root": "/alt"
}
"#;
    let meta: Metadata = serde_json::from_str(json).unwrap();
    assert_eq!(meta.packages.len(), 1);
    let alt = &meta.packages[0];
    let deps = &alt.dependencies;
    assert_eq!(deps.len(), 1);
    let dep = &deps[0];
    assert_eq!(dep.registry, Some("https://example.com".to_string()));
}

#[test]
fn current_dir() {
    let meta = MetadataCommand::new()
        .current_dir("tests/all/namedep")
        .exec()
        .unwrap();
    let namedep = meta.packages.iter().find(|p| p.name == "namedep").unwrap();
    assert!(namedep.name.starts_with("namedep"));
}

#[test]
fn parse_stream_is_robust() {
    // Proc macros can print stuff to stdout, which naturally breaks JSON messages.
    // Let's check that we don't die horribly in this case, and report an error.
    let json_output = r##"{"reason":"compiler-artifact","package_id":"chatty 0.1.0 (path+file:///chatty-macro/chatty)","target":{"kind":["proc-macro"],"crate_types":["proc-macro"],"name":"chatty","src_path":"/chatty-macro/chatty/src/lib.rs","edition":"2018","doctest":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/chatty-macro/target/debug/deps/libchatty-f2adcff24cdf3bb2.so"],"executable":null,"fresh":false}
Evil proc macro was here!
{"reason":"compiler-artifact","package_id":"chatty-macro 0.1.0 (path+file:///chatty-macro)","target":{"kind":["lib"],"crate_types":["lib"],"name":"chatty-macro","src_path":"/chatty-macro/src/lib.rs","edition":"2018","doctest":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":[],"filenames":["/chatty-macro/target/debug/libchatty_macro.rlib","/chatty-macro/target/debug/deps/libchatty_macro-cb5956ed52a11fb6.rmeta"],"executable":null,"fresh":false}
"##;
    let mut n_messages = 0;
    let mut text = String::new();
    for message in cargo_metadata::Message::parse_stream(json_output.as_bytes()) {
        let message = message.unwrap();
        match message {
            cargo_metadata::Message::TextLine(line) => text = line,
            _ => n_messages += 1,
        }
    }
    assert_eq!(n_messages, 2);
    assert_eq!(text, "Evil proc macro was here!");
}

#[test]
fn advanced_feature_configuration() {
    fn build_features<F: FnOnce(&mut MetadataCommand) -> &mut MetadataCommand>(
        func: F,
    ) -> Vec<String> {
        let mut meta = MetadataCommand::new();
        let meta = meta.manifest_path("tests/all/Cargo.toml");

        let meta = func(meta);
        let meta = meta.exec().unwrap();

        let resolve = meta.resolve.as_ref().unwrap();

        let all = resolve
            .nodes
            .iter()
            .find(|n| n.id.to_string().starts_with("all"))
            .unwrap();

        all.features.clone()
    }

    // Default behavior; tested above
    let default_features = build_features(|meta| meta);
    assert_eq!(
        sorted!(default_features),
        vec!["bitflags", "default", "feat1"]
    );

    // Manually specify the same default features
    let manual_features = build_features(|meta| {
        meta.features(CargoOpt::NoDefaultFeatures)
            .features(CargoOpt::SomeFeatures(vec![
                "feat1".into(),
                "bitflags".into(),
            ]))
    });
    assert_eq!(sorted!(manual_features), vec!["bitflags", "feat1"]);

    // Multiple SomeFeatures is same as one longer SomeFeatures
    let manual_features = build_features(|meta| {
        meta.features(CargoOpt::NoDefaultFeatures)
            .features(CargoOpt::SomeFeatures(vec!["feat1".into()]))
            .features(CargoOpt::SomeFeatures(vec!["feat2".into()]))
    });
    assert_eq!(sorted!(manual_features), vec!["feat1", "feat2"]);

    // No features + All features == All features
    let all_features = build_features(|meta| {
        meta.features(CargoOpt::AllFeatures)
            .features(CargoOpt::NoDefaultFeatures)
    });
    assert_eq!(
        sorted!(all_features),
        vec!["bitflags", "default", "feat1", "feat2"]
    );

    // The '--all-features' flag supersedes other feature flags
    let all_flag_variants = build_features(|meta| {
        meta.features(CargoOpt::SomeFeatures(vec!["feat2".into()]))
            .features(CargoOpt::NoDefaultFeatures)
            .features(CargoOpt::AllFeatures)
    });
    assert_eq!(sorted!(all_flag_variants), sorted!(all_features));
}

#[test]
fn depkind_to_string() {
    assert_eq!(DependencyKind::Normal.to_string(), "normal");
    assert_eq!(DependencyKind::Development.to_string(), "dev");
    assert_eq!(DependencyKind::Build.to_string(), "build");
    assert_eq!(DependencyKind::Unknown.to_string(), "Unknown");
}
