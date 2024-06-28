use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::ExitCode,
};

use cargo_metadata::MetadataCommand;
use cargo_util_schemas::manifest::{InheritableDependency, TomlInheritedDependency, TomlManifest};
use clap::Parser;

#[derive(Parser)]
struct App {
    manifest_path: PathBuf,
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .init();

    let app = App::parse();

    let metadata = MetadataCommand::new()
        .no_deps()
        .manifest_path(&app.manifest_path)
        .exec()
        .unwrap();

    let root_cargo_toml = read_manifest(&app.manifest_path);

    let workspace_dependencies = root_cargo_toml.workspace.unwrap().dependencies.unwrap();

    let workspace_member_manifests = metadata
        .workspace_packages()
        .into_iter()
        .map(|member| (member, read_manifest(&member.manifest_path)))
        .collect::<Vec<_>>();

    let mut is_err = false;

    for (member, member_manifest) in &workspace_member_manifests {
        let Some(ref deps) = member_manifest.dependencies else {
            continue;
        };

        for (dep_name, dep) in deps {
            match dep {
                InheritableDependency::Value(_) => {
                    if dep_name.as_ref() == "cosmwasm-schema"
                        || dep_name.as_ref() == "cosmwasm-std"
                        || dep_name.as_ref() == "cw-storage-plus"
                        || dep_name.as_ref() == "axum"
                    {
                        tracing::warn!(
                            member = %member.name,
                            "{dep_name} is being ignored for deduplication checks as there are currently multiple incompatible versions being used in the repo"
                        );
                        continue;
                    }
                    if workspace_dependencies.contains_key(dep_name) {
                        is_err = true;

                        tracing::error!(
                            member = %member.name,
                            "`{dep_name}` exists in workspace.dependencies and should be used instead",
                        );
                    }
                }
                InheritableDependency::Inherit(inherit) => {
                    if matches!(
                        inherit,
                        TomlInheritedDependency {
                            default_features: Some(false),
                            ..
                        } | TomlInheritedDependency {
                            default_features2: Some(false),
                            ..
                        }
                    ) {
                        tracing::error!(
                            member = %member.name,
                            "specifying `default-features = false` for `{dep_name}` has no effect as it is a workspace dependency",
                        );
                    }
                }
            }
        }
    }

    workspace_member_manifests
        .iter()
        .flat_map(|(package, manifest)| {
            manifest
                .dependencies
                .iter()
                .flatten()
                .filter_map(|(name, dep)| match dep {
                    InheritableDependency::Value(_) => Some((&package.name, name)),
                    _ => None,
                })
        })
        .fold(BTreeMap::<_, Vec<_>>::new(), |mut acc, (package, dep)| {
            acc.entry(dep).or_default().push(&**package);
            acc
        })
        .into_iter()
        .for_each(|(dep, packages)| {
            if packages.len() >= 2 && !workspace_dependencies.contains_key(dep)  {
                tracing::info!(
                    "`{dep}` is used in multiple crates ({}), consider making it a workspace dependency", packages.join(", ")
                )
            }
        });

    if is_err {
        ExitCode::FAILURE
    } else {
        tracing::info!("no issues to report");
        ExitCode::SUCCESS
    }
}

fn read_manifest(path: impl AsRef<Path>) -> TomlManifest {
    std::fs::read_to_string(path)
        .as_deref()
        .map(toml::from_str)
        .unwrap()
        .unwrap()
}
