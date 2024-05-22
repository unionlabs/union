use std::convert::TryInto;

use crate::error::ErrorKind;

/// Builds the cargo project located at `project_path` and returns the generated wasm file contents.
///
/// NOTE: This function does not check whether the resulting wasm file is a valid smart
/// contract or not.
pub async fn compile_project(project_path: &str) -> crate::Result<Vec<u8>> {
    let project_path = std::fs::canonicalize(project_path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => ErrorKind::Io.message(format!(
            "Incorrect file supplied to compile_project('{}')",
            project_path
        )),
        _ => ErrorKind::Io.custom(e),
    })?;

    let cargo_near_build_command = cargo_near::BuildCommand {
        release: true,
        embed_abi: true,
        doc: false,
        color: cargo_near::ColorPreference::Always,
        no_abi: true,
        out_dir: None,
        manifest_path: Some(
            project_path
                .join("Cargo.toml")
                .try_into()
                .map_err(|e| ErrorKind::Io.custom(e))?,
        ),
    };

    let compile_artifact =
        cargo_near::build::run(cargo_near_build_command).map_err(|e| ErrorKind::Io.custom(e))?;

    let file = compile_artifact
        .path
        .canonicalize()
        .map_err(|e| ErrorKind::Io.custom(e))?;
    tokio::fs::read(file)
        .await
        .map_err(|e| ErrorKind::Io.custom(e))
}
