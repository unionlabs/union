// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A helper module to probe the Windows Registry when looking for
//! windows-specific tools.

use std::process::Command;

use crate::Tool;
#[cfg(windows)]
use crate::ToolFamily;

#[cfg(windows)]
const MSVC_FAMILY: ToolFamily = ToolFamily::Msvc { clang_cl: false };

/// Attempts to find a tool within an MSVC installation using the Windows
/// registry as a point to search from.
///
/// The `target` argument is the target that the tool should work for (e.g.
/// compile or link for) and the `tool` argument is the tool to find (e.g.
/// `cl.exe` or `link.exe`).
///
/// This function will return `None` if the tool could not be found, or it will
/// return `Some(cmd)` which represents a command that's ready to execute the
/// tool with the appropriate environment variables set.
///
/// Note that this function always returns `None` for non-MSVC targets.
pub fn find(target: &str, tool: &str) -> Option<Command> {
    find_tool(target, tool).map(|c| c.to_command())
}

/// Similar to the `find` function above, this function will attempt the same
/// operation (finding a MSVC tool in a local install) but instead returns a
/// `Tool` which may be introspected.
#[cfg(not(windows))]
pub fn find_tool(_target: &str, _tool: &str) -> Option<Tool> {
    None
}

/// Documented above.
#[cfg(windows)]
pub fn find_tool(target: &str, tool: &str) -> Option<Tool> {
    // This logic is all tailored for MSVC, if we're not that then bail out
    // early.
    if !target.contains("msvc") {
        return None;
    }

    // Looks like msbuild isn't located in the same location as other tools like
    // cl.exe and lib.exe. To handle this we probe for it manually with
    // dedicated registry keys.
    if tool.contains("msbuild") {
        return impl_::find_msbuild(target);
    }

    if tool.contains("devenv") {
        return impl_::find_devenv(target);
    }

    // Ok, if we're here, now comes the fun part of the probing. Default shells
    // or shells like MSYS aren't really configured to execute `cl.exe` and the
    // various compiler tools shipped as part of Visual Studio. Here we try to
    // first find the relevant tool, then we also have to be sure to fill in
    // environment variables like `LIB`, `INCLUDE`, and `PATH` to ensure that
    // the tool is actually usable.

    return impl_::find_msvc_environment(tool, target)
        .or_else(|| impl_::find_msvc_15plus(tool, target))
        .or_else(|| impl_::find_msvc_14(tool, target))
        .or_else(|| impl_::find_msvc_12(tool, target))
        .or_else(|| impl_::find_msvc_11(tool, target));
}

/// A version of Visual Studio
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum VsVers {
    /// Visual Studio 12 (2013)
    Vs12,
    /// Visual Studio 14 (2015)
    Vs14,
    /// Visual Studio 15 (2017)
    Vs15,
    /// Visual Studio 16 (2019)
    Vs16,
    /// Visual Studio 17 (2022)
    Vs17,

    /// Hidden variant that should not be matched on. Callers that want to
    /// handle an enumeration of `VsVers` instances should always have a default
    /// case meaning that it's a VS version they don't understand.
    #[doc(hidden)]
    #[allow(bad_style)]
    __Nonexhaustive_do_not_match_this_or_your_code_will_break,
}

/// Find the most recent installed version of Visual Studio
///
/// This is used by the cmake crate to figure out the correct
/// generator.
#[cfg(not(windows))]
pub fn find_vs_version() -> Result<VsVers, String> {
    Err(format!("not windows"))
}

/// Documented above
#[cfg(windows)]
pub fn find_vs_version() -> Result<VsVers, String> {
    use std::env;

    match env::var("VisualStudioVersion") {
        Ok(version) => match &version[..] {
            "17.0" => Ok(VsVers::Vs17),
            "16.0" => Ok(VsVers::Vs16),
            "15.0" => Ok(VsVers::Vs15),
            "14.0" => Ok(VsVers::Vs14),
            "12.0" => Ok(VsVers::Vs12),
            vers => Err(format!(
                "\n\n\
                 unsupported or unknown VisualStudio version: {}\n\
                 if another version is installed consider running \
                 the appropriate vcvars script before building this \
                 crate\n\
                 ",
                vers
            )),
        },
        _ => {
            // Check for the presence of a specific registry key
            // that indicates visual studio is installed.
            if impl_::has_msbuild_version("17.0") {
                Ok(VsVers::Vs17)
            } else if impl_::has_msbuild_version("16.0") {
                Ok(VsVers::Vs16)
            } else if impl_::has_msbuild_version("15.0") {
                Ok(VsVers::Vs15)
            } else if impl_::has_msbuild_version("14.0") {
                Ok(VsVers::Vs14)
            } else if impl_::has_msbuild_version("12.0") {
                Ok(VsVers::Vs12)
            } else {
                Err(format!(
                    "\n\n\
                     couldn't determine visual studio generator\n\
                     if VisualStudio is installed, however, consider \
                     running the appropriate vcvars script before building \
                     this crate\n\
                     "
                ))
            }
        }
    }
}

#[cfg(windows)]
mod impl_ {
    use crate::com;
    use crate::registry::{RegistryKey, LOCAL_MACHINE};
    use crate::setup_config::SetupConfiguration;
    use crate::vs_instances::{VsInstances, VswhereInstance};
    use std::convert::TryFrom;
    use std::env;
    use std::ffi::OsString;
    use std::fs::File;
    use std::io::Read;
    use std::iter;
    use std::mem;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::str::FromStr;

    use super::MSVC_FAMILY;
    use crate::Tool;

    struct MsvcTool {
        tool: PathBuf,
        libs: Vec<PathBuf>,
        path: Vec<PathBuf>,
        include: Vec<PathBuf>,
    }

    impl MsvcTool {
        fn new(tool: PathBuf) -> MsvcTool {
            MsvcTool {
                tool: tool,
                libs: Vec::new(),
                path: Vec::new(),
                include: Vec::new(),
            }
        }

        fn into_tool(self) -> Tool {
            let MsvcTool {
                tool,
                libs,
                path,
                include,
            } = self;
            let mut tool = Tool::with_family(tool.into(), MSVC_FAMILY);
            add_env(&mut tool, "LIB", libs);
            add_env(&mut tool, "PATH", path);
            add_env(&mut tool, "INCLUDE", include);
            tool
        }
    }

    /// Checks to see if the `VSCMD_ARG_TGT_ARCH` environment variable matches the
    /// given target's arch. Returns `None` if the variable does not exist.
    #[cfg(windows)]
    fn is_vscmd_target(target: &str) -> Option<bool> {
        let vscmd_arch = env::var("VSCMD_ARG_TGT_ARCH").ok()?;
        // Convert the Rust target arch to its VS arch equivalent.
        let arch = match target.split("-").next() {
            Some("x86_64") => "x64",
            Some("aarch64") => "arm64",
            Some("i686") | Some("i586") => "x86",
            Some("thumbv7a") => "arm",
            // An unrecognized arch.
            _ => return Some(false),
        };
        Some(vscmd_arch == arch)
    }

    /// Attempt to find the tool using environment variables set by vcvars.
    pub fn find_msvc_environment(tool: &str, target: &str) -> Option<Tool> {
        // Early return if the environment doesn't contain a VC install.
        if env::var_os("VCINSTALLDIR").is_none() {
            return None;
        }
        let vs_install_dir = env::var_os("VSINSTALLDIR")?.into();

        // If the vscmd target differs from the requested target then
        // attempt to get the tool using the VS install directory.
        if is_vscmd_target(target) == Some(false) {
            // We will only get here with versions 15+.
            tool_from_vs15plus_instance(tool, target, &vs_install_dir)
        } else {
            // Fallback to simply using the current environment.
            env::var_os("PATH")
                .and_then(|path| {
                    env::split_paths(&path)
                        .map(|p| p.join(tool))
                        .find(|p| p.exists())
                })
                .map(|path| Tool::with_family(path.into(), MSVC_FAMILY))
        }
    }

    fn find_msbuild_vs17(target: &str) -> Option<Tool> {
        find_tool_in_vs16plus_path(r"MSBuild\Current\Bin\MSBuild.exe", target, "17")
    }

    #[allow(bare_trait_objects)]
    fn vs16plus_instances(target: &str, version: &'static str) -> Box<Iterator<Item = PathBuf>> {
        let instances = if let Some(instances) = vs15plus_instances(target) {
            instances
        } else {
            return Box::new(iter::empty());
        };
        Box::new(instances.into_iter().filter_map(move |instance| {
            let installation_name = instance.installation_name()?;
            if installation_name.starts_with(&format!("VisualStudio/{}.", version)) {
                Some(instance.installation_path()?)
            } else if installation_name.starts_with(&format!("VisualStudioPreview/{}.", version)) {
                Some(instance.installation_path()?)
            } else {
                None
            }
        }))
    }

    fn find_tool_in_vs16plus_path(tool: &str, target: &str, version: &'static str) -> Option<Tool> {
        vs16plus_instances(target, version)
            .filter_map(|path| {
                let path = path.join(tool);
                if !path.is_file() {
                    return None;
                }
                let mut tool = Tool::with_family(path, MSVC_FAMILY);
                if target.contains("x86_64") {
                    tool.env.push(("Platform".into(), "X64".into()));
                }
                if target.contains("aarch64") {
                    tool.env.push(("Platform".into(), "ARM64".into()));
                }
                Some(tool)
            })
            .next()
    }

    fn find_msbuild_vs16(target: &str) -> Option<Tool> {
        find_tool_in_vs16plus_path(r"MSBuild\Current\Bin\MSBuild.exe", target, "16")
    }

    // In MSVC 15 (2017) MS once again changed the scheme for locating
    // the tooling.  Now we must go through some COM interfaces, which
    // is super fun for Rust.
    //
    // Note that much of this logic can be found [online] wrt paths, COM, etc.
    //
    // [online]: https://blogs.msdn.microsoft.com/vcblog/2017/03/06/finding-the-visual-c-compiler-tools-in-visual-studio-2017/
    //
    // Returns MSVC 15+ instances (15, 16 right now), the order should be consider undefined.
    //
    // However, on ARM64 this method doesn't work because VS Installer fails to register COM component on ARM64.
    // Hence, as the last resort we try to use vswhere.exe to list available instances.
    fn vs15plus_instances(target: &str) -> Option<VsInstances> {
        vs15plus_instances_using_com().or_else(|| vs15plus_instances_using_vswhere(target))
    }

    fn vs15plus_instances_using_com() -> Option<VsInstances> {
        com::initialize().ok()?;

        let config = SetupConfiguration::new().ok()?;
        let enum_setup_instances = config.enum_all_instances().ok()?;

        Some(VsInstances::ComBased(enum_setup_instances))
    }

    fn vs15plus_instances_using_vswhere(target: &str) -> Option<VsInstances> {
        let program_files_path: PathBuf = env::var("ProgramFiles(x86)")
            .or_else(|_| env::var("ProgramFiles"))
            .ok()?
            .into();

        let vswhere_path =
            program_files_path.join(r"Microsoft Visual Studio\Installer\vswhere.exe");

        if !vswhere_path.exists() {
            return None;
        }

        let arch = target.split('-').next().unwrap();
        let tools_arch = match arch {
            "i586" | "i686" | "x86_64" => Some("x86.x64"),
            "arm" | "thumbv7a" => Some("ARM"),
            "aarch64" => Some("ARM64"),
            _ => None,
        };

        let vswhere_output = Command::new(vswhere_path)
            .args(&[
                "-latest",
                "-products",
                "*",
                "-requires",
                &format!("Microsoft.VisualStudio.Component.VC.Tools.{}", tools_arch?),
                "-format",
                "text",
                "-nologo",
            ])
            .stderr(std::process::Stdio::inherit())
            .output()
            .ok()?;

        let vs_instances =
            VsInstances::VswhereBased(VswhereInstance::try_from(&vswhere_output.stdout).ok()?);

        Some(vs_instances)
    }

    // Inspired from official microsoft/vswhere ParseVersionString
    // i.e. at most four u16 numbers separated by '.'
    fn parse_version(version: &str) -> Option<Vec<u16>> {
        version
            .split('.')
            .map(|chunk| u16::from_str(chunk).ok())
            .collect()
    }

    pub fn find_msvc_15plus(tool: &str, target: &str) -> Option<Tool> {
        let iter = vs15plus_instances(target)?;
        iter.into_iter()
            .filter_map(|instance| {
                let version = parse_version(&instance.installation_version()?)?;
                let instance_path = instance.installation_path()?;
                let tool = tool_from_vs15plus_instance(tool, target, &instance_path)?;
                Some((version, tool))
            })
            .max_by(|(a_version, _), (b_version, _)| a_version.cmp(b_version))
            .map(|(_version, tool)| tool)
    }

    // While the paths to Visual Studio 2017's devenv and MSBuild could
    // potentially be retrieved from the registry, finding them via
    // SetupConfiguration has shown to be [more reliable], and is preferred
    // according to Microsoft. To help head off potential regressions though,
    // we keep the registry method as a fallback option.
    //
    // [more reliable]: https://github.com/alexcrichton/cc-rs/pull/331
    fn find_tool_in_vs15_path(tool: &str, target: &str) -> Option<Tool> {
        let mut path = match vs15plus_instances(target) {
            Some(instances) => instances
                .into_iter()
                .filter_map(|instance| instance.installation_path())
                .map(|path| path.join(tool))
                .find(|ref path| path.is_file()),
            None => None,
        };

        if path.is_none() {
            let key = r"SOFTWARE\WOW6432Node\Microsoft\VisualStudio\SxS\VS7";
            path = LOCAL_MACHINE
                .open(key.as_ref())
                .ok()
                .and_then(|key| key.query_str("15.0").ok())
                .map(|path| PathBuf::from(path).join(tool))
                .and_then(|path| if path.is_file() { Some(path) } else { None });
        }

        path.map(|path| {
            let mut tool = Tool::with_family(path, MSVC_FAMILY);
            if target.contains("x86_64") {
                tool.env.push(("Platform".into(), "X64".into()));
            }
            if target.contains("aarch64") {
                tool.env.push(("Platform".into(), "ARM64".into()));
            }
            tool
        })
    }

    fn tool_from_vs15plus_instance(
        tool: &str,
        target: &str,
        instance_path: &PathBuf,
    ) -> Option<Tool> {
        let (bin_path, host_dylib_path, lib_path, include_path) =
            vs15plus_vc_paths(target, instance_path)?;
        let tool_path = bin_path.join(tool);
        if !tool_path.exists() {
            return None;
        };

        let mut tool = MsvcTool::new(tool_path);
        tool.path.push(bin_path.clone());
        tool.path.push(host_dylib_path);
        tool.libs.push(lib_path);
        tool.include.push(include_path);

        if let Some((atl_lib_path, atl_include_path)) = atl_paths(target, &bin_path) {
            tool.libs.push(atl_lib_path);
            tool.include.push(atl_include_path);
        }

        add_sdks(&mut tool, target)?;

        Some(tool.into_tool())
    }

    fn vs15plus_vc_paths(
        target: &str,
        instance_path: &PathBuf,
    ) -> Option<(PathBuf, PathBuf, PathBuf, PathBuf)> {
        let version_path =
            instance_path.join(r"VC\Auxiliary\Build\Microsoft.VCToolsVersion.default.txt");
        let mut version_file = File::open(version_path).ok()?;
        let mut version = String::new();
        version_file.read_to_string(&mut version).ok()?;
        let version = version.trim();
        let host = match host_arch() {
            X86 => "X86",
            X86_64 => "X64",
            // There is no natively hosted compiler on ARM64.
            // Instead, use the x86 toolchain under emulation (there is no x64 emulation).
            AARCH64 => "X86",
            _ => return None,
        };
        let target = lib_subdir(target)?;
        // The directory layout here is MSVC/bin/Host$host/$target/
        let path = instance_path.join(r"VC\Tools\MSVC").join(version);
        // This is the path to the toolchain for a particular target, running
        // on a given host
        let bin_path = path
            .join("bin")
            .join(&format!("Host{}", host))
            .join(&target);
        // But! we also need PATH to contain the target directory for the host
        // architecture, because it contains dlls like mspdb140.dll compiled for
        // the host architecture.
        let host_dylib_path = path
            .join("bin")
            .join(&format!("Host{}", host))
            .join(&host.to_lowercase());
        let lib_path = path.join("lib").join(&target);
        let include_path = path.join("include");
        Some((bin_path, host_dylib_path, lib_path, include_path))
    }

    fn atl_paths(target: &str, path: &Path) -> Option<(PathBuf, PathBuf)> {
        let atl_path = path.join("atlmfc");
        let sub = lib_subdir(target)?;
        if atl_path.exists() {
            Some((atl_path.join("lib").join(sub), atl_path.join("include")))
        } else {
            None
        }
    }

    // For MSVC 14 we need to find the Universal CRT as well as either
    // the Windows 10 SDK or Windows 8.1 SDK.
    pub fn find_msvc_14(tool: &str, target: &str) -> Option<Tool> {
        let vcdir = get_vc_dir("14.0")?;
        let mut tool = get_tool(tool, &vcdir, target)?;
        add_sdks(&mut tool, target)?;
        Some(tool.into_tool())
    }

    fn add_sdks(tool: &mut MsvcTool, target: &str) -> Option<()> {
        let sub = lib_subdir(target)?;
        let (ucrt, ucrt_version) = get_ucrt_dir()?;

        let host = match host_arch() {
            X86 => "x86",
            X86_64 => "x64",
            AARCH64 => "arm64",
            _ => return None,
        };

        tool.path
            .push(ucrt.join("bin").join(&ucrt_version).join(host));

        let ucrt_include = ucrt.join("include").join(&ucrt_version);
        tool.include.push(ucrt_include.join("ucrt"));

        let ucrt_lib = ucrt.join("lib").join(&ucrt_version);
        tool.libs.push(ucrt_lib.join("ucrt").join(sub));

        if let Some((sdk, version)) = get_sdk10_dir() {
            tool.path.push(sdk.join("bin").join(host));
            let sdk_lib = sdk.join("lib").join(&version);
            tool.libs.push(sdk_lib.join("um").join(sub));
            let sdk_include = sdk.join("include").join(&version);
            tool.include.push(sdk_include.join("um"));
            tool.include.push(sdk_include.join("cppwinrt"));
            tool.include.push(sdk_include.join("winrt"));
            tool.include.push(sdk_include.join("shared"));
        } else if let Some(sdk) = get_sdk81_dir() {
            tool.path.push(sdk.join("bin").join(host));
            let sdk_lib = sdk.join("lib").join("winv6.3");
            tool.libs.push(sdk_lib.join("um").join(sub));
            let sdk_include = sdk.join("include");
            tool.include.push(sdk_include.join("um"));
            tool.include.push(sdk_include.join("winrt"));
            tool.include.push(sdk_include.join("shared"));
        }

        Some(())
    }

    // For MSVC 12 we need to find the Windows 8.1 SDK.
    pub fn find_msvc_12(tool: &str, target: &str) -> Option<Tool> {
        let vcdir = get_vc_dir("12.0")?;
        let mut tool = get_tool(tool, &vcdir, target)?;
        let sub = lib_subdir(target)?;
        let sdk81 = get_sdk81_dir()?;
        tool.path.push(sdk81.join("bin").join(sub));
        let sdk_lib = sdk81.join("lib").join("winv6.3");
        tool.libs.push(sdk_lib.join("um").join(sub));
        let sdk_include = sdk81.join("include");
        tool.include.push(sdk_include.join("shared"));
        tool.include.push(sdk_include.join("um"));
        tool.include.push(sdk_include.join("winrt"));
        Some(tool.into_tool())
    }

    // For MSVC 11 we need to find the Windows 8 SDK.
    pub fn find_msvc_11(tool: &str, target: &str) -> Option<Tool> {
        let vcdir = get_vc_dir("11.0")?;
        let mut tool = get_tool(tool, &vcdir, target)?;
        let sub = lib_subdir(target)?;
        let sdk8 = get_sdk8_dir()?;
        tool.path.push(sdk8.join("bin").join(sub));
        let sdk_lib = sdk8.join("lib").join("win8");
        tool.libs.push(sdk_lib.join("um").join(sub));
        let sdk_include = sdk8.join("include");
        tool.include.push(sdk_include.join("shared"));
        tool.include.push(sdk_include.join("um"));
        tool.include.push(sdk_include.join("winrt"));
        Some(tool.into_tool())
    }

    fn add_env(tool: &mut Tool, env: &str, paths: Vec<PathBuf>) {
        let prev = env::var_os(env).unwrap_or(OsString::new());
        let prev = env::split_paths(&prev);
        let new = paths.into_iter().chain(prev);
        tool.env
            .push((env.to_string().into(), env::join_paths(new).unwrap()));
    }

    // Given a possible MSVC installation directory, we look for the linker and
    // then add the MSVC library path.
    fn get_tool(tool: &str, path: &Path, target: &str) -> Option<MsvcTool> {
        bin_subdir(target)
            .into_iter()
            .map(|(sub, host)| {
                (
                    path.join("bin").join(sub).join(tool),
                    path.join("bin").join(host),
                )
            })
            .filter(|&(ref path, _)| path.is_file())
            .map(|(path, host)| {
                let mut tool = MsvcTool::new(path);
                tool.path.push(host);
                tool
            })
            .filter_map(|mut tool| {
                let sub = vc_lib_subdir(target)?;
                tool.libs.push(path.join("lib").join(sub));
                tool.include.push(path.join("include"));
                let atlmfc_path = path.join("atlmfc");
                if atlmfc_path.exists() {
                    tool.libs.push(atlmfc_path.join("lib").join(sub));
                    tool.include.push(atlmfc_path.join("include"));
                }
                Some(tool)
            })
            .next()
    }

    // To find MSVC we look in a specific registry key for the version we are
    // trying to find.
    fn get_vc_dir(ver: &str) -> Option<PathBuf> {
        let key = r"SOFTWARE\Microsoft\VisualStudio\SxS\VC7";
        let key = LOCAL_MACHINE.open(key.as_ref()).ok()?;
        let path = key.query_str(ver).ok()?;
        Some(path.into())
    }

    // To find the Universal CRT we look in a specific registry key for where
    // all the Universal CRTs are located and then sort them asciibetically to
    // find the newest version. While this sort of sorting isn't ideal,  it is
    // what vcvars does so that's good enough for us.
    //
    // Returns a pair of (root, version) for the ucrt dir if found
    fn get_ucrt_dir() -> Option<(PathBuf, String)> {
        let key = r"SOFTWARE\Microsoft\Windows Kits\Installed Roots";
        let key = LOCAL_MACHINE.open(key.as_ref()).ok()?;
        let root = key.query_str("KitsRoot10").ok()?;
        let readdir = Path::new(&root).join("lib").read_dir().ok()?;
        let max_libdir = readdir
            .filter_map(|dir| dir.ok())
            .map(|dir| dir.path())
            .filter(|dir| {
                dir.components()
                    .last()
                    .and_then(|c| c.as_os_str().to_str())
                    .map(|c| c.starts_with("10.") && dir.join("ucrt").is_dir())
                    .unwrap_or(false)
            })
            .max()?;
        let version = max_libdir.components().last().unwrap();
        let version = version.as_os_str().to_str().unwrap().to_string();
        Some((root.into(), version))
    }

    // Vcvars finds the correct version of the Windows 10 SDK by looking
    // for the include `um\Windows.h` because sometimes a given version will
    // only have UCRT bits without the rest of the SDK. Since we only care about
    // libraries and not includes, we instead look for `um\x64\kernel32.lib`.
    // Since the 32-bit and 64-bit libraries are always installed together we
    // only need to bother checking x64, making this code a tiny bit simpler.
    // Like we do for the Universal CRT, we sort the possibilities
    // asciibetically to find the newest one as that is what vcvars does.
    // Before doing that, we check the "WindowsSdkDir" and "WindowsSDKVersion"
    // environment variables set by vcvars to use the environment sdk version
    // if one is already configured.
    fn get_sdk10_dir() -> Option<(PathBuf, String)> {
        if let (Ok(root), Ok(version)) = (env::var("WindowsSdkDir"), env::var("WindowsSDKVersion"))
        {
            return Some((root.into(), version.trim_end_matches('\\').to_string()));
        }

        let key = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v10.0";
        let key = LOCAL_MACHINE.open(key.as_ref()).ok()?;
        let root = key.query_str("InstallationFolder").ok()?;
        let readdir = Path::new(&root).join("lib").read_dir().ok()?;
        let mut dirs = readdir
            .filter_map(|dir| dir.ok())
            .map(|dir| dir.path())
            .collect::<Vec<_>>();
        dirs.sort();
        let dir = dirs
            .into_iter()
            .rev()
            .filter(|dir| dir.join("um").join("x64").join("kernel32.lib").is_file())
            .next()?;
        let version = dir.components().last().unwrap();
        let version = version.as_os_str().to_str().unwrap().to_string();
        Some((root.into(), version))
    }

    // Interestingly there are several subdirectories, `win7` `win8` and
    // `winv6.3`. Vcvars seems to only care about `winv6.3` though, so the same
    // applies to us. Note that if we were targeting kernel mode drivers
    // instead of user mode applications, we would care.
    fn get_sdk81_dir() -> Option<PathBuf> {
        let key = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v8.1";
        let key = LOCAL_MACHINE.open(key.as_ref()).ok()?;
        let root = key.query_str("InstallationFolder").ok()?;
        Some(root.into())
    }

    fn get_sdk8_dir() -> Option<PathBuf> {
        let key = r"SOFTWARE\Microsoft\Microsoft SDKs\Windows\v8.0";
        let key = LOCAL_MACHINE.open(key.as_ref()).ok()?;
        let root = key.query_str("InstallationFolder").ok()?;
        Some(root.into())
    }

    const PROCESSOR_ARCHITECTURE_INTEL: u16 = 0;
    const PROCESSOR_ARCHITECTURE_AMD64: u16 = 9;
    const PROCESSOR_ARCHITECTURE_ARM64: u16 = 12;
    const X86: u16 = PROCESSOR_ARCHITECTURE_INTEL;
    const X86_64: u16 = PROCESSOR_ARCHITECTURE_AMD64;
    const AARCH64: u16 = PROCESSOR_ARCHITECTURE_ARM64;

    // When choosing the tool to use, we have to choose the one which matches
    // the target architecture. Otherwise we end up in situations where someone
    // on 32-bit Windows is trying to cross compile to 64-bit and it tries to
    // invoke the native 64-bit compiler which won't work.
    //
    // For the return value of this function, the first member of the tuple is
    // the folder of the tool we will be invoking, while the second member is
    // the folder of the host toolchain for that tool which is essential when
    // using a cross linker. We return a Vec since on x64 there are often two
    // linkers that can target the architecture we desire. The 64-bit host
    // linker is preferred, and hence first, due to 64-bit allowing it more
    // address space to work with and potentially being faster.
    fn bin_subdir(target: &str) -> Vec<(&'static str, &'static str)> {
        let arch = target.split('-').next().unwrap();
        match (arch, host_arch()) {
            ("i586", X86) | ("i686", X86) => vec![("", "")],
            ("i586", X86_64) | ("i686", X86_64) => vec![("amd64_x86", "amd64"), ("", "")],
            ("x86_64", X86) => vec![("x86_amd64", "")],
            ("x86_64", X86_64) => vec![("amd64", "amd64"), ("x86_amd64", "")],
            ("arm", X86) | ("thumbv7a", X86) => vec![("x86_arm", "")],
            ("arm", X86_64) | ("thumbv7a", X86_64) => vec![("amd64_arm", "amd64"), ("x86_arm", "")],
            _ => vec![],
        }
    }

    fn lib_subdir(target: &str) -> Option<&'static str> {
        let arch = target.split('-').next().unwrap();
        match arch {
            "i586" | "i686" => Some("x86"),
            "x86_64" => Some("x64"),
            "arm" | "thumbv7a" => Some("arm"),
            "aarch64" => Some("arm64"),
            _ => None,
        }
    }

    // MSVC's x86 libraries are not in a subfolder
    fn vc_lib_subdir(target: &str) -> Option<&'static str> {
        let arch = target.split('-').next().unwrap();
        match arch {
            "i586" | "i686" => Some(""),
            "x86_64" => Some("amd64"),
            "arm" | "thumbv7a" => Some("arm"),
            "aarch64" => Some("arm64"),
            _ => None,
        }
    }

    #[allow(bad_style)]
    fn host_arch() -> u16 {
        type DWORD = u32;
        type WORD = u16;
        type LPVOID = *mut u8;
        type DWORD_PTR = usize;

        #[repr(C)]
        struct SYSTEM_INFO {
            wProcessorArchitecture: WORD,
            _wReserved: WORD,
            _dwPageSize: DWORD,
            _lpMinimumApplicationAddress: LPVOID,
            _lpMaximumApplicationAddress: LPVOID,
            _dwActiveProcessorMask: DWORD_PTR,
            _dwNumberOfProcessors: DWORD,
            _dwProcessorType: DWORD,
            _dwAllocationGranularity: DWORD,
            _wProcessorLevel: WORD,
            _wProcessorRevision: WORD,
        }

        extern "system" {
            fn GetNativeSystemInfo(lpSystemInfo: *mut SYSTEM_INFO);
        }

        unsafe {
            let mut info = mem::zeroed();
            GetNativeSystemInfo(&mut info);
            info.wProcessorArchitecture
        }
    }

    // Given a registry key, look at all the sub keys and find the one which has
    // the maximal numeric value.
    //
    // Returns the name of the maximal key as well as the opened maximal key.
    fn max_version(key: &RegistryKey) -> Option<(OsString, RegistryKey)> {
        let mut max_vers = 0;
        let mut max_key = None;
        for subkey in key.iter().filter_map(|k| k.ok()) {
            let val = subkey
                .to_str()
                .and_then(|s| s.trim_left_matches("v").replace(".", "").parse().ok());
            let val = match val {
                Some(s) => s,
                None => continue,
            };
            if val > max_vers {
                if let Ok(k) = key.open(&subkey) {
                    max_vers = val;
                    max_key = Some((subkey, k));
                }
            }
        }
        max_key
    }

    pub fn has_msbuild_version(version: &str) -> bool {
        match version {
            "17.0" => {
                find_msbuild_vs17("x86_64-pc-windows-msvc").is_some()
                    || find_msbuild_vs17("i686-pc-windows-msvc").is_some()
                    || find_msbuild_vs17("aarch64-pc-windows-msvc").is_some()
            }
            "16.0" => {
                find_msbuild_vs16("x86_64-pc-windows-msvc").is_some()
                    || find_msbuild_vs16("i686-pc-windows-msvc").is_some()
                    || find_msbuild_vs16("aarch64-pc-windows-msvc").is_some()
            }
            "15.0" => {
                find_msbuild_vs15("x86_64-pc-windows-msvc").is_some()
                    || find_msbuild_vs15("i686-pc-windows-msvc").is_some()
                    || find_msbuild_vs15("aarch64-pc-windows-msvc").is_some()
            }
            "12.0" | "14.0" => LOCAL_MACHINE
                .open(&OsString::from(format!(
                    "SOFTWARE\\Microsoft\\MSBuild\\ToolsVersions\\{}",
                    version
                )))
                .is_ok(),
            _ => false,
        }
    }

    pub fn find_devenv(target: &str) -> Option<Tool> {
        find_devenv_vs15(&target)
    }

    fn find_devenv_vs15(target: &str) -> Option<Tool> {
        find_tool_in_vs15_path(r"Common7\IDE\devenv.exe", target)
    }

    // see http://stackoverflow.com/questions/328017/path-to-msbuild
    pub fn find_msbuild(target: &str) -> Option<Tool> {
        // VS 15 (2017) changed how to locate msbuild
        if let Some(r) = find_msbuild_vs16(target) {
            return Some(r);
        } else if let Some(r) = find_msbuild_vs15(target) {
            return Some(r);
        } else {
            find_old_msbuild(target)
        }
    }

    fn find_msbuild_vs15(target: &str) -> Option<Tool> {
        find_tool_in_vs15_path(r"MSBuild\15.0\Bin\MSBuild.exe", target)
    }

    fn find_old_msbuild(target: &str) -> Option<Tool> {
        let key = r"SOFTWARE\Microsoft\MSBuild\ToolsVersions";
        LOCAL_MACHINE
            .open(key.as_ref())
            .ok()
            .and_then(|key| {
                max_version(&key).and_then(|(_vers, key)| key.query_str("MSBuildToolsPath").ok())
            })
            .map(|path| {
                let mut path = PathBuf::from(path);
                path.push("MSBuild.exe");
                let mut tool = Tool::with_family(path, MSVC_FAMILY);
                if target.contains("x86_64") {
                    tool.env.push(("Platform".into(), "X64".into()));
                }
                tool
            })
    }
}
