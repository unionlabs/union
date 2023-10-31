use crate::directory::Directory;
use crate::error::Error;
use crate::manifest::Edition;
use serde::de::value::MapAccessDeserializer;
use serde::de::{self, Visitor};
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap as Map;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use toml::Value;

pub fn get_manifest(manifest_dir: &Directory) -> Manifest {
    try_get_manifest(manifest_dir).unwrap_or_default()
}

fn try_get_manifest(manifest_dir: &Directory) -> Result<Manifest, Error> {
    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let manifest_str = fs::read_to_string(cargo_toml_path)?;
    let mut manifest: Manifest = toml::from_str(&manifest_str)?;

    fix_dependencies(&mut manifest.dependencies, manifest_dir);
    fix_dependencies(&mut manifest.dev_dependencies, manifest_dir);
    for target in manifest.target.values_mut() {
        fix_dependencies(&mut target.dependencies, manifest_dir);
        fix_dependencies(&mut target.dev_dependencies, manifest_dir);
    }

    Ok(manifest)
}

pub fn get_workspace_manifest(manifest_dir: &Directory) -> WorkspaceManifest {
    try_get_workspace_manifest(manifest_dir).unwrap_or_default()
}

pub fn try_get_workspace_manifest(manifest_dir: &Directory) -> Result<WorkspaceManifest, Error> {
    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let manifest_str = fs::read_to_string(cargo_toml_path)?;
    let mut manifest: WorkspaceManifest = toml::from_str(&manifest_str)?;

    fix_patches(&mut manifest.patch, manifest_dir);
    fix_replacements(&mut manifest.replace, manifest_dir);

    Ok(manifest)
}

fn fix_dependencies(dependencies: &mut Map<String, Dependency>, dir: &Directory) {
    dependencies.remove("trybuild");
    for dep in dependencies.values_mut() {
        dep.path = dep.path.as_ref().map(|path| Directory::new(dir.join(path)));
    }
}

fn fix_patches(patches: &mut Map<String, RegistryPatch>, dir: &Directory) {
    for registry in patches.values_mut() {
        registry.crates.remove("trybuild");
        for patch in registry.crates.values_mut() {
            patch.path = patch.path.as_ref().map(|path| dir.join(path));
        }
    }
}

fn fix_replacements(replacements: &mut Map<String, Patch>, dir: &Directory) {
    replacements.remove("trybuild");
    for replacement in replacements.values_mut() {
        replacement.path = replacement.path.as_ref().map(|path| dir.join(path));
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct WorkspaceManifest {
    #[serde(default)]
    pub patch: Map<String, RegistryPatch>,
    #[serde(default)]
    pub replace: Map<String, Patch>,
}

#[derive(Deserialize, Default, Debug)]
pub struct Manifest {
    #[serde(default)]
    pub package: Package,
    #[serde(default)]
    pub features: Map<String, Vec<String>>,
    #[serde(default)]
    pub dependencies: Map<String, Dependency>,
    #[serde(default, alias = "dev-dependencies")]
    pub dev_dependencies: Map<String, Dependency>,
    #[serde(default)]
    pub target: Map<String, TargetDependencies>,
}

#[derive(Deserialize, Default, Debug)]
pub struct Package {
    pub name: String,
    #[serde(default)]
    pub edition: Edition,
    pub resolver: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(remote = "Self")]
pub struct Dependency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Directory>,
    #[serde(
        rename = "default-features",
        default = "get_true",
        skip_serializing_if = "is_true"
    )]
    pub default_features: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TargetDependencies {
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub dependencies: Map<String, Dependency>,
    #[serde(
        default,
        alias = "dev-dependencies",
        skip_serializing_if = "Map::is_empty"
    )]
    pub dev_dependencies: Map<String, Dependency>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct RegistryPatch {
    crates: Map<String, Patch>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Patch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

fn get_true() -> bool {
    true
}

fn is_true(boolean: &bool) -> bool {
    *boolean
}

impl Serialize for Dependency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Dependency::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for Dependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DependencyVisitor;

        impl<'de> Visitor<'de> for DependencyVisitor {
            type Value = Dependency;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a version string like \"0.9.8\" or a \
                     dependency like { version = \"0.9.8\" }",
                )
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Dependency {
                    version: Some(s.to_owned()),
                    path: None,
                    default_features: true,
                    features: Vec::new(),
                    git: None,
                    branch: None,
                    tag: None,
                    rev: None,
                    rest: Map::new(),
                })
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                Dependency::deserialize(MapAccessDeserializer::new(map))
            }
        }

        deserializer.deserialize_any(DependencyVisitor)
    }
}
