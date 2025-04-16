# buildPnpmPackage.nix
# ===
# This function allows for easy construct `buildNpmPackage` derivation with
# proper defaults for integrating with PNPM Workspaces.

{ pkgs, lib, ... }:
args@{
  # to derive pname and version
  packageJsonPath,
  # to provision sources additional to monorepo boilerplate
  extraSrcs,
  # workspace project names required for build (e.g. anything with "workspace:*" verison declaration)
  pnpmWorkspaces ? [ ],
  hash ? lib.fakeHash,
  pnpm ? pkgs.pnpm_10,
  ...
}:

let
  src =
    with lib.fileset;
    (toSource {
      root = ./../..;
      fileset = unions (
        [
          ../../package.json
          ../../patches
          ../../pnpm-lock.yaml
          ../../pnpm-workspace.yaml
          ../../tsconfig.base.json
          ../../vitest.setup.ts
          ../../vitest.shared.ts
          ../../vitest.workspace.ts
        ]
        ++ extraSrcs
      );
    });
  packageJson = lib.importJSON packageJsonPath;
  pname = packageJson.name;
  inherit (packageJson) version;
  pnpmDeps = pnpm.fetchDeps {
    inherit
      hash
      pname
      pnpmWorkspaces
      src
      version
      ;
  };
in
pkgs.buildNpmPackage (
  args
  // rec {
    inherit
      pname
      pnpmDeps
      pnpmWorkspaces
      src
      version
      ;
    npmConfigHook = pnpm.configHook;
    npmDeps = pnpmDeps;
  }
)
