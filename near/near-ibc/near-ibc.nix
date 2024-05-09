{ ... }: {
  perSystem = { self', lib, pkgs, system, config, crane, stdenv, dbg, ... }:
    let
      near-ibc = crane.buildWasmContract {
        crateDirFromRoot = "near/near-ibc";
      };

      near-integration-tests = crane.buildWorkspaceMember {
        crateDirFromRoot = "near/near-ibc";
        cargoBuildExtraArgs = "--example integration-tests";
      };

      near-sandbox = pkgs.rustPlatform.buildRustPackage rec {
        pname = "neard";
        version = "9f5e20b29f1a15a00fc50d6051b3b44bb6db60b6";

        src = pkgs.fetchFromGitHub {
          owner = "near";
          repo = "nearcore";
          rev = version;
          hash = "sha256-dMwVXDpIJHbLpKqXfgqV9bbr4j7RtPwhe1grdwKvfr0=";
        };

        cargoLock = {
          lockFile = "${src}/Cargo.lock";
          allowBuiltinFetchGit = true;
        };
      };
    in
    {
      packages.near-sandbox = near-sandbox;
      
    };
}
