{ inputs, ... }: {
  perSystem = { pkgs, rust, system, lib, ... }:
    let
      crane = inputs.crane;
      withBuildTarget = target:
        let
          toolchain = pkgs.rust-bin.fromRustupToolchain {
            inherit (rust.config) channel profile;
            components = rust.config.components ++ [ "cargo" "rustc" "rust-src" ];
            # hopefully if we ever use wasi this issue will be resolved: pkgs.rust.toRustTarget pkgs.hostPlatform
            targets = [ target (pkgs.rust.toRustTarget pkgs.hostPlatform) ];
          };
        in
        crane.lib.${system}.overrideToolchain (toolchain) // { inherit toolchain; };
      craneLib = crane.lib.${system}.overrideToolchain rust.nightly;

      mkChecks = pkgName: checks: pkgs.lib.mapAttrs' (name: value: { name = "${pkgName}-${name}"; value = value; }) checks;

      rustSrc =
        let
          unionvisor-testdata = path: _type: (builtins.match ".*unionvisor/src/testdata/.*" path) != null;
          jsonFilter = path: _type: (builtins.match ".*json$" path) != null;
          jsonOrCargo = path: type:
            (unionvisor-testdata path type) || (jsonFilter path type) || (craneLib.filterCargoSources path type);
        in
        lib.cleanSourceWith {
          src = craneLib.path ../../.;
          filter = jsonOrCargo;
        };

      commonAttrs = {
        # fake values to suppress warnings; see https://github.com/ipetkov/crane/issues/281
        pname = "union-workspace";
        version = "v0.0.0";

        src = rustSrc;
        buildInputs = [ pkgs.pkg-config pkgs.openssl ]
          ++ (
          pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
            Security
          ])
        );
        doCheck = false;
        cargoExtraArgs = "--workspace --exclude ethereum-consensus --exclude ethereum-verifier";
        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
      };

      cargoArtifacts = craneLib.buildDepsOnly commonAttrs;

    in
    {
      _module.args = rec {
        crane = {
          lib = craneLib;
          hostTarget = pkgs.rust.toRustTarget pkgs.hostPlatform;
          inherit withBuildTarget cargoArtifacts commonAttrs mkChecks rustSrc;
          buildWasmContract = import ./buildWasmContract.nix { inherit crane pkgs lib; };
        };
      };
    };
}
